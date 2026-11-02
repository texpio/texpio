import { act, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, expect, it, vi } from 'vitest'
import { EngineControls } from './EngineControls'
import { ExpansionProvider } from '@/features/expansion/ExpansionProvider'
import { ExpansionNotice } from '@/features/expansion/ExpansionNotice'
import { useUIStore } from '@/store/ui-store'

const native = vi.hoisted(() => ({
  status: vi.fn(),
  stop: vi.fn(),
  start: vi.fn(),
  pause: vi.fn(),
  library: vi.fn(),
}))
const currentPlatform = vi.hoisted(() => ({ value: 'unknown' }))
vi.mock('@/hooks/use-platform', () => ({
  usePlatform: () => currentPlatform.value,
  useIsMacOS: () => currentPlatform.value === 'macos',
}))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@/lib/bindings', () => ({
  commands: {
    productStatus: native.status,
    productStop: native.stop,
    productStart: native.start,
    productPause: native.pause,
    libraryLoad: native.library,
  },
}))
const stopped = {
  phase: 'stopped',
  submitted: 0,
  cancelled: 0,
  lastError: null,
  clipboardIncomplete: false,
}
const library = {
  schemaVersion: 1,
  revision: 1,
  folders: [],
  snippets: [
    {
      id: '00000000-0000-0000-0000-000000000001',
      folderId: null,
      title: 'Saved',
      trigger: ';saved',
      body: 'Saved body',
      enabled: true,
      deleted: false,
    },
  ],
}
beforeEach(() => {
  vi.clearAllMocks()
  currentPlatform.value = 'unknown'
  useUIStore.setState({ expansionStartBlocked: false })
  native.status.mockResolvedValue(stopped)
  native.library.mockResolvedValue({ status: 'ok', data: library })
  for (const [action, phase] of [
    [native.start, 'running'],
    [native.stop, 'stopped'],
    [native.pause, 'paused'],
  ] as const) {
    action.mockImplementation(async () => {
      const next = { ...stopped, phase }
      native.status.mockResolvedValue(next)
      return { status: 'ok', data: next }
    })
  }
})
function show() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const ui = (controls: boolean) => (
    <QueryClientProvider client={client}>
      <ExpansionProvider>
        <ExpansionNotice />
        {controls && <EngineControls />}
      </ExpansionProvider>
    </QueryClientProvider>
  )
  const result = render(ui(true))
  return { ...result, client, hideControls: () => result.rerender(ui(false)) }
}
it('keeps running controls in Settings and shows a persistent warning after pause until turned on', async () => {
  native.status.mockResolvedValue({ ...stopped, phase: 'running' })
  show()
  await screen.findByText('Running')
  expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  await userEvent.click(screen.getByRole('button', { name: 'Pause' }))
  expect(await screen.findByRole('alert')).toHaveTextContent('Expansion is paused')
  await userEvent.click(screen.getByRole('button', { name: 'Turn on expansion' }))
  await waitFor(() => expect(screen.queryByRole('alert')).not.toBeInTheDocument())
  expect(native.start).toHaveBeenCalledOnce()
})
it('routes macOS input blocking to the in-app permission guide instead of a doomed start', async () => {
  currentPlatform.value = 'macos'
  native.status.mockResolvedValue({ ...stopped, lastError: 'inputBlocked' })
  show()
  const review = await screen.findByRole('button', { name: 'Review permissions' })
  expect(screen.queryByRole('button', { name: 'Turn on expansion' })).not.toBeInTheDocument()
  await userEvent.click(review)
  expect(useUIStore.getState().activeView).toBe('preferences')
  expect(useUIStore.getState().preferencesSection).toBe('expansion')
})
it('allows fault cleanup before enabling Start', async () => {
  native.status.mockResolvedValue({ ...stopped, phase: 'fault', lastError: 'busy' })
  show()
  await screen.findByText('Fault')
  expect(screen.getByRole('button', { name: 'Start expansion' })).toBeDisabled()
  await userEvent.click(screen.getByRole('button', { name: 'Reset expansion' }))
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Turn on expansion' })).toBeEnabled(),
  )
  expect(native.stop).toHaveBeenCalledOnce()
  expect(native.start).not.toHaveBeenCalled()
})
it.each(['empty', 'disabled', 'trash', 'dirty'] as const)(
  'blocks starting with %s library state',
  async (kind) => {
    if (kind === 'dirty') useUIStore.setState({ expansionStartBlocked: true })
    else
      native.library.mockResolvedValue({
        status: 'ok',
        data: {
          ...library,
          snippets:
            kind === 'empty'
              ? []
              : [
                  {
                    ...library.snippets[0],
                    enabled: kind !== 'disabled',
                    deleted: kind === 'trash',
                  },
                ],
        },
      })
    show()
    await screen.findByText('Stopped')
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Turn on expansion' })).toBeDisabled(),
    )
    await userEvent.click(screen.getByRole('button', { name: 'Turn on expansion' }))
    expect(native.start).not.toHaveBeenCalled()
  },
)
it('shares pending actions and errors when Settings is unmounted', async () => {
  let reject!: (reason: unknown) => void
  native.start.mockReturnValueOnce(
    new Promise((_, fail) => {
      reject = fail
    }),
  )
  const view = show()
  const turnOn = await screen.findByRole('button', { name: 'Turn on expansion' })
  await waitFor(() => expect(turnOn).toBeEnabled())
  await userEvent.click(turnOn)
  expect(screen.getByRole('button', { name: 'Start expansion' })).toBeDisabled()
  view.hideControls()
  await userEvent.click(screen.getByRole('button', { name: 'Turning on…' }))
  expect(native.start).toHaveBeenCalledOnce()
  await act(async () => reject('inputBlocked'))
  expect(
    await screen.findByText('Input is blocked by the system. Check the text before trying again.'),
  ).toBeVisible()
  expect(screen.getByRole('alert')).toHaveTextContent('Expansion is off')
})
it('recovers unavailable status through Retry without starting expansion', async () => {
  native.status.mockRejectedValueOnce('unavailable')
  show()
  expect(await screen.findByRole('alert')).toHaveTextContent('Expansion status is unavailable')
  await userEvent.click(screen.getByRole('button', { name: 'Retry' }))
  await screen.findByRole('button', { name: 'Turn on expansion' })
  expect(native.start).not.toHaveBeenCalled()
})
it('keeps incomplete clipboard restoration visible even while running', async () => {
  native.status.mockResolvedValue({ ...stopped, phase: 'running', clipboardIncomplete: true })
  show()
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'Some clipboard formats could not be restored.',
  )
})
it('does not claim expansion is off when a later status check fails with cached paused data', async () => {
  native.status.mockResolvedValue({ ...stopped, phase: 'paused' })
  const { client } = show()
  await screen.findByText('Expansion is paused')
  native.status.mockRejectedValue('unavailable')
  await act(async () => {
    await client.refetchQueries({ queryKey: ['library-engine'] })
  })
  const notice = screen.getByRole('alert')
  await waitFor(() => expect(notice).toHaveTextContent('Expansion status is unavailable'))
  expect(notice).not.toHaveTextContent('Your abbreviations won’t expand')
  expect(screen.queryByRole('button', { name: 'Turn on expansion' })).not.toBeInTheDocument()
})
