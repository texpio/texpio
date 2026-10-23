import { commands, requireDesktop } from './tauri-bindings'
import type { AppInfo } from './bindings'
export type { AppInfo } from './bindings'

export async function getAppInfo(): Promise<AppInfo> {
  try {
    requireDesktop()
    const info: unknown = await commands.getAppInfo()
    if (
      typeof info !== 'object' ||
      info === null ||
      Array.isArray(info) ||
      !('version' in info) ||
      typeof info.version !== 'string' ||
      !info.version.trim() ||
      !('platform' in info) ||
      typeof info.platform !== 'string' ||
      !info.platform.trim()
    ) {
      throw new Error('Malformed response')
    }
    return { version: info.version, platform: info.platform }
  } catch {
    throw new Error('Native desktop information is unavailable.')
  }
}
