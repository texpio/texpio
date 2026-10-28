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
  allowSoftwareInput: false,
  activation: 'immediate',
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
it('preserves the terminating character preference when switching activation modes', async () => {
  show()
  const activation = await screen.findByRole('combobox', { name: 'Expand abbreviation' })
  const terminator = screen.getByRole('switch', { name: 'Keep the terminating character' })
  await waitFor(() => expect(activation).toBeEnabled())
  expect(activation).toHaveValue('immediate')
  expect(terminator).toBeDisabled()
  expect(terminator).toBeChecked()

  await userEvent.selectOptions(activation, 'terminator')
  await waitFor(() =>
    expect(native.save).toHaveBeenLastCalledWith({ ...settings, activation: 'terminator' }),
  )
  await waitFor(() => expect(terminator).toBeEnabled())
  expect(activation).toHaveValue('terminator')
  expect(terminator).toBeChecked()

  await userEvent.click(terminator)
  await waitFor(() =>
    expect(native.save).toHaveBeenLastCalledWith({
      ...settings,
      activation: 'terminator',
      preserveTerminator: false,
    }),
  )
  await waitFor(() => expect(activation).toBeEnabled())
  await userEvent.selectOptions(activation, 'immediate')
  await waitFor(() =>
    expect(native.save).toHaveBeenLastCalledWith({ ...settings, preserveTerminator: false }),
  )
  await waitFor(() => expect(activation).toHaveValue('immediate'))
  expect(terminator).toBeDisabled()
  expect(terminator).not.toBeChecked()
})
it('keeps immediate activation and its disabled terminator control when a save fails', async () => {
  native.save.mockRejectedValue(new Error('unavailable'))
  show()
  const activation = await screen.findByRole('combobox', { name: 'Expand abbreviation' })
  await waitFor(() => expect(activation).toBeEnabled())
  await userEvent.selectOptions(activation, 'terminator')
  expect(await screen.findByRole('alert')).toBeVisible()
  expect(activation).toHaveValue('immediate')
  expect(screen.getByRole('switch', { name: 'Keep the terminating character' })).toBeDisabled()
})
it('requires opting in to remote keyboard compatibility and retains other native settings', async () => {
  show()
  const compatibility = await screen.findByRole('switch', {
    name: 'Remote keyboard and key remapper compatibility',
  })
  await waitFor(() => expect(compatibility).toBeEnabled())
  expect(compatibility).not.toBeChecked()
  await userEvent.click(compatibility)
  await waitFor(() =>
    expect(native.save).toHaveBeenLastCalledWith({ ...settings, allowSoftwareInput: true }),
  )
  expect(compatibility).toBeChecked()
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
  expect(
    screen.queryByRole('switch', { name: 'Start expansion when Texpio launches' }),
  ).not.toBeInTheDocument()
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
  expect(
    [...shortcut.parentElement!.querySelectorAll('[data-slot="kbd"]')].map(
      (key) => key.textContent,
    ),
  ).toEqual(['Ctrl', 'Shift', 'Space'])
  await userEvent.clear(shortcut)
  await userEvent.type(shortcut, 'Ctrl+Alt+T')
  expect(
    [...shortcut.parentElement!.querySelectorAll('[data-slot="kbd"]')].map(
      (key) => key.textContent,
    ),
  ).toEqual(['Ctrl', 'Alt', 'T'])
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
      'A keyboard shortcut is unavailable. Choose another shortcut in Settings.',
    ),
  ).toBeVisible()
  expect(screen.getByLabelText('Insertion method')).toHaveValue('paste')
})
it('restores the acknowledged shortcut display after a shortcut save fails', async () => {
  native.save.mockRejectedValue(
    Object.assign(new Error('unavailable'), { code: 'shortcutUnavailable' }),
  )
  show()
  await waitFor(() => expect(screen.getByLabelText('Expansion toggle shortcut')).toBeEnabled())
  const input = screen.getByLabelText('Expansion toggle shortcut')
  await userEvent.clear(input)
  await userEvent.type(input, 'Ctrl+Alt+T')
  await userEvent.tab()
  await screen.findByRole('alert')
  const restored = screen.getByLabelText('Expansion toggle shortcut')
  expect(restored).toHaveValue('CmdOrCtrl+Shift+Space')
  expect(
    [...restored.parentElement!.querySelectorAll('[data-slot="kbd"]')].map(
      (key) => key.textContent,
    ),
  ).toEqual(['Ctrl', 'Shift', 'Space'])
})
