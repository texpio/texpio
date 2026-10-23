import { createContext } from 'react'
import type { Theme } from './tauri-bindings'
export type ThemePreference = Theme
export interface ThemeContextValue {
  preference: Theme
  theme: Theme
  setPreference: (value: Theme) => void
  setTheme: (value: Theme) => void
  error: string | null
}
export const ThemeContext = createContext<ThemeContextValue | undefined>(undefined)
