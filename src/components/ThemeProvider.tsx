import { useEffect, useState, type PropsWithChildren } from 'react'
import { ThemeContext } from '@/lib/theme-context'
import {
  readAppearanceCache,
  usePreferences,
  useSavePreferences,
  usePreferencesSync,
} from '@/services/preferences'
import { applyLanguage, resolveLanguage } from '@/i18n/config'
import type { Theme } from '@/lib/tauri-bindings'

export function ThemeProvider({ children }: PropsWithChildren) {
  const [cached] = useState(readAppearanceCache)
  const query = usePreferences()
  const save = useSavePreferences()
  usePreferencesSync()
  const preference = query.data?.theme ?? cached
  const language = query.data?.language ?? null
  useEffect(() => {
    void applyLanguage(resolveLanguage(language, navigator.languages))
  }, [language])
  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const apply = () =>
      document.documentElement.classList.toggle(
        'dark',
        preference === 'dark' || (preference === 'system' && media.matches),
      )
    apply()
    media.addEventListener('change', apply)
    return () => media.removeEventListener('change', apply)
  }, [preference])
  const setPreference = (theme: Theme) => save.mutate({ theme })
  return (
    <ThemeContext
      value={{
        preference,
        theme: preference,
        setPreference,
        setTheme: setPreference,
        error: query.isError || save.isError ? 'Preferences are unavailable.' : null,
      }}
    >
      {children}
    </ThemeContext>
  )
}
