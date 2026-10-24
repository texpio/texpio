import { isTauri } from '@tauri-apps/api/core'
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { commands, requireDesktop, unwrapResult } from './tauri-bindings'
import { approveExit } from './exit-guard'
export type UpdateCheckResult =
  | { status: 'unconfigured' }
  | { status: 'no-update' }
  | { status: 'available'; version: string; currentVersion: string }
  | { status: 'error'; code: 'unavailable' }
export type UpdateActionResult = { success: true } | { success: false; code: 'unavailable' }
let configured = false
let activeUpdate: Update | null = null
let busy = false
export function getUpdateConfiguration() {
  return { configured }
}
export async function refreshUpdateConfiguration() {
  if (!isTauri()) {
    configured = false
    return getUpdateConfiguration()
  }
  const result = await commands.getUpdateConfiguration()
  configured = typeof result === 'object' && result !== null && result.configured === true
  return getUpdateConfiguration()
}
async function closeUpdate() {
  const update = activeUpdate
  activeUpdate = null
  try {
    await update?.close()
  } catch {
    /* Never surface native resource errors. */
  }
}
export async function cancelUpdate() {
  if (!busy) await closeUpdate()
}
export async function checkForUpdates(): Promise<UpdateCheckResult> {
  if (busy) return { status: 'error', code: 'unavailable' }
  busy = true
  try {
    await closeUpdate()
    if (!(await refreshUpdateConfiguration()).configured) return { status: 'unconfigured' }
    const update = await check()
    if (!update) return { status: 'no-update' }
    activeUpdate = update
    const version = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/
    if (
      update.version.length > 128 ||
      update.currentVersion.length > 128 ||
      !version.test(update.version) ||
      !version.test(update.currentVersion)
    ) {
      await closeUpdate()
      return { status: 'error', code: 'unavailable' }
    }
    return { status: 'available', version: update.version, currentVersion: update.currentVersion }
  } catch {
    await closeUpdate()
    configured = false
    return { status: 'error', code: 'unavailable' }
  } finally {
    busy = false
  }
}
export async function installUpdate(
  progress?: (event: DownloadEvent) => void,
): Promise<UpdateActionResult> {
  if (busy || !activeUpdate) return { success: false, code: 'unavailable' }
  busy = true
  try {
    await activeUpdate.download(progress)
    requireDesktop()
    if (!(await approveExit())) return { success: false, code: 'unavailable' }
    unwrapResult(await commands.prepareForRelaunch())
    await activeUpdate.install()
    return { success: true }
  } catch {
    return { success: false, code: 'unavailable' }
  } finally {
    await closeUpdate()
    busy = false
  }
}
export async function safeRelaunch(): Promise<UpdateActionResult> {
  if (busy) return { success: false, code: 'unavailable' }
  busy = true
  try {
    requireDesktop()
    if (!(await approveExit())) return { success: false, code: 'unavailable' }
    unwrapResult(await commands.prepareForRelaunch())
    await relaunch()
    return { success: true }
  } catch {
    return { success: false, code: 'unavailable' }
  } finally {
    busy = false
  }
}
