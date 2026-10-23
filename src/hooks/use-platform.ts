import { isTauri } from '@tauri-apps/api/core'
import { platform } from '@tauri-apps/plugin-os'
import { useMemo } from 'react'
export type AppPlatform = 'macos' | 'windows' | 'linux' | 'unknown'
export function getPlatform(): AppPlatform {
  if (!isTauri()) return 'unknown'
  try {
    const value = platform()
    return value === 'macos' || value === 'windows' || value === 'linux' ? value : 'unknown'
  } catch {
    return 'unknown'
  }
}
export function usePlatform() {
  return useMemo(() => getPlatform(), [])
}
export function useIsMacOS() {
  return usePlatform() === 'macos'
}
export function useIsWindows() {
  return usePlatform() === 'windows'
}
export function useIsLinux() {
  return usePlatform() === 'linux'
}
