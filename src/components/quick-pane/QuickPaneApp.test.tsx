import { StrictMode } from 'react'
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { emit } from '@tauri-apps/api/event'
import { QuickPaneApp } from './QuickPaneApp'
import { dismissQuickPane } from '@/services/quick-pane'
import { applyLanguage } from '@/i18n/config'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/components/ThemeProvider'
import { mockSystemTheme } from '@/test/system-theme'
const native = vi.hoisted(() => ({ focus: vi.fn(), stop: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn().mockResolvedValue(undefined) }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ onFocusChanged: native.focus }),
}))
vi.mock('@/services/quick-pane', () => ({ dismissQuickPane: vi.fn().mockResolvedValue(undefined) }))
beforeEach(async () => {
  native.focus.mockResolvedValue(native.stop)
  await applyLanguage('en')
})
it('Escape never submits entry', async () => {
  render(<QuickPaneApp />)
  await userEvent.type(screen.getByRole('textbox'), 'synthetic entry')
  await userEvent.keyboard('{Escape}')
  expect(dismissQuickPane).toHaveBeenCalledOnce()
  expect(emit).not.toHaveBeenCalled()
  expect(screen.getByRole('textbox')).toHaveValue('')
})
it('submits once despite repeated Enter and never emits empty text', async () => {
  render(<QuickPaneApp />)
  await userEvent.keyboard('{Enter}')
  expect(emit).not.toHaveBeenCalled()
  await userEvent.type(screen.getByRole('textbox'), 'sample')
  fireEvent.keyDown(screen.getByRole('textbox'), { key: 'Enter', repeat: true })
  expect(emit).not.toHaveBeenCalled()
  await userEvent.keyboard('{Enter}')
  await waitFor(() =>
    expect(emit).toHaveBeenCalledExactlyOnceWith('quick-pane-submit', { text: 'sample' }),
  )
})
it('cancels on blur and cleans late focus listeners through StrictMode', async () => {
  const view = render(
    <StrictMode>
      <QuickPaneApp />
    </StrictMode>,
  )
  await userEvent.type(screen.getByRole('textbox'), 'discard')
  const handler = native.focus.mock.calls[native.focus.mock.calls.length - 1]?.[0]
  handler({ payload: false })
  await waitFor(() => expect(screen.getByRole('textbox')).toHaveValue(''))
  expect(emit).not.toHaveBeenCalled()
  view.unmount()
  await waitFor(() => expect(native.stop).toHaveBeenCalledTimes(2))
})
it('uses translated entry text', async () => {
  await applyLanguage('fr')
  render(<QuickPaneApp />)
  expect(screen.getByRole('textbox')).toHaveAttribute('placeholder', 'Saisissez du texte…')
})
it('keeps an entry available when submission fails', async () => {
  vi.mocked(emit).mockRejectedValueOnce(new Error('private OS detail'))
  render(<QuickPaneApp />)
  await userEvent.type(screen.getByRole('textbox'), 'retry me{Enter}')
  await waitFor(() => expect(screen.getByRole('textbox')).toHaveValue('retry me'))
  expect(screen.getByRole('alert')).not.toHaveTextContent('private')
})
function deferredEmission() {
  let resolve!: () => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<void>((done, fail) => {
    resolve = done
    reject = fail
  })
  return { promise, resolve, reject }
}
it.each([
  { cancel: 'blur', outcome: 'reject' },
  { cancel: 'Escape', outcome: 'reject' },
  { cancel: 'blur', outcome: 'resolve' },
  { cancel: 'Escape', outcome: 'resolve' },
])('dismisses immediately on $cancel and ignores a late $outcome', async ({ cancel, outcome }) => {
  const emission = deferredEmission()
  vi.mocked(emit).mockReturnValueOnce(emission.promise)
  render(<QuickPaneApp />)
  await userEvent.type(screen.getByRole('textbox'), 'synthetic private entry{Enter}')
  expect(emit).toHaveBeenCalledOnce()
  if (cancel === 'blur') {
    const handler = native.focus.mock.calls[native.focus.mock.calls.length - 1]?.[0]
    act(() => handler({ payload: false }))
  } else await userEvent.keyboard('{Escape}')
  expect(dismissQuickPane).toHaveBeenCalledOnce()
  await act(async () => {
    if (outcome === 'reject') emission.reject(new Error('private native failure'))
    else emission.resolve()
  })
  expect(screen.getByRole('textbox')).toHaveValue('')
  expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  expect(dismissQuickPane).toHaveBeenCalledOnce()
})
it('a canceled submission cannot restore text or hide a reopened entry', async () => {
  const emission = deferredEmission()
  vi.mocked(emit).mockReturnValueOnce(emission.promise)
  render(<QuickPaneApp />)
  await userEvent.type(screen.getByRole('textbox'), 'discard me{Enter}{Escape}')
  const handler = native.focus.mock.calls[native.focus.mock.calls.length - 1]?.[0]
  act(() => handler({ payload: true }))
  await userEvent.type(screen.getByRole('textbox'), 'new interaction')
  await act(async () => emission.reject(new Error('late failure')))
  expect(screen.getByRole('textbox')).toHaveValue('new interaction')
  expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  expect(dismissQuickPane).toHaveBeenCalledOnce()
})
it('a late older submission does not release a newer submission guard', async () => {
  const older = deferredEmission()
  const newer = deferredEmission()
  vi.mocked(emit).mockReturnValueOnce(older.promise).mockReturnValueOnce(newer.promise)
  render(<QuickPaneApp />)
  await userEvent.type(screen.getByRole('textbox'), 'older{Enter}{Escape}')
  const handler = native.focus.mock.calls[native.focus.mock.calls.length - 1]?.[0]
  act(() => handler({ payload: true }))
  await userEvent.type(screen.getByRole('textbox'), 'newer{Enter}')
  expect(emit).toHaveBeenCalledTimes(2)
  await act(async () => older.reject(new Error('old failure')))
  await userEvent.keyboard('{Enter}')
  expect(emit).toHaveBeenCalledTimes(2)
  expect(dismissQuickPane).toHaveBeenCalledOnce()
  await act(async () => newer.reject(new Error('new failure')))
  expect(screen.getByRole('textbox')).toHaveValue('newer')
})
it('uses the shared provider for live appearance changes', async () => {
  mockSystemTheme()
  const client = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity } } })
  client.setQueryData(['preferences'], {
    theme: 'light',
    language: 'en',
    quick_pane_shortcut: null,
  })
  render(
    <QueryClientProvider client={client}>
      <ThemeProvider>
        <QuickPaneApp />
      </ThemeProvider>
    </QueryClientProvider>,
  )
  expect(document.documentElement).not.toHaveClass('dark')
  act(() =>
    client.setQueryData(['preferences'], {
      theme: 'dark',
      language: 'en',
      quick_pane_shortcut: null,
    }),
  )
  await waitFor(() => expect(document.documentElement).toHaveClass('dark'))
})
