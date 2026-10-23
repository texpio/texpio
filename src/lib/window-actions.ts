import { getCurrentWindow } from '@tauri-apps/api/window'
import { requireDesktop } from './tauri-bindings'
export type WindowAction =
  | 'window-close'
  | 'window-minimize'
  | 'window-toggle-maximize'
  | 'window-fullscreen'
  | 'window-exit-fullscreen'
export async function executeWindowAction(action: WindowAction) {
  try {
    requireDesktop()
    const window = getCurrentWindow()
    switch (action) {
      case 'window-close':
        await window.close()
        break
      case 'window-minimize':
        await window.minimize()
        break
      case 'window-toggle-maximize':
        await window.toggleMaximize()
        break
      case 'window-fullscreen':
        await window.setFullscreen(true)
        break
      case 'window-exit-fullscreen':
        await window.setFullscreen(false)
        break
    }
    return { success: true as const }
  } catch {
    return { success: false as const, error: 'Window action is unavailable.' }
  }
}
