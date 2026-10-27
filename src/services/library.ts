import { useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readTextFile, stat, writeTextFile } from '@tauri-apps/plugin-fs'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { commands, requireDesktop, unwrapResult } from '@/lib/tauri-bindings'
import type { Folder, ImportPreview, Library, SaveLibrary, Snippet, Result } from '@/lib/bindings'
import { parseStatus, ProbeCommandError, refusalLabels } from '@/lib/probe'

export type { Folder, ImportPreview, Library, SaveLibrary, Snippet }
export type LibraryErrorCode =
  'invalidData' | 'conflict' | 'storage' | 'tooLarge' | 'unsupportedVersion' | 'unavailable'
const codes: LibraryErrorCode[] = [
  'invalidData',
  'conflict',
  'storage',
  'tooLarge',
  'unsupportedVersion',
  'unavailable',
]
export class LibraryCommandError extends Error {
  constructor(public readonly code: LibraryErrorCode) {
    super('Library service is unavailable.')
    this.name = 'LibraryCommandError'
  }
}
export const libraryKeys = { all: ['library'] as const, engine: ['library-engine'] as const }
const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/
const object = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value)
const count = (value: unknown): value is number =>
  typeof value === 'number' && Number.isSafeInteger(value) && value >= 0 && value <= 0xffffffff
const text = (value: unknown, max: number): value is string =>
  typeof value === 'string' &&
  value.trim().length > 0 &&
  Array.from(value).length <= max &&
  !value.includes('\0')
export function validSnippetBody(value: unknown): value is string {
  if (typeof value !== 'string' || Array.from(value).length > 1000000) return false
  return !Array.from(value).some((character) => {
    const code = character.codePointAt(0)!
    return (code < 32 && code !== 9 && code !== 10 && code !== 13) || (code >= 127 && code <= 159)
  })
}
function validRichBody(value: unknown): value is string {
  if (typeof value !== 'string' || value.length > 2 * 1024 * 1024) return false
  if (new TextEncoder().encode(value).length > 2 * 1024 * 1024) return false
  try {
    const envelope: unknown = JSON.parse(value)
    return (
      object(envelope) &&
      envelope.version === 1 &&
      object(envelope.doc) &&
      envelope.doc.type === 'doc' &&
      (envelope.doc.content === undefined || Array.isArray(envelope.doc.content))
    )
  } catch {
    return false
  }
}
export function validSnippetTrigger(value: unknown): value is string {
  return (
    typeof value === 'string' &&
    value.length > 0 &&
    Array.from(value).length <= 128 &&
    !Array.from(value).some((character) => {
      const code = character.codePointAt(0)!
      return /\s/u.test(character) || code < 32 || (code >= 127 && code <= 159)
    })
  )
}
function parseLibrary(value: unknown): Library {
  if (
    !object(value) ||
    value.schemaVersion !== 1 ||
    !count(value.revision) ||
    !Array.isArray(value.folders) ||
    !Array.isArray(value.snippets) ||
    value.folders.length > 1000 ||
    value.snippets.length > 10000
  )
    throw new LibraryCommandError('invalidData')
  const ids = new Set<string>()
  const folders = value.folders.map((folder: unknown) => {
    if (
      !object(folder) ||
      typeof folder.id !== 'string' ||
      !uuid.test(folder.id) ||
      ids.has(folder.id) ||
      !text(folder.name, 120)
    )
      throw new LibraryCommandError('invalidData')
    ids.add(folder.id)
    return { id: folder.id, name: folder.name }
  })
  const folderIds = new Set(folders.map((folder) => folder.id))
  const triggers = new Set<string>()
  const snippets = value.snippets.map((snippet: unknown) => {
    if (
      !object(snippet) ||
      typeof snippet.id !== 'string' ||
      !uuid.test(snippet.id) ||
      ids.has(snippet.id) ||
      !(
        snippet.folderId === null ||
        (typeof snippet.folderId === 'string' && folderIds.has(snippet.folderId))
      ) ||
      !text(snippet.title, 120) ||
      typeof snippet.trigger !== 'string' ||
      !validSnippetTrigger(snippet.trigger) ||
      !validSnippetBody(snippet.body) ||
      ('richBody' in snippet && snippet.richBody !== null && !validRichBody(snippet.richBody)) ||
      typeof snippet.enabled !== 'boolean' ||
      typeof snippet.deleted !== 'boolean'
    )
      throw new LibraryCommandError('invalidData')
    if (snippet.enabled && !snippet.deleted) {
      if (triggers.has(snippet.trigger)) throw new LibraryCommandError('invalidData')
      triggers.add(snippet.trigger)
    }
    ids.add(snippet.id)
    return {
      id: snippet.id,
      folderId: snippet.folderId,
      title: snippet.title,
      trigger: snippet.trigger,
      body: snippet.body,
      ...('richBody' in snippet ? { richBody: snippet.richBody as string | null } : {}),
      enabled: snippet.enabled,
      deleted: snippet.deleted,
    }
  })
  return { schemaVersion: 1, revision: value.revision, folders, snippets }
}
async function call<T>(
  operation: () => Promise<Result<unknown, unknown>>,
  parse: (value: unknown) => T,
): Promise<T> {
  try {
    requireDesktop()
    return parse(unwrapResult(await operation()))
  } catch (error) {
    if (error instanceof LibraryCommandError) throw error
    if (object(error) && codes.includes(error.code as LibraryErrorCode))
      throw new LibraryCommandError(error.code as LibraryErrorCode)

    throw new LibraryCommandError('unavailable')
  }
}
export const loadLibrary = () => call(commands.libraryLoad, parseLibrary)
export const saveLibrary = (input: SaveLibrary) =>
  call(() => commands.librarySave(input), parseLibrary)
export const importLibrary = (input: { json: string; expectedRevision: number }) =>
  call(() => commands.libraryImport(input.json, input.expectedRevision), parseLibrary)
export const previewLibraryImport = (json: string) =>
  call(
    () => commands.libraryImportPreview(json),
    (value): ImportPreview => {
      if (
        !object(value) ||
        !count(value.folders) ||
        !count(value.snippets) ||
        !count(value.skipped)
      )
        throw new LibraryCommandError('invalidData')
      return { folders: value.folders, snippets: value.snippets, skipped: value.skipped }
    },
  )
export async function startLibrary() {
  try {
    requireDesktop()
    return parseStatus(unwrapResult(await commands.libraryStart()))
  } catch (error) {
    if (
      object(error) &&
      typeof error.code === 'string' &&
      Object.prototype.hasOwnProperty.call(refusalLabels, error.code)
    )
      throw new ProbeCommandError(error.code as keyof typeof refusalLabels)

    throw new LibraryCommandError('unavailable')
  }
}
export async function copySnippet(body: string) {
  try {
    requireDesktop()
    await writeText(body)
  } catch {
    throw new LibraryCommandError('unavailable')
  }
}
export async function selectLibraryImport(): Promise<{
  json: string
  preview: ImportPreview
} | null> {
  try {
    requireDesktop()
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!path || Array.isArray(path)) return null
    const info = await stat(path)
    if (!info.isFile || info.size > 8 * 1024 * 1024) throw new LibraryCommandError('tooLarge')
    const json = await readTextFile(path)
    if (new TextEncoder().encode(json).length > 8 * 1024 * 1024)
      throw new LibraryCommandError('tooLarge')
    return { json, preview: await previewLibraryImport(json) }
  } catch (error) {
    if (error instanceof LibraryCommandError) throw error

    throw new LibraryCommandError('unavailable')
  }
}
export async function exportLibraryFile(): Promise<boolean> {
  try {
    requireDesktop()
    const json = await call(commands.libraryExport, (value) => {
      if (typeof value !== 'string') throw new LibraryCommandError('invalidData')
      return value
    })
    const path = await save({
      defaultPath: 'texpio-library.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!path) return false
    await writeTextFile(path, json)
    return true
  } catch (error) {
    if (error instanceof LibraryCommandError) throw error

    throw new LibraryCommandError('unavailable')
  }
}
export function useLibrary() {
  const client = useQueryClient()
  useEffect(() => {
    const refresh = () => {
      void client.invalidateQueries({ queryKey: libraryKeys.all })
    }
    window.addEventListener('focus', refresh)
    let cancelled = false
    let stop: (() => void) | undefined
    if (isTauri())
      void listen('library-changed', refresh).then(
        (unlisten) => {
          if (cancelled) unlisten()
          else stop = unlisten
        },
        () => {},
      )
    return () => {
      cancelled = true
      stop?.()
      window.removeEventListener('focus', refresh)
    }
  }, [client])
  return useQuery({ queryKey: libraryKeys.all, queryFn: loadLibrary, retry: false })
}
function useLibraryMutation<T>(operation: (input: T) => Promise<Library>) {
  const client = useQueryClient()
  return useMutation({
    mutationFn: operation,
    retry: false,
    onSuccess: (library) => {
      client.setQueryData(libraryKeys.all, library)
      void client.invalidateQueries({ queryKey: libraryKeys.engine })
    },
  })
}
export const useSaveLibrary = () => useLibraryMutation(saveLibrary)
export const useImportLibrary = () => useLibraryMutation(importLibrary)
