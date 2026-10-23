import { useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { usePlatform } from './use-platform'
export function useSquareCornersEffect() {
  const platform = usePlatform()
  useEffect(() => {
    if (platform === 'unknown' || platform === 'macos') return
    let cancelled = false
    let stop: (() => void) | undefined
    const window = getCurrentWindow()
    const update = async () => {
      try {
        const fullscreen = await window.isFullscreen()
        if (!cancelled) document.documentElement.classList.toggle('square-corners', fullscreen)
      } catch {
        /* A window may be closing. */
      }
    }
    void update()
    void window
      .onResized(() => {
        void update()
      })
      .then(
        (unlisten) => {
          if (cancelled) unlisten()
          else stop = unlisten
        },
        () => {},
      )
    return () => {
      cancelled = true
      stop?.()
      document.documentElement.classList.remove('square-corners')
    }
  }, [platform])
}
