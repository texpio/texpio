import { act, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { beforeEach, expect, it, vi } from 'vitest'
import { PreferencesPage } from './PreferencesPage'
import { useUIStore } from '@/store/ui-store'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn().mockResolvedValue(undefined) }))
vi.mock('@/services/quick-pane', () => ({
  showQuickPane: vi.fn(),
  getDefaultQuickPaneShortcut: () => Promise.resolve('CommandOrControl+Shift+.'),
  updateQuickPaneShortcut: vi.fn(),
}))
const product = vi.hoisted(() => ({ load: vi.fn(), save: vi.fn() }))
vi.mock('@/services/product', () => ({
  loadProductSettings: product.load,
  saveProductSettings: product.save,
}))
const productSettings = {
  enabled: false,
  activation: 'immediate',
  boundary: 'word',
  preserveTerminator: true,
  insertion: 'paste',
  restoreClipboard: true,
  typingSpeed: 'balanced',
  cancelKey: 'Escape',
  clipboardEnabled: false,
  clipboardShortcut: 'Ctrl+Shift+V',
  clipboardMode: 'paste',
  allowSoftwareInput: false,
  toggleShortcut: 'Ctrl+Alt+T',
  autostart: false,
  closeToTray: true,
  attachmentDelayMs: 500,
}
const preferences = { theme: 'system', language: 'fr', quick_pane_shortcut: 'CmdOrCtrl+Shift+.' }
beforeEach(() => {
  product.load.mockResolvedValue(productSettings)
  product.save.mockImplementation(async (value) => value)
  vi.mocked(isTauri).mockReturnValue(true)
  useUIStore.setState({ activeView: 'preferences', preferencesSection: 'general' })
})
function open() {
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <PreferencesPage />
    </QueryClientProvider>,
  )
}
it('finds settings sections by option labels without saving and restores navigation when cleared', async () => {
  vi.mocked(invoke).mockResolvedValue(preferences)
  open()
  const search = screen.getByRole('searchbox', { name: 'Search settings' })
  await userEvent.type(search, 'clipboard')
  expect(screen.queryByRole('radio', { name: 'General' })).not.toBeInTheDocument()
  expect(screen.getByRole('radio', { name: 'Expansion' })).toBeChecked()
  expect(screen.getByRole('radio', { name: 'Shortcuts' })).toBeInTheDocument()
  expect(await screen.findByLabelText('Insertion method')).toBeVisible()
  await userEvent.clear(search)
  expect(screen.getByRole('radio', { name: 'General' })).toBeInTheDocument()
  expect(screen.getByRole('radio', { name: 'Expansion' })).toBeChecked()
  await userEvent.type(search, 'no matching setting')
  expect(screen.getByRole('status')).toHaveTextContent('No matching settings')
  await userEvent.clear(search)
  await userEvent.click(screen.getByRole('radio', { name: 'General' }))
  expect(await screen.findByRole('combobox', { name: 'Theme' })).toBeVisible()
  expect(product.save).not.toHaveBeenCalled()
  expect(invoke).not.toHaveBeenCalledWith('save_preferences', expect.anything())
})
it('keeps corrupt preferences out of saves and recovers through Retry', async () => {
  vi.mocked(invoke)
    .mockRejectedValueOnce({ path: 'C:/private/secret' })
    .mockResolvedValue(preferences)
  open()
  expect(await screen.findByRole('alert')).toHaveTextContent('Preferences are unavailable.')
  expect(screen.queryByText(/private/)).not.toBeInTheDocument()
  expect(screen.getByRole('combobox', { name: 'Theme' })).toBeDisabled()
  expect(invoke).not.toHaveBeenCalledWith('save_preferences', expect.anything())
  await userEvent.click(screen.getByRole('button', { name: 'Retry' }))
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Theme' })).toBeEnabled())
})
it('persists theme while retaining language and shortcut', async () => {
  vi.mocked(invoke).mockImplementation(async (command) =>
    command === 'save_preferences' ? null : preferences,
  )
  open()
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Theme' })).toBeEnabled())
  await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Theme' }), 'dark')
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith('save_preferences', {
      preferences: { ...preferences, theme: 'dark' },
    }),
  )
})
it('groups Preferences into page sections instead of stacking them in a dialog', async () => {
  vi.mocked(invoke).mockResolvedValue(preferences)
  open()
  expect(screen.getByRole('heading', { name: 'Preferences' })).toBeInTheDocument()
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  expect(await screen.findByRole('combobox', { name: 'Theme' })).toBeVisible()
  expect(screen.queryByLabelText('Insertion method')).not.toBeInTheDocument()
  await userEvent.click(screen.getByRole('radio', { name: 'Expansion' }))
  expect(await screen.findByLabelText('Insertion method')).toBeVisible()
  expect(screen.queryByLabelText('Theme')).not.toBeInTheDocument()
  await userEvent.click(screen.getByRole('radio', { name: 'Shortcuts' }))
  expect(await screen.findByLabelText('Expansion toggle shortcut')).toBeVisible()
  expect(screen.queryByLabelText('Insertion method')).not.toBeInTheDocument()
})

it('retains a pending product save across sections and merges the acknowledged result into the next save', async () => {
  vi.mocked(invoke).mockResolvedValue(preferences)
  let finish!: (value: unknown) => void
  product.save.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  open()
  const startup = await screen.findByRole('switch', { name: 'Launch at Windows sign-in' })
  await waitFor(() => expect(startup).toBeEnabled())
  await userEvent.click(startup)
  await waitFor(() => expect(product.save).toHaveBeenCalledOnce())
  await userEvent.click(screen.getByRole('radio', { name: 'Expansion' }))
  try {
    expect(screen.getByLabelText('Insertion method')).toBeDisabled()
  } finally {
    await act(async () => finish({ ...productSettings, autostart: true }))
  }
  await waitFor(() => expect(screen.getByLabelText('Insertion method')).toBeEnabled())
  await userEvent.selectOptions(screen.getByLabelText('Insertion method'), 'type')
  await waitFor(() =>
    expect(product.save).toHaveBeenLastCalledWith({
      ...productSettings,
      autostart: true,
      insertion: 'type',
    }),
  )
})
it('keeps a failed product save visible after changing sections', async () => {
  vi.mocked(invoke).mockResolvedValue(preferences)
  let fail!: (error: Error) => void
  product.save.mockReturnValueOnce(
    new Promise((_, reject) => {
      fail = reject
    }),
  )
  open()
  const startup = await screen.findByRole('switch', { name: 'Launch at Windows sign-in' })
  await waitFor(() => expect(startup).toBeEnabled())
  await userEvent.click(startup)
  await waitFor(() => expect(product.save).toHaveBeenCalledOnce())
  await userEvent.click(screen.getByRole('radio', { name: 'Expansion' }))
  await act(async () => fail(new Error('unavailable')))
  expect(await screen.findByRole('alert')).toHaveTextContent('Your previous settings are kept.')
})
