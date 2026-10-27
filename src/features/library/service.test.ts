import { beforeEach, expect, it, vi } from 'vitest'
import { copySnippet, loadLibrary, saveLibrary, selectLibraryImport } from '@/services/library'
const native = vi.hoisted(() => ({
  desktop: vi.fn(),
  load: vi.fn(),
  save: vi.fn(),
  open: vi.fn(),
  stat: vi.fn(),
  read: vi.fn(),
  preview: vi.fn(),
  copy: vi.fn(),
}))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: native.desktop }))
vi.mock('@/lib/bindings', () => ({
  commands: {
    libraryLoad: native.load,
    librarySave: native.save,
    libraryImportPreview: native.preview,
  },
}))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: native.open, save: vi.fn() }))
vi.mock('@tauri-apps/plugin-fs', () => ({
  stat: native.stat,
  readTextFile: native.read,
  writeTextFile: vi.fn(),
}))
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: native.copy }))
beforeEach(() => {
  native.desktop.mockReturnValue(true)
})
const richBody = JSON.stringify({
  version: 1,
  doc: {
    type: 'doc',
    content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Hello' }] }],
  },
})
const snippet = {
  id: '20000000-0000-4000-8000-000000000000',
  folderId: null,
  title: 'Greeting',
  trigger: ';hello',
  body: 'Hello',
  enabled: true,
  deleted: false,
}
const library = { schemaVersion: 1, revision: 1, folders: [], snippets: [snippet] }

it('retains rich documents through load and save while leaving legacy omitted and null values intact', async () => {
  const rich = { ...library, snippets: [{ ...snippet, richBody }] }
  native.load.mockResolvedValueOnce({ status: 'ok', data: rich })
  const loaded = await loadLibrary()
  expect(loaded).toEqual(rich)
  native.save.mockResolvedValueOnce({ status: 'ok', data: rich })
  const input = { expectedRevision: 1, folders: loaded.folders, snippets: loaded.snippets }
  expect(await saveLibrary(input)).toEqual(rich)
  expect(native.save).toHaveBeenCalledWith(input)
  for (const legacy of [snippet, { ...snippet, richBody: null }]) {
    native.load.mockResolvedValueOnce({ status: 'ok', data: { ...library, snippets: [legacy] } })
    expect((await loadLibrary()).snippets[0]).toEqual(legacy)
  }
})

it('refuses malformed, wrongly typed, or oversized rich documents from native responses', async () => {
  for (const invalid of [
    '{',
    42,
    JSON.stringify({ version: 2, doc: { type: 'doc' } }),
    JSON.stringify({ version: 1, doc: { type: 'doc', content: 'invalid' } }),
    'x'.repeat(2 * 1024 * 1024 + 1),
    JSON.stringify({
      version: 1,
      doc: {
        type: 'doc',
        content: [
          { type: 'paragraph', content: [{ type: 'text', text: 'é'.repeat(1024 * 1024) }] },
        ],
      },
    }),
  ]) {
    native.load.mockResolvedValueOnce({
      status: 'ok',
      data: { ...library, snippets: [{ ...snippet, richBody: invalid }] },
    })
    await expect(loadLibrary()).rejects.toMatchObject({ code: 'invalidData' })
  }
})
it('browser preview refuses native library and clipboard operations without invoking them', async () => {
  native.desktop.mockReturnValue(false)
  await expect(loadLibrary()).rejects.toMatchObject({ code: 'unavailable' })
  await expect(copySnippet('synthetic body')).rejects.toMatchObject({ code: 'unavailable' })
  expect(native.load).not.toHaveBeenCalled()
  expect(native.copy).not.toHaveBeenCalled()
})
it('redacts an unknown native exception and validates malformed records before showing content', async () => {
  native.load.mockRejectedValueOnce(new Error('private filename'))
  await expect(loadLibrary()).rejects.toMatchObject({
    code: 'unavailable',
    message: 'Library service is unavailable.',
  })
  native.load.mockResolvedValueOnce({
    status: 'ok',
    data: {
      schemaVersion: 1,
      revision: 0,
      folders: [],
      snippets: [{ id: 'bad', body: 'private body' }],
    },
  })
  await expect(loadLibrary()).rejects.toMatchObject({ code: 'invalidData' })
})
it('does not read canceled or oversized selected imports', async () => {
  native.open.mockResolvedValueOnce(null)
  await expect(selectLibraryImport()).resolves.toBeNull()
  expect(native.stat).not.toHaveBeenCalled()
  native.open.mockResolvedValueOnce('C:/selected/large.json')
  native.stat.mockResolvedValueOnce({ isFile: true, size: 8 * 1024 * 1024 + 1 })
  await expect(selectLibraryImport()).rejects.toMatchObject({ code: 'tooLarge' })
  expect(native.read).not.toHaveBeenCalled()
  expect(native.preview).not.toHaveBeenCalled()
})
