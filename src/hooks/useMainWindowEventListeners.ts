import { useEffect } from 'react'
import { useCommandContext } from './use-command-context'
import { useKeyboardShortcuts } from './use-keyboard-shortcuts'
import { startNativeMenu } from '@/lib/menu'
import { listen } from '@tauri-apps/api/event'
import { isTauri } from '@tauri-apps/api/core'
import { useUIStore } from '@/store/ui-store'
export function useMainWindowEventListeners() {
  const context = useCommandContext()
  useKeyboardShortcuts(context)
  useEffect(() => {
    if (!isTauri()) return
    let cancelled = false
    let stop: (() => void) | undefined
    void listen<unknown>('quick-pane-submit', (event) => {
      if (cancelled || typeof event.payload !== 'object' || event.payload === null) return
      const text = (event.payload as { text?: unknown }).text
      if (typeof text === 'string' && text.trim().length > 0 && text.length <= 16384)
        useUIStore.getState().setLastQuickPaneEntry(text)
    }).then(
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
  }, [])
  useEffect(() => {
    const menu = startNativeMenu(context)
    return () => {
      void menu.dispose()
    }
  }, [context])
}
