import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { isTauri } from '@tauri-apps/api/core'
import { CommandPalette } from './CommandPalette'
import { useUIStore } from '@/store/ui-store'
import { applyLanguage } from '@/i18n/config'
vi.mock('@tauri-apps/api/core', () => ({ isTauri: vi.fn() }))
beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(false)
  useUIStore.setState({ commandPaletteOpen: true, activeView: 'home' })
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  )
  Element.prototype.scrollIntoView = vi.fn()
})
it('finds translated labels and routes selection through the registered action', async () => {
  await applyLanguage('fr')
  render(<CommandPalette />)
  const dialog = screen.getByRole('dialog', { name: 'Palette de commandes' })
  expect(dialog).toBeInTheDocument()
  await userEvent.type(screen.getByRole('combobox'), 'param')
  expect(screen.getByRole('option', { name: /paramètres/i })).toBeInTheDocument()
  await userEvent.click(screen.getByRole('option', { name: /paramètres/i }))
  expect(useUIStore.getState().activeView).toBe('preferences')
  expect(useUIStore.getState().commandPaletteOpen).toBe(false)
})
