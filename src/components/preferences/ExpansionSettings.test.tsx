import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, expect, it, vi } from 'vitest'
import { ExpansionSettings } from './ExpansionSettings'
const native = vi.hoisted(() => ({ load: vi.fn(), save: vi.fn() }))
vi.mock('@/services/product', () => ({
  loadProductSettings: native.load,
  saveProductSettings: native.save,
}))
const settings = {
  enabled: false,
  boundary: 'word',
  preserveTerminator: true,
  insertion: 'paste',
  restoreClipboard: true,
  typingSpeed: 'balanced',
  cancelKey: 'Escape',
  clipboardEnabled: true,
  clipboardShortcut: 'CmdOrCtrl+Shift+V',
  clipboardMode: 'paste',
  toggleShortcut: 'CmdOrCtrl+Shift+Space',
  autostart: false,
  closeToTray: true,
  attachmentDelayMs: 500,
}
beforeEach(() => {
  native.load.mockResolvedValue(settings)
  native.save.mockImplementation(async (input) => input)
})
function show() {
  return render(
    <QueryClientProvider
      client={
        new QueryClient({
          defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
        })
      }
    >
      <ExpansionSettings />
    </QueryClientProvider>,
  )
}
it('persists insertion choice while retaining other native settings', async () => {
  show()
  await waitFor(() => expect(screen.getByLabelText('Insertion method')).toBeEnabled())
  await userEvent.selectOptions(await screen.findByLabelText('Insertion method'), 'type')
  await waitFor(() => expect(native.save).toHaveBeenCalledWith({ ...settings, insertion: 'type' }))
  expect(screen.getByLabelText('Insertion method')).toHaveValue('type')
})
it('keeps acknowledged settings and shows an error when a save fails', async () => {
  native.save.mockRejectedValue(new Error('unavailable'))
  show()
  await waitFor(() => expect(screen.getByLabelText('Insertion method')).toBeEnabled())
  await userEvent.selectOptions(await screen.findByLabelText('Insertion method'), 'type')
  expect(await screen.findByRole('alert')).toBeVisible()
  expect(screen.getByLabelText('Insertion method')).toHaveValue('paste')
})
it('persists desktop options and shortcuts without overwriting other settings', async () => {
  show()
  await waitFor(() =>
    expect(screen.getByRole('switch', { name: 'Launch at Windows sign-in' })).toBeEnabled(),
  )
  await userEvent.click(screen.getByRole('switch', { name: 'Launch at Windows sign-in' }))
  await waitFor(() =>
    expect(native.save).toHaveBeenLastCalledWith({ ...settings, autostart: true }),
  )
  await userEvent.click(screen.getByRole('switch', { name: 'Keep running when the window closes' }))
  await waitFor(() =>
    expect(native.save).toHaveBeenLastCalledWith({
      ...settings,
      autostart: true,
      closeToTray: false,
    }),
  )
  const shortcut = screen.getByLabelText('Expansion toggle shortcut')
  await userEvent.clear(shortcut)
  await userEvent.type(shortcut, 'Ctrl+Alt+T')
  await userEvent.tab()
  await waitFor(() =>
    expect(native.save).toHaveBeenLastCalledWith({
      ...settings,
      autostart: true,
      closeToTray: false,
      toggleShortcut: 'Ctrl+Alt+T',
    }),
  )
})
it('explains a native shortcut conflict without discarding acknowledged settings', async () => {
  native.save.mockRejectedValue(
    Object.assign(new Error('unavailable'), { code: 'shortcutUnavailable' }),
  )
  show()
  await waitFor(() => expect(screen.getByLabelText('Insertion method')).toBeEnabled())
  await userEvent.selectOptions(screen.getByLabelText('Insertion method'), 'type')
  expect(
    await screen.findByText(
      'A keyboard shortcut is unavailable. Choose another shortcut in Preferences.',
    ),
  ).toBeVisible()
  expect(screen.getByLabelText('Insertion method')).toHaveValue('paste')
})
