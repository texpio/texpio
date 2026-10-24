import { StrictMode } from 'react'
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { QuickPaneApp } from './QuickPaneApp'
import { dismissQuickPane } from '@/services/quick-pane'
import { applyLanguage } from '@/i18n/config'
import { ThemeProvider } from '@/components/ThemeProvider'
import { mockSystemTheme } from '@/test/system-theme'

const native = vi.hoisted(() => ({
  focus: vi.fn(),
  stop: vi.fn(),
  load: vi.fn(),
  clipboard: vi.fn(),
}))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ onFocusChanged: native.focus }),
}))
vi.mock('@/lib/bindings', () => ({ commands: { libraryLoad: native.load } }))
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: native.clipboard }))
vi.mock('@/services/quick-pane', () => ({ dismissQuickPane: vi.fn().mockResolvedValue(undefined) }))
const data = {
  schemaVersion: 1,
  revision: 4,
  folders: [],
  snippets: [
    {
      id: '10000000-0000-4000-8000-000000000000',
      folderId: null,
      title: 'Greeting',
      trigger: ';hello',
      body: 'Hello, नमस्ते 👋',
      enabled: true,
      deleted: false,
    },
    {
      id: '20000000-0000-4000-8000-000000000000',
      folderId: null,
      title: 'Address',
      trigger: ';addr',
      body: '123 Example Road',
      enabled: true,
      deleted: false,
    },
    {
      id: '30000000-0000-4000-8000-000000000000',
      folderId: null,
      title: 'Disabled',
      trigger: ';disabled',
      body: 'Hidden',
      enabled: false,
      deleted: false,
    },
    {
      id: '40000000-0000-4000-8000-000000000000',
      folderId: null,
      title: 'Deleted',
      trigger: ';deleted',
      body: 'Hidden',
      enabled: true,
      deleted: true,
    },
  ],
}
beforeEach(() => {
  native.focus.mockResolvedValue(native.stop)
  native.load.mockResolvedValue({ status: 'ok', data })
  native.clipboard.mockResolvedValue(undefined)
})
function show(strict = false) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const pane = (
    <QueryClientProvider client={client}>
      <QuickPaneApp />
    </QueryClientProvider>
  )
  return render(strict ? <StrictMode>{pane}</StrictMode> : pane)
}
it('searches snippet bodies and copies a chosen active snippet explicitly', async () => {
  show()
  await screen.findByRole('button', { name: 'Copy Greeting' })
  expect(screen.queryByRole('button', { name: 'Copy Disabled' })).not.toBeInTheDocument()
  expect(screen.queryByRole('button', { name: 'Copy Deleted' })).not.toBeInTheDocument()
  expect(native.clipboard).not.toHaveBeenCalled()
  await userEvent.type(screen.getByRole('searchbox'), 'Example Road')
  expect(screen.queryByRole('button', { name: 'Copy Greeting' })).not.toBeInTheDocument()
  await userEvent.click(screen.getByRole('button', { name: 'Copy Address' }))
  await waitFor(() => expect(native.clipboard).toHaveBeenCalledExactlyOnceWith('123 Example Road'))
  expect(dismissQuickPane).toHaveBeenCalledOnce()
})
it('Escape clears the query and never copies', async () => {
  show()
  await screen.findByRole('button', { name: 'Copy Greeting' })
  await userEvent.type(screen.getByRole('searchbox'), 'synthetic query{Escape}')
  expect(screen.getByRole('searchbox')).toHaveValue('')
  expect(native.clipboard).not.toHaveBeenCalled()
  expect(dismissQuickPane).toHaveBeenCalledOnce()
})
it('repeated Enter cannot issue a second pending clipboard write', async () => {
  native.clipboard.mockImplementation(() => new Promise(() => {}))
  show()
  await screen.findByRole('button', { name: 'Copy Greeting' })
  fireEvent.keyDown(screen.getByRole('searchbox'), { key: 'Enter', repeat: true })
  expect(native.clipboard).not.toHaveBeenCalled()
  await userEvent.keyboard('{Enter}{Enter}')
  expect(native.clipboard).toHaveBeenCalledExactlyOnceWith('Hello, नमस्ते 👋')
  expect(dismissQuickPane).not.toHaveBeenCalled()
})
it('keeps a failed copy available without exposing native details', async () => {
  native.clipboard.mockRejectedValueOnce(new Error('private OS path'))
  show()
  await userEvent.click(await screen.findByRole('button', { name: 'Copy Greeting' }))
  await screen.findByRole('alert')
  expect(screen.getByRole('alert')).not.toHaveTextContent('private')
  expect(screen.getByRole('button', { name: 'Copy Greeting' })).toBeEnabled()
  expect(dismissQuickPane).not.toHaveBeenCalled()
})
it.each(['resolve', 'reject'])(
  'blur dismisses and a late copy %s cannot hide a reopened picker',
  async (outcome) => {
    let resolve!: () => void
    let reject!: (error: Error) => void
    native.clipboard.mockReturnValueOnce(
      new Promise<void>((done, fail) => {
        resolve = done
        reject = fail
      }),
    )
    show()
    await userEvent.click(await screen.findByRole('button', { name: 'Copy Greeting' }))
    const handler = native.focus.mock.calls[native.focus.mock.calls.length - 1]![0]
    act(() => handler({ payload: false }))
    await waitFor(() => expect(dismissQuickPane).toHaveBeenCalledOnce())
    act(() => handler({ payload: true }))
    await userEvent.type(screen.getByRole('searchbox'), 'Address')
    await act(async () => {
      if (outcome === 'resolve') resolve()
      else reject(new Error('late private failure'))
    })
    expect(screen.getByRole('searchbox')).toHaveValue('Address')
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
    expect(dismissQuickPane).toHaveBeenCalledOnce()
  },
)
it('cleans focus listeners through StrictMode and unmount', async () => {
  const view = show(true)
  await screen.findByRole('button', { name: 'Copy Greeting' })
  view.unmount()
  await waitFor(() => expect(native.stop).toHaveBeenCalledTimes(2))
})
it('uses translated search labels', async () => {
  await applyLanguage('fr')
  show()
  expect(screen.getByRole('searchbox', { name: 'Rechercher des textes' })).toHaveAttribute(
    'placeholder',
    'Titre, déclencheur ou contenu…',
  )
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
