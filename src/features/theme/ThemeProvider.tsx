import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type PropsWithChildren,
} from 'react'

export type ThemePreference = 'system' | 'light' | 'dark'

interface ThemeContextValue {
  preference: ThemePreference
  setPreference: (value: ThemePreference) => void
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined)
const storageKey = 'texpio.theme'

function readPreference(): ThemePreference {
  try {
    const stored = window.localStorage.getItem(storageKey)
    if (stored === 'light' || stored === 'dark' || stored === 'system') return stored
  } catch {
    // Storage can be unavailable; the desktop remains usable with the OS theme.
  }
  return 'system'
}

export function ThemeProvider({ children }: PropsWithChildren) {
  const [preference, updatePreference] = useState<ThemePreference>(readPreference)

  const setPreference = useCallback((value: ThemePreference) => {
    try {
      window.localStorage.setItem(storageKey, value)
      updatePreference(value)
    } catch {
      updatePreference('system')
    }
  }, [])

  useEffect(() => {
    const query = window.matchMedia('(prefers-color-scheme: dark)')
    const apply = () => {
      document.documentElement.classList.toggle(
        'dark',
        preference === 'dark' || (preference === 'system' && query.matches),
      )
    }
    apply()
    if (preference === 'system') {
      query.addEventListener('change', apply)
      return () => query.removeEventListener('change', apply)
    }
  }, [preference])

  const value = useMemo(() => ({ preference, setPreference }), [preference, setPreference])
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
}

export function useTheme(): ThemeContextValue {
  const value = useContext(ThemeContext)
  if (!value) throw new Error('Theme controls require ThemeProvider')
  return value
}
