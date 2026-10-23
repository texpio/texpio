import { useEffect, useState, type HTMLProps } from 'react'
import { cn } from '@/lib/utils'
import { MacOSIcons } from './WindowControlIcons'

import { useCommandContext } from '@/hooks/use-command-context'
import { runCommand } from '@/lib/commands'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface MacOSWindowControlsProps extends HTMLProps<HTMLDivElement> {
  className?: string
}

export function MacOSWindowControls({ className, ...props }: MacOSWindowControlsProps) {
  const context = useCommandContext()
  const [isAltKeyPressed, setIsAltKeyPressed] = useState(false)
  const [isHovering, setIsHovering] = useState(false)
  const [isWindowFocused, setIsWindowFocused] = useState(true)

  const LastIcon = isAltKeyPressed ? MacOSIcons.maximize : MacOSIcons.fullscreen
  const key = 'Alt'

  const handleMouseEnter = () => {
    setIsHovering(true)
  }
  const handleMouseLeave = () => {
    setIsHovering(false)
  }

  const handleAltKeyDown = (e: KeyboardEvent) => {
    if (e.key === key) {
      setIsAltKeyPressed(true)
    }
  }
  const handleAltKeyUp = (e: KeyboardEvent) => {
    if (e.key === key) {
      setIsAltKeyPressed(false)
    }
  }

  useEffect(() => {
    let cancelled = false
    // Attach event listeners when the component mounts
    window.addEventListener('keydown', handleAltKeyDown)
    window.addEventListener('keyup', handleAltKeyUp)

    // Listen for window focus/blur events
    const handleWindowFocus = () => setIsWindowFocused(true)
    const handleWindowBlur = () => {
      setIsWindowFocused(false)
      setIsAltKeyPressed(false)
    }

    window.addEventListener('focus', handleWindowFocus)
    window.addEventListener('blur', handleWindowBlur)

    // Also listen for Tauri window focus events if available
    const setupTauriFocusListener = async () => {
      try {
        const appWindow = getCurrentWindow()
        const unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
          if (!cancelled) setIsWindowFocused(focused)
        })
        return unlistenFocus
      } catch {
        // Fallback to window focus events if Tauri events aren't available
        return null
      }
    }

    let tauriUnlisten: (() => void) | null = null
    setupTauriFocusListener().then((unlisten) => {
      if (cancelled) unlisten?.()
      else tauriUnlisten = unlisten
    })

    // Cleanup event listeners
    return () => {
      cancelled = true
      window.removeEventListener('keydown', handleAltKeyDown)
      window.removeEventListener('keyup', handleAltKeyUp)
      window.removeEventListener('focus', handleWindowFocus)
      window.removeEventListener('blur', handleWindowBlur)
      if (tauriUnlisten) {
        tauriUnlisten()
      }
    }
  }, [])

  const handleClose = async () => {
    await runCommand('window-close', context)
  }

  const handleMinimize = async () => {
    await runCommand('window-minimize', context)
  }

  const handleMaximizeOrFullscreen = async () => {
    try {
      const appWindow = getCurrentWindow()
      const isFullscreen = await appWindow.isFullscreen()

      if (isFullscreen) {
        // If currently fullscreen, exit fullscreen regardless of Alt key
        await runCommand('window-exit-fullscreen', context)
      } else if (isAltKeyPressed) {
        // Alt + click: toggle maximize/restore
        await runCommand('window-toggle-maximize', context)
      } else {
        // Normal click: enter fullscreen
        await runCommand('window-fullscreen', context)
      }
    } catch {
      /* Query failure leaves the window unchanged. */
    }
  }

  return (
    <div
      className={cn(
        'flex items-center gap-2 px-3 text-black active:text-black dark:text-black',
        className,
      )}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      {...props}
    >
      <button
        type="button"
        onClick={handleClose}
        aria-label="Close window"
        className={cn(
          'group flex h-3 w-3 cursor-default items-center justify-center rounded-full border text-center text-black/60 hover:bg-[#ff544d] hover:border-black/[.12] active:bg-[#bf403a] active:text-black/60 dark:border-none',
          isWindowFocused ? 'border-black/[.12] bg-[#ff544d]' : 'border-gray-400/20 bg-gray-400',
        )}
      >
        <div className="flex h-3 w-3 items-center justify-center">
          {isHovering && <MacOSIcons.close className="h-[6px] w-[6px] opacity-60" />}
        </div>
      </button>
      <button
        type="button"
        onClick={handleMinimize}
        aria-label="Minimize window"
        className={cn(
          'group flex h-3 w-3 cursor-default items-center justify-center rounded-full border text-center text-black/60 hover:bg-[#ffbd2e] hover:border-black/[.12] active:bg-[#bf9122] active:text-black/60 dark:border-none',
          isWindowFocused ? 'border-black/[.12] bg-[#ffbd2e]' : 'border-gray-400/20 bg-gray-400',
        )}
      >
        <div className="flex h-3 w-3 items-center justify-center">
          {isHovering && <MacOSIcons.minimize className="h-[2px] w-[6px] opacity-60" />}
        </div>
      </button>
      <button
        type="button"
        onClick={handleMaximizeOrFullscreen}
        aria-label={isAltKeyPressed ? 'Maximize window' : 'Enter fullscreen'}
        className={cn(
          'group flex h-3 w-3 cursor-default items-center justify-center rounded-full border text-center text-black/60 hover:bg-[#28c93f] hover:border-black/[.12] active:bg-[#1e9930] active:text-black/60 dark:border-none',
          isWindowFocused ? 'border-black/[.12] bg-[#28c93f]' : 'border-gray-400/20 bg-gray-400',
        )}
      >
        <div className="flex h-3 w-3 items-center justify-center">
          {isHovering && <LastIcon className="size-[5px] opacity-60" />}
        </div>
      </button>
    </div>
  )
}
