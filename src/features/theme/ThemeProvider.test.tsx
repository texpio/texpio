import { act, renderHook } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { mockSystemTheme } from '@/test/system-theme'
import { ThemeProvider, useTheme } from './ThemeProvider'

describe('theme preference', () => {
  it.each([null, 'neon'])('defaults to System for stored value %s', (value) => {
    mockSystemTheme(true)
    if (value) localStorage.setItem('texpio.theme', value)
    const { result } = renderHook(useTheme, { wrapper: ThemeProvider })
    expect(result.current.preference).toBe('system')
    expect(document.documentElement).toHaveClass('dark')
  })

  it.each(['light', 'dark'] as const)('restores the stored %s override', (value) => {
    mockSystemTheme(value !== 'dark')
    localStorage.setItem('texpio.theme', value)
    const { result } = renderHook(useTheme, { wrapper: ThemeProvider })
    expect(result.current.preference).toBe(value)
    expect(document.documentElement.classList.contains('dark')).toBe(value === 'dark')
  })

  it('follows live system changes only in System mode', () => {
    const setSystemDark = mockSystemTheme(false)
    const { result } = renderHook(useTheme, { wrapper: ThemeProvider })
    act(() => setSystemDark(true))
    expect(document.documentElement).toHaveClass('dark')
    act(() => result.current.setPreference('light'))
    expect(localStorage.getItem('texpio.theme')).toBe('light')
    act(() => setSystemDark(false))
    act(() => setSystemDark(true))
    expect(document.documentElement).not.toHaveClass('dark')
    act(() => result.current.setPreference('dark'))
    expect(localStorage.getItem('texpio.theme')).toBe('dark')
    act(() => setSystemDark(false))
    expect(document.documentElement).toHaveClass('dark')
    act(() => result.current.setPreference('system'))
    expect(document.documentElement).not.toHaveClass('dark')
    expect(localStorage.getItem('texpio.theme')).toBe('system')
  })

  it('survives denied preference reads with a System default', () => {
    mockSystemTheme(true)
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new DOMException('Blocked', 'SecurityError')
    })
    const { result } = renderHook(useTheme, { wrapper: ThemeProvider })
    expect(result.current.preference).toBe('system')
    expect(document.documentElement).toHaveClass('dark')
  })

  it('falls back to System if saving the preference is denied', () => {
    mockSystemTheme(false)
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new DOMException('Blocked', 'SecurityError')
    })
    const { result } = renderHook(useTheme, { wrapper: ThemeProvider })
    act(() => result.current.setPreference('dark'))
    expect(result.current.preference).toBe('system')
    expect(document.documentElement).not.toHaveClass('dark')
  })
})
