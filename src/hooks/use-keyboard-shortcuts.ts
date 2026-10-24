import { useEffect } from 'react'
import { getAllCommands, runCommand, type CommandContext } from '@/lib/commands'
function editable(target: EventTarget | null) {
  return (
    target instanceof Element &&
    !!target.closest(
      'input,textarea,select,[contenteditable]:not([contenteditable="false"]),[role="textbox"]',
    )
  )
}
export function createKeyboardDispatcher(context: CommandContext) {
  return (event: KeyboardEvent) => {
    if (
      event.defaultPrevented ||
      event.repeat ||
      event.isComposing ||
      event.altKey ||
      event.shiftKey
    )
      return
    const primary =
      context.platform === 'macos'
        ? event.metaKey && !event.ctrlKey
        : event.ctrlKey && !event.metaKey
    if (!primary) return
    const key = event.key.toLowerCase()
    const command = getAllCommands(context).find(
      (item) => item.shortcut?.toLowerCase() === `cmdorctrl+${key}`,
    )
    if (
      !command ||
      (editable(event.target) &&
        ![
          'open-preferences',
          'toggle-command-palette',
          'library-new',
          'library-save',
          'library-find',
          'library-replace',
        ].includes(command.id))
    )
      return
    event.preventDefault()
    void runCommand(command.id, context)
  }
}
export function useKeyboardShortcuts(context: CommandContext) {
  useEffect(() => {
    const dispatch = createKeyboardDispatcher(context)
    document.addEventListener('keydown', dispatch)
    return () => document.removeEventListener('keydown', dispatch)
  }, [context])
}
