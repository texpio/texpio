import { describe, expect, it } from 'vitest'
import { createUIStore, normalizeLayout } from './ui-store'

describe('persisted shell layout', () => {
  it('normalizes invalid restored widths', () => {
    const layout = normalizeLayout({ leftPercent: Infinity, rightPercent: -10 })
    expect(layout.leftPercent).toBe(20)
    expect(layout.rightPercent).toBe(20)
    const crowded = normalizeLayout({ leftPercent: 40, rightPercent: 40 })
    expect(crowded.leftPercent + crowded.rightPercent).toBeLessThanOrEqual(70)
  })
  it('restores layout without persisting modal, view or entry data', () => {
    const store = createUIStore(localStorage)
    store.getState().setPanelSizes(25, 30)
    store.getState().setSidebarVisible('left', false)
    store.getState().setPanelSizes(0, 30)
    store.getState().setCommandPaletteOpen(true)
    store.getState().setPreferencesOpen(true)
    store.getState().setLastQuickPaneEntry('synthetic private entry')
    store.getState().setActiveView('developer')
    const restored = createUIStore(localStorage).getState()
    expect(restored.layout).toEqual({
      leftVisible: false,
      rightVisible: true,
      leftPercent: 25,
      rightPercent: 30,
    })
    expect(restored.commandPaletteOpen).toBe(false)
    expect(restored.preferencesOpen).toBe(false)
    expect(restored.lastQuickPaneEntry).toBe(null)
    expect(restored.activeView).toBe('home')
    expect(localStorage.getItem('texpio.layout')).not.toContain('synthetic')
    restored.setSidebarVisible('left', true)
    expect(createUIStore(localStorage).getState().layout.leftVisible).toBe(true)
  })
  it('survives corrupt and denied storage', () => {
    localStorage.setItem('texpio.layout', '{broken')
    expect(createUIStore(localStorage).getState().layout.leftPercent).toBe(20)
    const denied = {
      getItem() {
        throw new Error('denied')
      },
      setItem() {
        throw new Error('denied')
      },
    }
    const store = createUIStore(denied)
    store.getState().setSidebarVisible('right', false)
    expect(store.getState().layout.rightVisible).toBe(false)
  })
})
