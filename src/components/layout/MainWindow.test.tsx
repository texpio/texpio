import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { isTauri } from '@tauri-apps/api/core'
import { beforeEach, expect, it, vi } from 'vitest'
import { ThemeProvider } from '@/components/ThemeProvider'
import { MainWindow } from './MainWindow'
import { useUIStore, normalizeLayout } from '@/store/ui-store'
import { mockSystemTheme } from '@/test/system-theme'
import { applyLanguage } from '@/i18n/config'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))
vi.mock('@/features/foundation/FoundationScreen', () => ({
  FoundationScreen: () => <section aria-label="Stopped probe" />,
}))
beforeEach(async () => {
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
it('opens Developer without mounting or arming the probe from Home', async () => {
  const client = new QueryClient()
  render(
    <QueryClientProvider client={client}>
      <ThemeProvider>
        <MainWindow />
      </ThemeProvider>
    </QueryClientProvider>,
  )
  expect(screen.getByRole('heading', { name: 'Snippet library' })).toBeInTheDocument()
  expect(screen.queryByLabelText('Stopped probe')).not.toBeInTheDocument()
  await userEvent.click(screen.getByRole('button', { name: 'Developer' }))
  expect(screen.getByLabelText('Stopped probe')).toBeInTheDocument()
  expect(screen.queryByRole('button', { name: /Minimize window/i })).not.toBeInTheDocument()
})
