import { useEffect } from 'react'
import { useCommandContext } from './use-command-context'
import { useKeyboardShortcuts } from './use-keyboard-shortcuts'
import { startNativeMenu } from '@/lib/menu'
export function useMainWindowEventListeners() {
  const context = useCommandContext()
  useKeyboardShortcuts(context)
  useEffect(() => {
    const menu = startNativeMenu(context)
    return () => {
      void menu.dispose()
    }
  }, [context])
}
