import { toast } from 'sonner'
import { commands, requireDesktop, unwrapResult } from './tauri-bindings'
import { logger } from './logger'
type NotificationType = 'success' | 'error' | 'info' | 'warning'
export async function notify(
  title: string,
  message?: string,
  options: { type?: NotificationType; native?: boolean; duration?: number } = {},
): Promise<{ delivered: 'native' | 'toast' | 'unavailable' }> {
  if (options.native) {
    try {
      requireDesktop()
      unwrapResult(await commands.sendNativeNotification(title, message ?? null))
      return { delivered: 'native' }
    } catch {
      await logger.warn('notificationFailed')
    }
  }
  try {
    toast[options.native ? 'error' : (options.type ?? 'info')](title, {
      description: message,
      duration: options.duration,
    })
    return { delivered: 'toast' }
  } catch {
    return { delivered: 'unavailable' }
  }
}
export const notifications = {
  success: (title: string, message?: string, native?: boolean) =>
    notify(title, message, { type: 'success', native }),
  error: (title: string, message?: string, native?: boolean) =>
    notify(title, message, { type: 'error', native }),
  info: (title: string, message?: string, native?: boolean) =>
    notify(title, message, { type: 'info', native }),
  warning: (title: string, message?: string, native?: boolean) =>
    notify(title, message, { type: 'warning', native }),
}
export const { success, error, info, warning } = notifications
