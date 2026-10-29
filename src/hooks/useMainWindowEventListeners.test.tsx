import { renderHook } from '@testing-library/react'
import { expect, it, vi } from 'vitest'
import { useMainWindowEventListeners } from './useMainWindowEventListeners'

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('./use-command-context', () => ({ useCommandContext: () => ({}) }))
vi.mock('./use-keyboard-shortcuts', () => ({ useKeyboardShortcuts: () => {} }))
vi.mock('@/lib/menu', () => ({ startNativeMenu: () => ({ dispose: async () => {} }) }))
vi.mock('@/lib/recovery', () => ({ cleanupOldRecoveryFiles: async () => {} }))
vi.mock('@/lib/context-menu', () => ({
  createEditContextMenu: () => ({ show: async () => {}, dispose: async () => {} }),
}))

it('lets the webview show its native context menu for editable text', () => {
  renderHook(useMainWindowEventListeners)
  const editor = document.createElement('div')
  editor.setAttribute('contenteditable', 'true')
  document.body.appendChild(editor)
  const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true })
  editor.dispatchEvent(event)
  editor.remove()
  expect(event.defaultPrevented).toBe(false)
})
