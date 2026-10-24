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
  clipboardMode: 'type',
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
