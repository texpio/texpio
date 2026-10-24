import { act, render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, expect, it, vi } from 'vitest'
import { LibraryView } from './LibraryView'

const native = vi.hoisted(() => ({
  load: vi.fn(),
  save: vi.fn(),
  preview: vi.fn(),
  import: vi.fn(),
  export: vi.fn(),
  start: vi.fn(),
  status: vi.fn(),
  stop: vi.fn(),
  pause: vi.fn(),
  open: vi.fn(),
  read: vi.fn(),
  stat: vi.fn(),
  clipboard: vi.fn(),
}))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }))
vi.mock('@/lib/bindings', () => ({
  commands: {
    libraryLoad: native.load,
    librarySave: native.save,
    libraryImportPreview: native.preview,
    libraryImport: native.import,
    libraryExport: native.export,
    libraryStart: native.start,
    probeStatus: native.status,
    probeStop: native.stop,
    probePause: native.pause,
  },
}))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: native.open, save: vi.fn() }))
vi.mock('@tauri-apps/plugin-fs', () => ({
  readTextFile: native.read,
  stat: native.stat,
  writeTextFile: vi.fn(),
}))
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: native.clipboard }))
const initial = {
  schemaVersion: 1,
  revision: 4,
  folders: [{ id: '10000000-0000-4000-8000-000000000000', name: 'Personal' }],
  snippets: [
    {
      id: '20000000-0000-4000-8000-000000000000',
      folderId: null,
      title: 'Greeting',
      trigger: ';hello',
      body: 'Hello, नमस्ते 👋',
      enabled: true,
      deleted: false,
    },
    {
      id: '30000000-0000-4000-8000-000000000000',
      folderId: null,
      title: 'Address',
      trigger: ';addr',
      body: '123 Example Road',
      enabled: true,
      deleted: false,
    },
  ],
}
const stopped = {
  phase: 'stopped',
  profile: null,
  strategy: 'unicode',
  counters: { submitted: 0, observed: 0, refused: 0, uncertain: 0 },
  refusalReason: null,
  lastActivityAgeMs: null,
}
beforeEach(() => {
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  )
  native.load.mockResolvedValue({ status: 'ok', data: structuredClone(initial) })
  native.status.mockResolvedValue({ status: 'ok', data: stopped })
  native.stop.mockResolvedValue({ status: 'ok', data: stopped })
  native.save.mockImplementation(async (input) => ({
    status: 'ok',
    data: {
      schemaVersion: 1,
      revision: input.expectedRevision + 1,
      folders: input.folders,
      snippets: input.snippets,
    },
  }))
})
function show() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return {
    ...render(
      <QueryClientProvider client={client}>
        <LibraryView />
      </QueryClientProvider>,
    ),
    client,
  }
}
it('saves a new Unicode snippet only through explicit Save and retains the acknowledged revision', async () => {
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  expect(native.start).not.toHaveBeenCalled()
  await userEvent.click(screen.getByRole('button', { name: 'New snippet' }))
  await userEvent.type(screen.getByLabelText('Title'), 'Sign off')
  await userEvent.type(screen.getByLabelText('Trigger'), ';bye')
  await userEvent.type(screen.getByLabelText('Body'), 'धन्यवाद 👋')
  expect(native.save).not.toHaveBeenCalled()
  await userEvent.click(screen.getByRole('button', { name: 'Save snippet' }))
  await screen.findByText('Saved. Start expansion again to use your changes.')
  expect(native.save).toHaveBeenCalledWith(
    expect.objectContaining({
      expectedRevision: 4,
      snippets: expect.arrayContaining([
        expect.objectContaining({ title: 'Sign off', trigger: ';bye', body: 'धन्यवाद 👋' }),
      ]),
    }),
  )
  expect(screen.getByRole('button', { name: 'Save snippet' })).toBeDisabled()
  expect(localStorage.getItem('texpio.layout') ?? '').not.toContain('धन्यवाद')
})
it('searches title, trigger and body while preserving the unsaved editor', async () => {
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await userEvent.type(screen.getByLabelText('Title'), ' draft')
  await userEvent.type(screen.getByRole('searchbox'), 'Example Road')
  expect(screen.queryByRole('button', { name: /Greeting/ })).not.toBeInTheDocument()
  expect(screen.getByRole('button', { name: /Address/ })).toBeInTheDocument()
  expect(screen.getByLabelText('Title')).toHaveValue('Greeting draft')
  await userEvent.click(screen.getByRole('button', { name: /Address/ }))
  const dialog = screen.getByRole('alertdialog')
  await userEvent.click(within(dialog).getByRole('button', { name: 'Keep editing' }))
  expect(screen.getByLabelText('Title')).toHaveValue('Greeting draft')
  await userEvent.click(screen.getByRole('button', { name: /Address/ }))
  await userEvent.click(
    within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Discard draft' }),
  )
  expect(screen.getByLabelText('Title')).toHaveValue('Address')
})
it('keeps a conflicted draft and refuses invalid or duplicate active triggers before saving', async () => {
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await userEvent.clear(screen.getByLabelText('Trigger'))
  await userEvent.type(screen.getByLabelText('Trigger'), ';addr')
  await userEvent.click(screen.getByRole('button', { name: 'Save snippet' }))
  expect(screen.getByRole('alert')).toHaveTextContent('already used')
  expect(native.save).not.toHaveBeenCalled()
  await userEvent.clear(screen.getByLabelText('Trigger'))
  await userEvent.type(screen.getByLabelText('Trigger'), ';new')
  native.save.mockResolvedValueOnce({ status: 'error', error: { code: 'conflict' } })
  await userEvent.click(screen.getByRole('button', { name: 'Save snippet' }))
  await screen.findByText(
    'The library changed in another window. Your draft is kept. Reload before saving again.',
  )
  expect(screen.getByLabelText('Trigger')).toHaveValue(';new')
  expect(screen.getByRole('button', { name: 'Save snippet' })).toBeEnabled()
})
it('previews selected JSON before importing and uses the current revision for confirmation', async () => {
  native.open.mockResolvedValue('C:/selected/library.json')
  native.stat.mockResolvedValue({ isFile: true, size: 320 })
  native.read.mockResolvedValue('{"schemaVersion":1}')
  native.preview.mockResolvedValue({ status: 'ok', data: { folders: 1, snippets: 2, skipped: 3 } })
  native.import.mockResolvedValue({ status: 'ok', data: { ...initial, revision: 5 } })
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await userEvent.click(screen.getByRole('button', { name: 'Import JSON' }))
  const dialog = await screen.findByRole('dialog', { name: 'Import preview' })
  expect(dialog).toHaveTextContent('1 folders, 2 snippets, 3 unchanged records skipped')
  expect(native.import).not.toHaveBeenCalled()
  await userEvent.click(within(dialog).getByRole('button', { name: 'Import snippets' }))
  await waitFor(() => expect(native.import).toHaveBeenCalledWith('{"schemaVersion":1}', 4))
})
it('never replaces an unsaved draft when a payload-free change event refreshes the library', async () => {
  const { client } = show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await userEvent.type(screen.getByLabelText('Body'), ' locally edited')
  act(() => client.setQueryData(['library'], { ...initial, revision: 5 }))
  expect(screen.getByLabelText('Body')).toHaveValue('Hello, नमस्ते 👋 locally edited')
  expect(
    await screen.findByText(
      'The library changed in another window. Your draft is kept. Reload before saving again.',
    ),
  ).toBeInTheDocument()
})
it('trashes the latest acknowledged record without restoring stale editor content', async () => {
  const { client } = show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  act(() =>
    client.setQueryData(['library'], {
      ...initial,
      revision: 5,
      snippets: initial.snippets.map((item) =>
        item.title === 'Greeting' ? { ...item, body: 'Updated elsewhere' } : item,
      ),
    }),
  )
  await screen.findByText(
    'The library changed in another window. Your draft is kept. Reload before saving again.',
  )
  await userEvent.click(screen.getByRole('button', { name: 'Move to Trash' }))
  await waitFor(() =>
    expect(native.save).toHaveBeenCalledWith(
      expect.objectContaining({
        expectedRevision: 5,
        snippets: expect.arrayContaining([
          expect.objectContaining({ title: 'Greeting', body: 'Updated elsewhere', deleted: true }),
        ]),
      }),
    ),
  )
})
it('keeps refused imports visible inside their confirmation dialog', async () => {
  native.open.mockResolvedValue('C:/selected/library.json')
  native.stat.mockResolvedValue({ isFile: true, size: 320 })
  native.read.mockResolvedValue('{"schemaVersion":1}')
  native.preview.mockResolvedValue({ status: 'ok', data: { folders: 0, snippets: 1, skipped: 0 } })
  native.import.mockResolvedValue({ status: 'error', error: { code: 'conflict' } })
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await userEvent.click(screen.getByRole('button', { name: 'Import JSON' }))
  const dialog = await screen.findByRole('dialog', { name: 'Import preview' })
  await userEvent.click(within(dialog).getByRole('button', { name: 'Import snippets' }))
  expect(await within(dialog).findByRole('alert')).toHaveTextContent('No records were imported')
})
it('duplicates as disabled, then trashes and restores without losing literal text', async () => {
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await userEvent.click(screen.getByRole('button', { name: 'Duplicate' }))
  expect(screen.getByLabelText('Title')).toHaveValue('Greeting (copy)')
  expect(screen.getByRole('switch')).not.toBeChecked()
  await userEvent.click(screen.getByRole('button', { name: 'Save snippet' }))
  await screen.findByText('Saved. Start expansion again to use your changes.')
  await userEvent.click(screen.getByRole('button', { name: 'Move to Trash' }))
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(2))
  await userEvent.click(screen.getByRole('radio', { name: 'Trash' }))
  await userEvent.click(await screen.findByRole('button', { name: /Greeting \(copy\)/ }))
  expect(screen.getByLabelText('Body')).toHaveValue('Hello, नमस्ते 👋')
  await userEvent.click(screen.getByRole('button', { name: 'Restore snippet' }))
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(3))
  expect(native.save).toHaveBeenLastCalledWith(
    expect.objectContaining({
      snippets: expect.arrayContaining([
        expect.objectContaining({
          title: 'Greeting (copy)',
          enabled: false,
          deleted: false,
          body: 'Hello, नमस्ते 👋',
        }),
      ]),
    }),
  )
})
it('creates folders and removes them while preserving their snippets as unfiled', async () => {
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await userEvent.click(screen.getByRole('button', { name: 'New folder' }))
  await userEvent.type(screen.getByLabelText('Folder name'), 'Work')
  await userEvent.click(screen.getByRole('button', { name: 'Save folder' }))
  await screen.findByRole('button', { name: 'Manage folder' })
  await userEvent.click(screen.getByRole('button', { name: 'New snippet' }))
  await userEvent.type(screen.getByLabelText('Title'), 'Work note')
  await userEvent.type(screen.getByLabelText('Trigger'), ';work')
  await userEvent.type(screen.getByLabelText('Body'), 'Keep this text')
  await userEvent.click(screen.getByRole('button', { name: 'Save snippet' }))
  await screen.findByText('Saved. Start expansion again to use your changes.')
  await userEvent.click(screen.getByRole('button', { name: 'Manage folder' }))
  await userEvent.click(screen.getByRole('button', { name: 'Remove folder' }))
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(3))
  expect(native.save).toHaveBeenLastCalledWith(
    expect.objectContaining({
      folders: initial.folders,
      snippets: expect.arrayContaining([
        expect.objectContaining({ title: 'Work note', folderId: null, body: 'Keep this text' }),
      ]),
    }),
  )
})
