import type { WindowAction } from '@/lib/window-actions'
import type { AppCommand } from './types'
const actions: Array<[WindowAction, string]> = [
  ['window-close', 'windowClose'],
  ['window-minimize', 'windowMinimize'],
  ['window-toggle-maximize', 'windowToggleMaximize'],
  ['window-fullscreen', 'windowFullscreen'],
  ['window-exit-fullscreen', 'windowExitFullscreen'],
]
export const windowCommands: AppCommand[] = actions.map(([id, key]) => ({
  id,
  labelKey: `commands.${key}.label`,
  descriptionKey: `commands.${key}.description`,
  group: 'window',
  isAvailable: (context) => context.platform !== 'unknown',
  execute: async (context) => {
    const result = await context.windowAction(id)
    if (!result.success) throw new Error('Window action unavailable')
  },
}))
