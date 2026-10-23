import { useState, useEffect } from 'react'
import { cn } from '@/lib/utils'

import { executeWindowAction } from '@/lib/window-actions'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { WindowsIcons } from './WindowControlIcons'

/**
 * Windows-style window control buttons (minimize, maximize/restore, close).
 * Positioned on the RIGHT side of the title bar, following Windows conventions.
 */
export function WindowsWindowControls() {
  const [isMaximized, setIsMaximized] = useState(false)

  // Initialize and sync maximized state with actual window state
  useEffect(() => {
    const appWindow = getCurrentWindow()
    let aborted = false

    // Query initial state
    appWindow
      .isMaximized()
      .then((value) => {
        if (!aborted) setIsMaximized(value)
      })
      .catch(() => {
        // Ignore errors - window may not be ready
      })

    // Subscribe to resize events to keep state in sync
    // (handles maximize/unmaximize via title bar double-click, etc.)
    let resolvedUnsub: (() => void) | null = null

    appWindow
      .onResized(async () => {
        try {
          const maximized = await appWindow.isMaximized()
          if (!aborted) setIsMaximized(maximized)
        } catch {
          // Ignore errors during cleanup
        }
      })
      .then(
        (unsub) => {
          // If already aborted, unsubscribe immediately
          if (aborted) {
            unsub()
          } else {
            resolvedUnsub = unsub
          }
        },
        () => {},
      )

    return () => {
      aborted = true
      // If unsubscribe has resolved, call it; otherwise it will be called in the then handler
      if (resolvedUnsub) {
        resolvedUnsub()
      }
    }
  }, [])

  const handleClose = async () => {
    await executeWindowAction('window-close')
  }

  const handleMinimize = async () => {
    await executeWindowAction('window-minimize')
  }

  const handleMaximizeToggle = async () => {
    await executeWindowAction('window-toggle-maximize')
  }

  // Base button styles for Windows controls
  const buttonClass = 'flex h-8 w-12 items-center justify-center transition-colors'

  return (
    <div className="flex">
      {/* Minimize */}
      <button
        type="button"
        onClick={handleMinimize}
        className={cn(buttonClass, 'hover:bg-foreground/10')}
        title="Minimize"
        aria-label="Minimize window"
      >
        <WindowsIcons.minimize />
      </button>

      {/* Maximize/Restore */}
      <button
        type="button"
        onClick={handleMaximizeToggle}
        className={cn(buttonClass, 'hover:bg-foreground/10')}
        title={isMaximized ? 'Restore' : 'Maximize'}
        aria-label={isMaximized ? 'Restore window' : 'Maximize window'}
      >
        {isMaximized ? <WindowsIcons.restore /> : <WindowsIcons.maximize />}
      </button>

      {/* Close */}
      <button
        type="button"
        onClick={handleClose}
        className={cn(buttonClass, 'hover:bg-destructive hover:text-destructive-foreground')}
        title="Close"
        aria-label="Close window"
      >
        <WindowsIcons.close />
      </button>
    </div>
  )
}
