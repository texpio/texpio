import { useContext } from 'react'
import { ThemeContext } from '@/lib/theme-context'
export function useTheme() {
  const value = useContext(ThemeContext)
  if (!value) throw new Error('Theme controls require ThemeProvider')
  return value
}
