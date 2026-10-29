import { useEffect } from 'react'
import { useCommandContext } from './use-command-context'
import { useKeyboardShortcuts } from './use-keyboard-shortcuts'
import { startNativeMenu } from '@/lib/menu'
import { isTauri } from '@tauri-apps/api/core'
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
  }, [])
  useEffect(() => {
    const menu = startNativeMenu(context)
    return () => {
      void menu.dispose()
    }
  }, [context])
}
