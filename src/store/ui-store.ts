import { create } from 'zustand'

export interface LayoutState {
  leftVisible: boolean
  rightVisible: boolean
  leftPercent: number
  rightPercent: number
}
const defaults: LayoutState = {
  leftVisible: true,
  rightVisible: true,
  leftPercent: 20,
  rightPercent: 20,
}
type StoragePort = Pick<Storage, 'getItem' | 'setItem'>
function width(value: unknown) {
  return typeof value === 'number' && Number.isFinite(value) && value >= 15 && value <= 40
    ? value
    : 20
}
export function normalizeLayout(value: unknown): LayoutState {
  const data = typeof value === 'object' && value !== null ? (value as Partial<LayoutState>) : {}
  let leftPercent = width(data.leftPercent),
    rightPercent = width(data.rightPercent)
  const total = leftPercent + rightPercent
  if (total > 70) {
    leftPercent *= 70 / total
    rightPercent = 70 - leftPercent
  }
  return {
    leftPercent,
    rightPercent,
    leftVisible: typeof data.leftVisible === 'boolean' ? data.leftVisible : true,
    rightVisible: typeof data.rightVisible === 'boolean' ? data.rightVisible : true,
  }
}
interface UIState {
  layout: LayoutState
  activeView: 'home' | 'developer'
  commandPaletteOpen: boolean
  preferencesOpen: boolean
  lastQuickPaneEntry: string | null
  setSidebarVisible: (side: 'left' | 'right', visible: boolean) => void
  setPanelSizes: (left: number, right: number) => void
  setActiveView: (view: 'home' | 'developer') => void
  setCommandPaletteOpen: (open: boolean) => void
  setPreferencesOpen: (open: boolean) => void
  setLastQuickPaneEntry: (text: string | null) => void
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
    commandPaletteOpen: false,
    preferencesOpen: false,
    lastQuickPaneEntry: null,
    setSidebarVisible: (side, visible) =>
      set((state) => ({
        layout: { ...state.layout, [side === 'left' ? 'leftVisible' : 'rightVisible']: visible },
      })),
    setPanelSizes: (left, right) =>
      set((state) => ({
        layout: normalizeLayout({
          ...state.layout,
          leftPercent: left === 0 ? state.layout.leftPercent : left,
          rightPercent: right === 0 ? state.layout.rightPercent : right,
        }),
      })),
    setActiveView: (activeView) => set({ activeView }),
    setCommandPaletteOpen: (commandPaletteOpen) => set({ commandPaletteOpen }),
    setPreferencesOpen: (preferencesOpen) => set({ preferencesOpen }),
    setLastQuickPaneEntry: (lastQuickPaneEntry) => set({ lastQuickPaneEntry }),
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
const storage: StoragePort = {
  getItem: (key) => localStorage.getItem(key),
  setItem: (key, value) => localStorage.setItem(key, value),
}
export const useUIStore = createUIStore(storage)
