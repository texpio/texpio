import { isTauri } from '@tauri-apps/api/core'
import { platform } from '@tauri-apps/plugin-os'
import { expect, it, vi } from 'vitest'
import { getPlatform } from './use-platform'
vi.mock('@tauri-apps/api/core', () => ({ isTauri: vi.fn() }))
vi.mock('@tauri-apps/plugin-os', () => ({ platform: vi.fn() }))
it('keeps browser and failed detection explicitly unknown', () => {
  vi.mocked(isTauri).mockReturnValue(false)
  expect(getPlatform()).toBe('unknown')
  expect(platform).not.toHaveBeenCalled()
  vi.mocked(isTauri).mockReturnValue(true)
  vi.mocked(platform).mockImplementation(() => {
    throw new Error('private')
  })
  expect(getPlatform()).toBe('unknown')
})
it('recognizes each desktop without inventing support for other platforms', () => {
  vi.mocked(isTauri).mockReturnValue(true)
  for (const value of ['windows', 'macos', 'linux'] as const) {
    vi.mocked(platform).mockReturnValue(value)
    expect(getPlatform()).toBe(value)
  }
  vi.mocked(platform).mockReturnValue('android')
  expect(getPlatform()).toBe('unknown')
})
