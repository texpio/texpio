import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { beforeEach, expect, it, vi } from 'vitest'
import { PreferencesDialog } from './PreferencesDialog'
import { useUIStore } from '@/store/ui-store'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn().mockResolvedValue(undefined) }))
const preferences = { theme: 'system', language: 'fr', quick_pane_shortcut: 'CmdOrCtrl+Shift+.' }
beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(true)
  useUIStore.setState({ preferencesOpen: true })
})
function open() {
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <PreferencesDialog />
    </QueryClientProvider>,
  )
}
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
