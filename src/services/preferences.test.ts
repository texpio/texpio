import { invoke, isTauri } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { loadPreferences, updatePreferences, appearanceCacheKey } from './preferences'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({
  emit: vi.fn().mockResolvedValue(undefined),
  listen: vi.fn().mockResolvedValue(() => {}),
}))
const defaults = { theme: 'system', language: null, quick_pane_shortcut: null }
beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(true)
})

describe('authoritative preferences', () => {
  it('migrates legacy theme through the atomic native command without overriding saved System', async () => {
    localStorage.setItem('texpio.theme', 'dark')
    vi.mocked(invoke).mockResolvedValue(defaults)
    await expect(loadPreferences()).resolves.toEqual(defaults)
    expect(invoke).toHaveBeenCalledWith('migrate_legacy_theme', { theme: 'dark' })
    expect(localStorage.getItem('texpio.theme')).toBe(null)
    expect(localStorage.getItem(appearanceCacheKey)).toBe('system')
    await loadPreferences()
    expect(invoke).toHaveBeenLastCalledWith('load_preferences')
  })
  it('keeps legacy data and cache on migration failure', async () => {
    localStorage.setItem('texpio.theme', 'dark')
    localStorage.setItem(appearanceCacheKey, 'light')
    vi.mocked(invoke).mockRejectedValue({ path: 'C:/private' })
    await expect(loadPreferences()).rejects.toThrow('Preferences are unavailable.')
    expect(localStorage.getItem('texpio.theme')).toBe('dark')
    expect(localStorage.getItem(appearanceCacheKey)).toBe('light')
  })
  it.each([null, {}, { ...defaults, theme: 'neon' }, { ...defaults, language: 'unsupported' }])(
    'refuses malformed preferences %j',
    async (value) => {
      vi.mocked(invoke).mockResolvedValue(value)
      await expect(loadPreferences()).rejects.toThrow('Preferences are unavailable.')
      expect(localStorage.getItem(appearanceCacheKey)).toBe(null)
    },
  )
  it('serializes rapid patches without dropping other settings and survives a failed save', async () => {
    let stored = { ...defaults, theme: 'light', language: 'fr' }
    let failSave = false
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === 'load_preferences') return stored
      if (command === 'save_preferences') {
        if (failSave) {
          failSave = false
          throw 'io'
        }
        stored = (args as { preferences: typeof stored }).preferences
        return null
      }
      throw new Error('unexpected command')
    })
    await Promise.all([updatePreferences({ theme: 'dark' }), updatePreferences({ language: 'ar' })])
    expect(stored).toEqual({ ...defaults, theme: 'dark', language: 'ar' })
    failSave = true
    await expect(updatePreferences({ theme: 'light' })).rejects.toThrow(
      'Preferences are unavailable.',
    )
    expect(localStorage.getItem(appearanceCacheKey)).toBe('dark')
    await expect(updatePreferences({ theme: 'system' })).resolves.toEqual({
      ...stored,
      theme: 'system',
    })
  })
  it('makes no native calls in browser preview', async () => {
    vi.mocked(isTauri).mockReturnValue(false)
    await loadPreferences()
    expect(invoke).not.toHaveBeenCalled()
  })
  it('returns committed shortcut state when appearance saves overlap a binding change', async () => {
    const committed = { ...defaults, theme: 'dark', quick_pane_shortcut: 'control+shift+KeyJ' }
    vi.mocked(invoke)
      .mockResolvedValueOnce(defaults)
      .mockResolvedValueOnce(null)
      .mockResolvedValue(committed)
    await expect(updatePreferences({ theme: 'dark' })).resolves.toEqual(committed)
  })
})
