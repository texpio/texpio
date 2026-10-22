import { invoke, isTauri } from '@tauri-apps/api/core'

export interface AppInfo {
  version: string
  platform: string
}

export async function getAppInfo(): Promise<AppInfo> {
  try {
    if (!isTauri()) throw new Error('Browser preview')
    const info = await invoke<unknown>('get_app_info')
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
