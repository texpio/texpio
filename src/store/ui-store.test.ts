import { describe, expect, it } from 'vitest'
import { createUIStore, normalizeLayout } from './ui-store'

describe('persisted shell layout', () => {
  it('migrates old right-sidebar layout data into bounded workspace panes', () => {
    expect(
      normalizeLayout({
        leftVisible: false,
        leftPercent: 25,
        rightVisible: true,
        rightPercent: 40,
      }),
    ).toEqual({ leftVisible: false, leftPercent: 25, listPercent: 36 })
    expect(normalizeLayout({ leftPercent: Infinity, listPercent: -10 })).toEqual({
      leftVisible: true,
      leftPercent: 22,
      listPercent: 36,
    })
    expect(normalizeLayout({ leftPercent: 40, listPercent: 80 })).toEqual({
      leftVisible: true,
      leftPercent: 22,
      listPercent: 36,
    })
  })
  it('persists only layout while keeping navigation and filters transient', () => {
    const store = createUIStore(localStorage)
    store.getState().setSidebarWidth(25)
    store.getState().setSnippetListWidth(40)
    store.getState().setSidebarVisible(false)
    store.getState().setSidebarWidth(0)
    store.getState().setCommandPaletteOpen(true)
    store.getState().setLibraryFilter('favorites', 'synthetic-private-folder')
    store.getState().setActiveView('preferences')
    store.getState().setPreferencesSection('data')
    store.getState().setExpansionStartBlocked(false)
    const restored = createUIStore(localStorage).getState()
    expect(restored.layout).toEqual({ leftVisible: false, leftPercent: 25, listPercent: 40 })
    expect(restored.commandPaletteOpen).toBe(false)
    expect(restored.activeView).toBe('home')
    expect(restored.preferencesSection).toBe('general')
    expect(restored.expansionStartBlocked).toBe(true)
    expect(restored.libraryCollection).toBe('active')
    expect(restored.libraryFolder).toBe('all')
    expect(JSON.parse(localStorage.getItem('texpio.layout')!)).toEqual(restored.layout)
    restored.setSidebarVisible(true)
    expect(createUIStore(localStorage).getState().layout.leftVisible).toBe(true)
  })
  it('survives corrupt and denied storage', () => {
    localStorage.setItem('texpio.layout', '{broken')
    expect(createUIStore(localStorage).getState().layout.leftPercent).toBe(22)
    const denied = {
      getItem() {
        throw new Error('denied')
      },
      setItem() {
        throw new Error('denied')
      },
    }
    const store = createUIStore(denied)
    store.getState().setSidebarVisible(false)
    expect(store.getState().layout.leftVisible).toBe(false)
  })
})
