import { useUIStore } from '@/store/ui-store'
import type { AppCommand } from './types'
export type LibraryAction =
  'new' | 'save' | 'find' | 'replace' | 'search' | 'new-folder' | 'manage-folder'
let active: ((action: LibraryAction) => void) | null = null
export function bindLibraryActions(handler: (action: LibraryAction) => void) {
  active = handler
  return () => {
    if (active === handler) active = null
  }
}
const entries = [
  ['new', 'N'],
  ['save', 'S'],
  ['find', 'F'],
  ['replace', 'H'],
  ['search', null],
  ['new-folder', null],
  ['manage-folder', null],
] as const
export const libraryCommands: AppCommand[] = entries.map(([action, key]) => ({
  id: `library-${action}`,
  labelKey: `editor.command.${action}`,
  group: 'library',
  ...(key ? { shortcut: `CmdOrCtrl+${key}` } : {}),
  isAvailable: () => !!active && useUIStore.getState().activeView === 'home',
  execute: () => {
    active?.(action)
  },
}))
