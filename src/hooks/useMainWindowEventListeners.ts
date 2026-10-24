import { useEffect } from 'react'
import { useCommandContext } from './use-command-context'
import { useKeyboardShortcuts } from './use-keyboard-shortcuts'
import { startNativeMenu } from '@/lib/menu'
import { isTauri } from '@tauri-apps/api/core'
import { createEditContextMenu } from '@/lib/context-menu'
import { cleanupOldRecoveryFiles } from '@/lib/recovery'
import { logger } from '@/lib/logger'
export function useMainWindowEventListeners() {
  const context = useCommandContext()
  useKeyboardShortcuts(context)
  useEffect(() => {
    if (!isTauri()) return
    void cleanupOldRecoveryFiles().catch(() => {
      void logger.warn('recoveryFailed')
    })
    const menu = createEditContextMenu()
    const popup = (event: MouseEvent) => {
      if (
        event.target instanceof Element &&
        event.target.closest('input,textarea,[contenteditable="true"]')
      ) {
        event.preventDefault()
        void menu.show()
      }
    }
    document.addEventListener('contextmenu', popup)
    return () => {
      document.removeEventListener('contextmenu', popup)
      void menu.dispose()
    }
  }, [])
  useEffect(() => {
    const menu = startNativeMenu(context)
    return () => {
      void menu.dispose()
    }
  }, [context])
}
