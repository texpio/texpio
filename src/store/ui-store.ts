import { create } from 'zustand'

export interface LayoutState {
  leftVisible: boolean
  leftPercent: number
  listPercent: number
}
export type LibraryCollection = 'active' | 'favorites' | 'trash'
export type PreferencesSection = 'general' | 'expansion' | 'shortcuts' | 'data'
type StoragePort = Pick<Storage, 'getItem' | 'setItem'>
const defaults: LayoutState = { leftVisible: true, leftPercent: 22, listPercent: 36 }
function width(value: unknown, min: number, max: number, fallback: number) {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max
    ? value
    : fallback
}
export function normalizeLayout(value: unknown): LayoutState {
  const data = typeof value === 'object' && value !== null ? (value as Partial<LayoutState>) : {}
  return {
    leftVisible: typeof data.leftVisible === 'boolean' ? data.leftVisible : true,
    leftPercent: width(data.leftPercent, 18, 30, defaults.leftPercent),
    listPercent: width(data.listPercent, 25, 50, defaults.listPercent),
  }
}
interface UIState {
  layout: LayoutState
  activeView: 'home' | 'preferences'
  preferencesSection: PreferencesSection
  expansionStartBlocked: boolean
  setExpansionStartBlocked: (blocked: boolean) => void
  libraryCollection: LibraryCollection
  libraryFolder: string
  commandPaletteOpen: boolean
  updateDialogOpen: boolean
  permissionPromptDismissed: boolean
  dismissPermissionPrompt: () => void
  setSidebarVisible: (visible: boolean) => void
  setSidebarWidth: (percent: number) => void
  setSnippetListWidth: (percent: number) => void
  setActiveView: (view: 'home' | 'preferences') => void
  setPreferencesSection: (section: PreferencesSection) => void
  setLibraryFilter: (collection: LibraryCollection, folder?: string) => void
  setCommandPaletteOpen: (open: boolean) => void
  setUpdateDialogOpen: (open: boolean) => void
}
export function createUIStore(storage?: StoragePort) {
  let layout = defaults
  try {
    layout = normalizeLayout(JSON.parse(storage?.getItem('texpio.layout') ?? 'null'))
  } catch {
    /* Invalid storage falls back to a usable layout. */
  }
  const store = create<UIState>()((set) => ({
    layout,
    activeView: 'home',
    preferencesSection: 'general',
    expansionStartBlocked: true,
    setExpansionStartBlocked: (expansionStartBlocked) => set({ expansionStartBlocked }),
    libraryCollection: 'active',
    libraryFolder: 'all',
    commandPaletteOpen: false,
    updateDialogOpen: false,
    permissionPromptDismissed: false,
    dismissPermissionPrompt: () => set({ permissionPromptDismissed: true }),
    setSidebarVisible: (leftVisible) =>
      set((state) =>
        state.layout.leftVisible === leftVisible
          ? state
          : { layout: { ...state.layout, leftVisible } },
      ),
    setSidebarWidth: (percent) =>
      set((state) => {
        const leftPercent =
          percent === 0 ? state.layout.leftPercent : width(percent, 18, 30, defaults.leftPercent)
        return state.layout.leftPercent === leftPercent
          ? state
          : { layout: { ...state.layout, leftPercent } }
      }),
    setSnippetListWidth: (percent) =>
      set((state) => {
        const listPercent = width(percent, 25, 50, defaults.listPercent)
        return state.layout.listPercent === listPercent
          ? state
          : { layout: { ...state.layout, listPercent } }
      }),
    setActiveView: (activeView) => set({ activeView }),
    setPreferencesSection: (preferencesSection) => set({ preferencesSection }),
    setLibraryFilter: (libraryCollection, libraryFolder = 'all') =>
      set({ activeView: 'home', libraryCollection, libraryFolder }),
    setCommandPaletteOpen: (commandPaletteOpen) => set({ commandPaletteOpen }),
    setUpdateDialogOpen: (updateDialogOpen) => set({ updateDialogOpen }),
  }))
  store.subscribe((state, previous) => {
    if (state.layout === previous.layout) return
    try {
      storage?.setItem('texpio.layout', JSON.stringify(state.layout))
    } catch {
      /* Layout remains usable when storage is unavailable. */
    }
  })
  return store
}
export const useUIStore = createUIStore({
  getItem: (key) => localStorage.getItem(key),
  setItem: (key, value) => localStorage.setItem(key, value),
})
