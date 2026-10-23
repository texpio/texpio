import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, it, vi } from 'vitest'
import { ShortcutPicker } from './ShortcutPicker'
it('captures a chord once and Escape cancels another capture', async () => {
  const change = vi.fn()
  render(
    <ShortcutPicker
      value="CommandOrControl+Shift+."
      defaultValue="CommandOrControl+Shift+."
      onChange={change}
    />,
  )
  const button = screen.getByRole('button', { name: /Quick Pane Shortcut/ })
  await userEvent.click(button)
  fireEvent.keyDown(button, { code: 'KeyJ', key: 'J', ctrlKey: true, shiftKey: true })
  fireEvent.keyUp(button, { code: 'KeyJ', key: 'J' })
  expect(change).toHaveBeenCalledExactlyOnceWith('CommandOrControl+Shift+J')
  await userEvent.click(button)
  fireEvent.keyDown(button, { key: 'Escape', code: 'Escape' })
  fireEvent.keyUp(button, { key: 'Escape', code: 'Escape' })
  expect(change).toHaveBeenCalledOnce()
})
