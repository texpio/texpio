import { isTauri } from '@tauri-apps/api/core'
import { emit, listen } from '@tauri-apps/api/event'
import { useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, unwrapResult, type AppPreferences, type Theme } from '@/lib/tauri-bindings'

export const appearanceCacheKey = 'texpio.appearance'
export const preferencesQueryKeys = {
  all: ['preferences'] as const,
  preferences: () => ['preferences'] as const,
}
function theme(value: unknown): value is Theme {
  return value === 'system' || value === 'light' || value === 'dark'
}
function read(key: string) {
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}
export function readAppearanceCache(): Theme {
  const value = read(appearanceCacheKey) ?? read('texpio.theme')
  return theme(value) ? value : 'system'
}
function cache(value: Theme) {
  try {
    localStorage.setItem(appearanceCacheKey, value)
  } catch {
    /* Native preferences remain authoritative. */
  }
}
function parse(value: unknown): AppPreferences {
  if (typeof value !== 'object' || value === null || Array.isArray(value))
    throw new Error('Preferences are unavailable.')
  const data = value as Record<string, unknown>
  if (
    !theme(data.theme) ||
    !(data.language === null || ['en', 'fr', 'ar'].includes(data.language as string)) ||
    !(
      data.quick_pane_shortcut === null ||
      (typeof data.quick_pane_shortcut === 'string' &&
        /^[\x20-\x7e]{1,128}$/.test(data.quick_pane_shortcut))
    )
  )
    throw new Error('Preferences are unavailable.')
  return {
    theme: data.theme,
    language: data.language as AppPreferences['language'],
    quick_pane_shortcut: data.quick_pane_shortcut as string | null,
  }
}
export async function loadPreferences(): Promise<AppPreferences> {
  if (!isTauri()) return { theme: readAppearanceCache(), language: null, quick_pane_shortcut: null }
  try {
    const legacy = read('texpio.theme')
    const value = parse(
      unwrapResult(
        await (theme(legacy) ? commands.migrateLegacyTheme(legacy) : commands.loadPreferences()),
      ),
    )
    cache(value.theme)
    if (theme(legacy)) {
      try {
        localStorage.removeItem('texpio.theme')
      } catch {
        /* A later migration safely rechecks native state. */
      }
    }
    return value
  } catch {
    throw new Error('Preferences are unavailable.')
  }
}
let pending: Promise<unknown> = Promise.resolve()
export function updatePreferences(patch: Partial<AppPreferences>): Promise<AppPreferences> {
  const operation = pending.then(async () => {
    try {
      let preferences = parse({ ...(await loadPreferences()), ...patch })
      if (isTauri()) {
        unwrapResult(await commands.savePreferences(preferences))
        preferences = await loadPreferences()
      } else localStorage.setItem(appearanceCacheKey, preferences.theme)
      cache(preferences.theme)
      if (isTauri()) {
        try {
          await emit('preferences-changed')
        } catch {
          /* A refresh can still load saved preferences. */
        }
      }
      return preferences
    } catch {
      throw new Error('Preferences are unavailable.')
    }
  })
  pending = operation.catch(() => {})
  return operation
}
export function usePreferences() {
  return useQuery({ queryKey: preferencesQueryKeys.all, queryFn: loadPreferences, retry: false })
}
export function useSavePreferences() {
  const client = useQueryClient()
  return useMutation({
    mutationFn: updatePreferences,
    onSuccess: (preferences) => client.setQueryData(preferencesQueryKeys.all, preferences),
  })
}
export function usePreferencesSync() {
  const client = useQueryClient()
  useEffect(() => {
    const refresh = () => {
      void client.invalidateQueries({ queryKey: preferencesQueryKeys.all })
    }
    if (!isTauri()) {
      const changed = (event: StorageEvent) => {
        if (event.key === appearanceCacheKey) refresh()
      }
      window.addEventListener('storage', changed)
      return () => window.removeEventListener('storage', changed)
    }
    let cancelled = false
    let stop: (() => void) | undefined
    void listen('preferences-changed', refresh).then(
      (unlisten) => {
        if (cancelled) unlisten()
        else stop = unlisten
      },
      () => {},
    )
    return () => {
      cancelled = true
      stop?.()
    }
  }, [client])
}
