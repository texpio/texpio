import { act, renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { PropsWithChildren } from 'react'
import { beforeEach, expect, it, vi } from 'vitest'
import { ThemeProvider } from './ThemeProvider'
import { useTheme } from '@/hooks/use-theme'
import { mockSystemTheme } from '@/test/system-theme'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({
  emit: vi.fn().mockResolvedValue(undefined),
  listen: vi.fn(),
}))
let client: QueryClient
beforeEach(() => {
  client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  mockSystemTheme(false)
  vi.mocked(isTauri).mockReturnValue(true)
  vi.mocked(listen).mockResolvedValue(() => {})
})
function Providers({ children }: PropsWithChildren) {
  return (
    <QueryClientProvider client={client}>
      <ThemeProvider>{children}</ThemeProvider>
    </QueryClientProvider>
  )
}
it('resynchronizes theme when another window changes persisted preferences', async () => {
  let stored = { theme: 'light', language: null, quick_pane_shortcut: null }
  vi.mocked(invoke).mockImplementation(async () => stored)
  let changed = () => {}
  vi.mocked(listen).mockImplementation(async (_event, callback) => {
    changed = () => callback({ event: 'preferences-changed', id: 1, payload: undefined })
    return () => {}
  })
  const { result } = renderHook(useTheme, { wrapper: Providers })
  await waitFor(() => expect(result.current.preference).toBe('light'))
  stored = { ...stored, theme: 'dark' }
  act(changed)
  await waitFor(() => expect(document.documentElement).toHaveClass('dark'))
  expect(result.current.preference).toBe('dark')
})
it('cleans up listener registration that completes after unmount', async () => {
  vi.mocked(invoke).mockResolvedValue({
    theme: 'system',
    language: null,
    quick_pane_shortcut: null,
  })
  let finish!: (unlisten: () => void) => void
  vi.mocked(listen).mockReturnValue(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  const stop = vi.fn()
  const hook = renderHook(useTheme, { wrapper: Providers })
  hook.unmount()
  await act(async () => finish(stop))
  expect(stop).toHaveBeenCalledOnce()
})
