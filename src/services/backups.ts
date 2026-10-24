import { open, save } from '@tauri-apps/plugin-dialog'
import { commands, requireDesktop, unwrapResult } from '@/lib/tauri-bindings'
import type { BackupFormat, ImportMode } from '@/lib/bindings'
export type { BackupFormat, ImportMode, BackupPreview } from '@/lib/bindings'
const formats = { json: ['json'], yaml: ['yaml', 'yml'], archive: ['zip'] }
export async function selectBackup(mode: ImportMode) {
  requireDesktop()
  const path = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'JSON, YAML, ZIP', extensions: ['json', 'yaml', 'yml', 'zip'] }],
  })
  if (!path || Array.isArray(path)) return null
  const extension = path.split('.').pop()?.toLowerCase()
  const format =
    extension === 'zip' ? 'archive' : extension === 'yaml' || extension === 'yml' ? 'yaml' : 'json'
  return unwrapResult(await commands.backupPreview(path, format, mode))
}
export async function exportBackup(format: BackupFormat) {
  requireDesktop()
  const path = await save({
    defaultPath: `texpio-backup.${formats[format][0]}`,
    filters: [
      { name: format === 'archive' ? 'ZIP' : format.toUpperCase(), extensions: formats[format] },
    ],
  })
  if (!path) return false
  unwrapResult(await commands.backupExport(path, format))
  return true
}
export const discardBackup = async (token: string) => {
  unwrapResult(await commands.backupDiscard(token))
}
export const importBackup = async (token: string) => {
  requireDesktop()
  return unwrapResult(await commands.backupImport(token))
}
export const recoveryStatus = async () => {
  requireDesktop()
  return unwrapResult(await commands.libraryRecoveryStatus())
}
export const recoverLibrary = async () => {
  requireDesktop()
  return unwrapResult(await commands.libraryRecover(true))
}
