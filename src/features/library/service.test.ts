import { beforeEach, expect, it, vi } from 'vitest'
import { copySnippet, loadLibrary, selectLibraryImport } from '@/services/library'
const native = vi.hoisted(() => ({
  desktop: vi.fn(),
  load: vi.fn(),
  open: vi.fn(),
  stat: vi.fn(),
  read: vi.fn(),
  preview: vi.fn(),
  copy: vi.fn(),
}))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: native.desktop }))
vi.mock('@/lib/bindings', () => ({
  commands: { libraryLoad: native.load, libraryImportPreview: native.preview },
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
