import type { LucideIcon } from 'lucide-react'
import type { AppPlatform } from '@/hooks/use-platform'
import type { executeWindowAction } from '@/lib/window-actions'
export interface CommandContext {
  openPreferences: () => void
  showToast: (message: string, type?: 'success' | 'error' | 'info') => void
  platform: AppPlatform
  windowAction: typeof executeWindowAction
}
export type CommandResult =
  { success: true } | { success: false; code: 'unknownCommand' | 'unavailable' | 'failed' }
export interface AppCommand {
  id: string
  labelKey: string
  descriptionKey?: string
  icon?: LucideIcon
  group?: string
  keywords?: string[]
  shortcut?: string
  palette?: boolean
  execute: (context: CommandContext) => void | Promise<void>
  isAvailable?: (context: CommandContext) => boolean
}
