import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { PropsWithChildren } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { mockSystemTheme } from '@/test/system-theme'
import { ThemeProvider, useTheme } from './ThemeProvider'
import { appearanceCacheKey } from '@/services/preferences'
vi.mock('@tauri-apps/api/core', () => ({ isTauri: vi.fn() }))
let client: QueryClient
beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(false)
  client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
})
function Providers({ children }: PropsWithChildren) {
  return (
    <QueryClientProvider client={client}>
      <ThemeProvider>{children}</ThemeProvider>
    </QueryClientProvider>
  )
}

describe('theme preference', () => {
  it.each([null, 'neon'])('defaults to System for stored value %s', (value) => {
    mockSystemTheme(true)
    if (value) localStorage.setItem('texpio.theme', value)
    const { result } = renderHook(useTheme, { wrapper: Providers })
    expect(result.current.preference).toBe('system')
    expect(document.documentElement).toHaveClass('dark')
  })

  it.each(['light', 'dark'] as const)('restores the stored %s override', (value) => {
    mockSystemTheme(value !== 'dark')
    localStorage.setItem('texpio.theme', value)
    const { result } = renderHook(useTheme, { wrapper: Providers })
    expect(result.current.preference).toBe(value)
    expect(document.documentElement.classList.contains('dark')).toBe(value === 'dark')
  })

  it('follows live system changes only in System mode', async () => {
    const setSystemDark = mockSystemTheme(false)
    const { result } = renderHook(useTheme, { wrapper: Providers })
    act(() => setSystemDark(true))
    expect(document.documentElement).toHaveClass('dark')
    await act(async () => result.current.setPreference('light'))
    await waitFor(() => expect(result.current.preference).toBe('light'))
    expect(localStorage.getItem(appearanceCacheKey)).toBe('light')
    act(() => setSystemDark(false))
    act(() => setSystemDark(true))
    expect(document.documentElement).not.toHaveClass('dark')
    await act(async () => result.current.setPreference('dark'))
    await waitFor(() => expect(result.current.preference).toBe('dark'))
    expect(localStorage.getItem(appearanceCacheKey)).toBe('dark')
    act(() => setSystemDark(false))
    expect(document.documentElement).toHaveClass('dark')
    await act(async () => result.current.setPreference('system'))
    await waitFor(() => expect(result.current.preference).toBe('system'))
    expect(document.documentElement).not.toHaveClass('dark')
    expect(localStorage.getItem(appearanceCacheKey)).toBe('system')
  })

  it('survives denied preference reads with a System default', () => {
    mockSystemTheme(true)
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new DOMException('Blocked', 'SecurityError')
    })
    const { result } = renderHook(useTheme, { wrapper: Providers })
    expect(result.current.preference).toBe('system')
    expect(document.documentElement).toHaveClass('dark')
  })

  it('falls back to System if saving the preference is denied', () => {
    mockSystemTheme(false)
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new DOMException('Blocked', 'SecurityError')
    })
    const { result } = renderHook(useTheme, { wrapper: Providers })
    act(() => result.current.setPreference('dark'))
    expect(result.current.preference).toBe('system')
    expect(document.documentElement).not.toHaveClass('dark')
  })
})
