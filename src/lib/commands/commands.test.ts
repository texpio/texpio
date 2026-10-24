import { beforeEach, describe, expect, it, vi } from 'vitest'
import { executeCommand, registerCommands, getAllCommands, type CommandContext } from './index'
import { createKeyboardDispatcher } from '@/hooks/use-keyboard-shortcuts'
import { createMenuAction, formatShortcut } from '@/lib/menu'
import { useUIStore, normalizeLayout } from '@/store/ui-store'

let context: CommandContext
beforeEach(() => {
  context = {
    platform: 'windows',
    openPreferences: vi.fn(),
    showToast: vi.fn(),
    windowAction: vi.fn().mockResolvedValue({ success: true }),
  }
  useUIStore.setState({
    layout: normalizeLayout(null),
    activeView: 'home',
    commandPaletteOpen: false,
  })
})
describe('common command registry', () => {
  it('unavailable commands do not execute', async () => {
    const action = vi.fn()
    registerCommands([
      {
        id: 'test.unavailable',
        labelKey: 'unavailable',
        execute: action,
        isAvailable: () => false,
      },
    ])
    await expect(executeCommand('test.unavailable', context)).resolves.toEqual({
      success: false,
      code: 'unavailable',
    })
    expect(action).not.toHaveBeenCalled()
  })
  it('returns only closed failure codes', async () => {
    registerCommands([
      {
        id: 'test.error',
        labelKey: 'error',
        execute: () => {
          throw new Error('C:/private/secret')
        },
      },
    ])
    await expect(executeCommand('test.error', context)).resolves.toEqual({
      success: false,
      code: 'failed',
    })
    await expect(executeCommand('C:/private/unknown', context)).resolves.toEqual({
      success: false,
      code: 'unknownCommand',
    })
  })
  it('menu, palette and key adapters invoke the same action exactly once', async () => {
    const action = vi.fn()
    registerCommands([
      { id: 'test.shared', labelKey: 'shared', shortcut: 'CmdOrCtrl+J', execute: action },
    ])
    await executeCommand('test.shared', context)
    expect(action).toHaveBeenCalledTimes(1)
    await createMenuAction('test.shared', context)()
    expect(action).toHaveBeenCalledTimes(2)
    const dispatch = createKeyboardDispatcher(context)
    dispatch(new KeyboardEvent('keydown', { key: 'j', ctrlKey: true, cancelable: true }))
    expect(action).toHaveBeenCalledTimes(3)
    for (const options of [
      { repeat: true },
      { shiftKey: true },
      { altKey: true },
      { metaKey: true },
      { isComposing: true },
    ])
      dispatch(new KeyboardEvent('keydown', { key: 'j', ctrlKey: true, ...options }))
    expect(action).toHaveBeenCalledTimes(3)
  })
  it('keeps editable fields and already handled events intact', () => {
    const input = document.createElement('input')
    document.body.append(input)
    const dispatch = createKeyboardDispatcher(context)
    input.addEventListener('keydown', dispatch)
    const key = new KeyboardEvent('keydown', { key: '1', ctrlKey: true, cancelable: true })
    input.dispatchEvent(key)
    expect(key.defaultPrevented).toBe(false)
    expect(useUIStore.getState().layout.leftVisible).toBe(true)
    input.remove()
    const handled = new KeyboardEvent('keydown', { key: ',', ctrlKey: true, cancelable: true })
    handled.preventDefault()
    dispatch(handled)
    expect(context.openPreferences).not.toHaveBeenCalled()
  })
  it('uses the correct platform modifier and opens preferences consistently', async () => {
    const dispatch = createKeyboardDispatcher({ ...context, platform: 'macos' })
    dispatch(new KeyboardEvent('keydown', { key: ',', ctrlKey: true }))
    expect(context.openPreferences).not.toHaveBeenCalled()
    dispatch(new KeyboardEvent('keydown', { key: ',', metaKey: true }))
    await createMenuAction('open-preferences', context)()
    expect(context.openPreferences).toHaveBeenCalledTimes(2)
    expect(formatShortcut('CmdOrCtrl+K', 'macos')).toBe('⌘K')
    expect(formatShortcut('CmdOrCtrl+K', 'windows')).toBe('Ctrl+K')
  })
  it('searches translated labels, descriptions and keywords', () => {
    registerCommands([
      {
        id: 'test.search',
        labelKey: 'search.label',
        descriptionKey: 'search.description',
        keywords: ['settings'],
        execute: vi.fn(),
      },
    ])
    const translate = (key: string) =>
      key === 'search.label'
        ? 'Préférences'
        : key === 'search.description'
          ? 'Choisir la langue'
          : key
    for (const search of ['pref', 'langue', 'settings'])
      expect(getAllCommands(context, search, translate).map((command) => command.id)).toContain(
        'test.search',
      )
  })
})
