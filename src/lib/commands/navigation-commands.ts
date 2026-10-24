import { Home, Search, Settings, PanelLeft } from 'lucide-react'
import { useUIStore } from '@/store/ui-store'
import type { AppCommand } from './types'
import { showQuickPane } from '@/services/quick-pane'
export const navigationCommands: AppCommand[] = [
  {
    id: 'check-for-updates',
    labelKey: 'menu.checkForUpdates',
    group: 'navigation',
    isAvailable: (context) => context.platform !== 'unknown',
    execute: () => {
      const state = useUIStore.getState()
      state.setUpdateDialogOpen(true)
    },
  },
  {
    id: 'show-quick-pane',
    labelKey: 'quickPane.open',
    group: 'navigation',
    isAvailable: (context) => context.platform !== 'unknown',
    execute: showQuickPane,
  },
  {
    id: 'open-home',
    labelKey: 'shell.home',
    icon: Home,
    group: 'navigation',
    execute: () => useUIStore.getState().setActiveView('home'),
  },
  {
    id: 'open-preferences',
    labelKey: 'commands.openPreferences.label',
    descriptionKey: 'commands.openPreferences.description',
    icon: Settings,
    group: 'navigation',
    shortcut: 'CmdOrCtrl+,',
    execute: (context) => context.openPreferences(),
  },
  {
    id: 'toggle-command-palette',
    labelKey: 'commandPalette.title',
    icon: Search,
    group: 'navigation',
    shortcut: 'CmdOrCtrl+K',
    palette: false,
    execute: () => {
      const state = useUIStore.getState()
      state.setCommandPaletteOpen(!state.commandPaletteOpen)
    },
  },
  {
    id: 'toggle-left-sidebar',
    labelKey: 'menu.toggleLeftSidebar',
    icon: PanelLeft,
    group: 'navigation',
    shortcut: 'CmdOrCtrl+1',
    execute: () => {
      const state = useUIStore.getState()
      state.setSidebarVisible(!state.layout.leftVisible)
    },
  },
]
