import { open } from '@tauri-apps/plugin-dialog'
import { commands, requireDesktop, unwrapResult } from '@/lib/tauri-bindings'
import type { Attachment, FileSettings } from '@/lib/bindings'
import { LibraryCommandError } from './library'

export const attachmentKeys = {
  all: ['attachments'] as const,
  snippet: (id: string, revision: number) => ['attachments', id, revision] as const,
}
async function call<T>(operation: () => Promise<T>): Promise<T> {
  try {
    requireDesktop()
    return await operation()
  } catch {
    throw new LibraryCommandError('unavailable')
  }
}
export const listAttachments = (id: string): Promise<Attachment[]> =>
  call(async () => unwrapResult(await commands.attachmentList(id)))
export const loadFileSettings = (id: string): Promise<FileSettings> =>
  call(async () => unwrapResult(await commands.attachmentSettings(id)))
export const removeAttachment = (id: string, revision: number, attachment: string) =>
  call(async () => unwrapResult(await commands.attachmentRemove(id, revision, attachment)))
export const reorderAttachments = (id: string, revision: number, order: string[]) =>
  call(async () => unwrapResult(await commands.attachmentReorder(id, revision, order)))
export const saveFileSettings = (id: string, revision: number, input: FileSettings) =>
  call(async () => unwrapResult(await commands.attachmentSaveSettings(id, revision, input)))
export const addSelectedPaths = (id: string, revision: number, paths: string[]) =>
  call(async () => unwrapResult(await commands.attachmentAdd(id, revision, paths)))
export async function selectAttachments(id: string, revision: number) {
  return call(async () => {
    const paths = await open({ multiple: true, directory: false })
    if (!paths) return
    await addSelectedPaths(id, revision, Array.isArray(paths) ? paths : [paths])
  })
}
export async function addDroppedFiles(id: string, revision: number, files: File[]) {
  return call(async () => {
    for (const file of files) {
      if (file.size > 6 * 1024 * 1024) throw new LibraryCommandError('tooLarge')
      const data = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader()
        reader.onerror = () => reject(new LibraryCommandError('unavailable'))
        reader.onload = () =>
          typeof reader.result === 'string'
            ? resolve(reader.result.slice(reader.result.indexOf(',') + 1))
            : reject(new LibraryCommandError('invalidData'))
        reader.readAsDataURL(file)
      })
      unwrapResult(await commands.attachmentAddData(id, revision, { name: file.name, data }))
    }
  })
}
