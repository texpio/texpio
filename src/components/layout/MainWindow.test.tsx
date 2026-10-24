import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { isTauri } from '@tauri-apps/api/core'
import { beforeEach, expect, it, vi } from 'vitest'
import { ThemeProvider } from '@/components/ThemeProvider'
import { MainWindow } from './MainWindow'
import { useUIStore, normalizeLayout } from '@/store/ui-store'
import { mockSystemTheme } from '@/test/system-theme'
import { applyLanguage } from '@/i18n/config'
import { mockPanelGeometry } from '@/test/panel-layout'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))
// Observe the shell mount lifecycle without native data or input effects.
vi.mock('@/features/library/LibraryView', () => ({
  LibraryView: () => (
    <section>
      <h1>Snippet library</h1>
      <label>
        Test draft
        <input defaultValue="" />
      </label>
    </section>
  ),
}))
beforeEach(async () => {
  mockPanelGeometry()
  mockSystemTheme()
  vi.mocked(isTauri).mockReturnValue(false)
  useUIStore.setState({ layout: normalizeLayout(null), activeView: 'home' })
  await applyLanguage('en')
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  )
})
it('opens Preferences as a page and preserves the library editor across navigation', async () => {
  render(
    <QueryClientProvider client={new QueryClient()}>
      <ThemeProvider>
        <MainWindow />
      </ThemeProvider>
    </QueryClientProvider>,
  )
  const draft = screen.getByLabelText('Test draft')
  await userEvent.type(draft, 'Unfinished snippet')
  await userEvent.click(screen.getByRole('button', { name: 'Preferences' }))
  expect(screen.getByRole('heading', { name: 'Preferences' })).toBeVisible()
  expect(screen.queryByRole('dialog', { name: 'Preferences' })).not.toBeInTheDocument()
  expect(draft).not.toBeVisible()
  await userEvent.click(
    within(screen.getByRole('navigation', { name: 'Library navigation' })).getByRole('button', {
      name: 'All snippets',
    }),
  )
  expect(screen.getByLabelText('Test draft')).toBe(draft)
  expect(draft).toBeVisible()
  expect(draft).toHaveValue('Unfinished snippet')
  expect(screen.queryByRole('button', { name: 'Developer' })).not.toBeInTheDocument()
  expect(screen.queryByLabelText(/right sidebar/i)).not.toBeInTheDocument()
  expect(screen.queryByRole('complementary', { name: 'Workspace details' })).not.toBeInTheDocument()
})
