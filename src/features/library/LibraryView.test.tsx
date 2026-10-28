import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, expect, it, vi } from 'vitest'
import { LibraryView } from './LibraryView'
import { LibraryNavigation } from './LibraryNavigation'
import { useUIStore } from '@/store/ui-store'
import { createKeyboardDispatcher } from '@/hooks/use-keyboard-shortcuts'
import { mockPanelGeometry } from '@/test/panel-layout'
import type { Editor } from '@tiptap/core'
import { toast } from 'sonner'
import type { SaveLibrary } from '@/lib/bindings'

let persistedLibrary: unknown

const native = vi.hoisted(() => ({
  load: vi.fn(),
  save: vi.fn(),
  metadata: vi.fn(),
  favorite: vi.fn(),
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
  quit: vi.fn(),
  listen: vi.fn().mockResolvedValue(() => {}),
}))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@tauri-apps/api/event', () => ({ listen: native.listen }))
vi.mock('@/lib/bindings', () => ({
  commands: {
    libraryLoad: native.load,
    libraryMetadata: native.metadata,
    librarySetFavorite: native.favorite,
    attachmentList: async () => ({ status: 'ok', data: [] }),
    attachmentSettings: async () => ({
      status: 'ok',
      data: { attachmentsFirst: false, strictOrder: false },
    }),
    librarySave: async (input: SaveLibrary) => {
      const result = await native.save(input)
      if (result.status === 'ok') persistedLibrary = structuredClone(result.data)
      return result
    },
    libraryImportPreview: native.preview,
    libraryImport: async (json: string, revision: number) => {
      const result = await native.import(json, revision)
      if (result.status === 'ok') persistedLibrary = structuredClone(result.data)
      return result
    },
    libraryExport: native.export,
    libraryStart: native.start,
    probeStatus: native.status,
    productStatus: async () => ({
      phase: 'stopped',
      submitted: 0,
      cancelled: 0,
      lastError: null,
      clipboardIncomplete: false,
    }),
    probeStop: native.stop,
    probePause: native.pause,
    quitApp: native.quit,
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
  mockPanelGeometry()
  vi.spyOn(window, 'scrollBy').mockImplementation(() => {})
  Object.defineProperty(document, 'elementFromPoint', {
    configurable: true,
    value: () => document.activeElement,
  })
  Object.defineProperty(Range.prototype, 'getBoundingClientRect', {
    configurable: true,
    value: () => new DOMRect(0, 0, 1, 16),
  })
  Object.defineProperty(Range.prototype, 'getClientRects', {
    configurable: true,
    value: () => [new DOMRect(0, 0, 1, 16)],
  })
  useUIStore.setState({ activeView: 'home', libraryCollection: 'active', libraryFolder: 'all' })
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  )
  persistedLibrary = structuredClone(initial)
  native.load.mockImplementation(async () => ({
    status: 'ok',
    data: structuredClone(persistedLibrary),
  }))
  native.metadata.mockResolvedValue({
    status: 'ok',
    data: initial.snippets.map((snippet, index) => ({
      snippetId: snippet.id,
      favorite: index === 1,
      usageCount: index + 1,
      createdAt: 1,
      updatedAt: 2,
      lastUsedAt: 3,
    })),
  })
  native.favorite.mockResolvedValue({ status: 'ok', data: null })
  native.status.mockResolvedValue({ status: 'ok', data: stopped })
  native.stop.mockResolvedValue({ status: 'ok', data: stopped })
  native.quit.mockResolvedValue({ status: 'ok', data: null })
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
        <LibraryNavigation />
        <LibraryView />
      </QueryClientProvider>,
    ),
    client,
  }
}
async function appendBody(text: string) {
  const body = await screen.findByRole('textbox', { name: 'Body' })
  const editor = (body as HTMLElement & { editor: Editor }).editor
  expect(editor).toBeDefined()
  act(() => {
    editor.commands.insertContentAt(editor.state.doc.content.size - 1, { type: 'text', text })
  })
  await waitFor(() => expect(body.textContent).toContain(text))
}
it('shows a success toast instead of an inline banner after creating a folder', async () => {
  const success = vi.spyOn(toast, 'success')
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await userEvent.click(screen.getByRole('button', { name: 'New folder' }))
  await userEvent.type(screen.getByLabelText('Folder name'), 'Work')
  await userEvent.click(screen.getByRole('button', { name: 'Save folder' }))
  await waitFor(() => expect(success).toHaveBeenCalledWith('Saved.'))
  expect(screen.queryByText('Done')).not.toBeInTheDocument()
  success.mockRestore()
})
it('automatically saves an edited snippet without a save button', async () => {
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await appendBody(' automatically saved')
  expect(screen.queryByRole('button', { name: 'Save snippet' })).not.toBeInTheDocument()
  await waitFor(
    () =>
      expect(native.save).toHaveBeenCalledWith(
        expect.objectContaining({
          expectedRevision: 4,
          snippets: expect.arrayContaining([
            expect.objectContaining({ body: 'Hello, नमस्ते 👋 automatically saved' }),
          ]),
        }),
      ),
    { timeout: 3000 },
  )
})
it('saves later edits after an in-flight save without replacing them', async () => {
  let finish!: (value: unknown) => void
  native.save.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await appendBody(' first')
  await waitFor(() => expect(native.save).toHaveBeenCalledOnce(), { timeout: 3000 })
  const first = native.save.mock.calls[0][0]
  await appendBody(' second')
  await act(async () =>
    finish({
      status: 'ok',
      data: { schemaVersion: 1, revision: 5, folders: first.folders, snippets: first.snippets },
    }),
  )
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(2), { timeout: 3000 })
  expect(native.save).toHaveBeenLastCalledWith(
    expect.objectContaining({
      expectedRevision: 5,
      snippets: expect.arrayContaining([
        expect.objectContaining({ body: 'Hello, नमस्ते 👋 first second' }),
      ]),
    }),
  )
  expect(screen.getByLabelText('Body')).toHaveTextContent('Hello, नमस्ते 👋 first second')
})
it('keeps a failed autosave and retries without losing the draft', async () => {
  native.save.mockResolvedValueOnce({ status: 'error', error: { code: 'storage' } })
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await appendBody(' retained')
  const retry = await screen.findByRole('button', { name: 'Retry' }, { timeout: 3000 })
  expect(screen.getByLabelText('Body')).toHaveTextContent('Hello, नमस्ते 👋 retained')
  await userEvent.click(retry)
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(2))
  await waitFor(() => expect(screen.queryByText('Unsaved draft')).not.toBeInTheDocument())
})
it('waits for an in-flight autosave before native Quit', async () => {
  let finish!: (value: unknown) => void
  native.save.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await appendBody(' before quit')
  await waitFor(() => expect(native.save).toHaveBeenCalledOnce(), { timeout: 3000 })
  const quit = native.listen.mock.calls.find(([event]) => event === 'quit-requested')![1]
  act(() => quit({ payload: null }))
  expect(native.quit).not.toHaveBeenCalled()
  const input = native.save.mock.calls[0][0]
  await act(async () =>
    finish({
      status: 'ok',
      data: { schemaVersion: 1, revision: 5, folders: input.folders, snippets: input.snippets },
    }),
  )
  await waitFor(() => expect(native.quit).toHaveBeenCalledOnce())
})
it('flushes an edit made in the same turn as a quit request', async () => {
  let finish!: (value: unknown) => void
  native.save.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  const title = screen.getByLabelText('Title')
  const quit = native.listen.mock.calls.find(([event]) => event === 'quit-requested')![1]
  act(() => {
    fireEvent.change(title, { target: { value: 'Latest title' } })
    quit({ payload: null })
  })
  await waitFor(() => expect(native.save).toHaveBeenCalledOnce())
  expect(native.quit).not.toHaveBeenCalled()
  const input = native.save.mock.calls[0][0]
  expect(input.snippets).toEqual(
    expect.arrayContaining([expect.objectContaining({ title: 'Latest title' })]),
  )
  await act(async () =>
    finish({
      status: 'ok',
      data: { schemaVersion: 1, revision: 5, folders: input.folders, snippets: input.snippets },
    }),
  )
  await waitFor(() => expect(native.quit).toHaveBeenCalledOnce())
})
it('saves a reverted edit after the pending save before quitting', async () => {
  let finish!: (value: unknown) => void
  native.save.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  const title = screen.getByLabelText('Title')
  fireEvent.change(title, { target: { value: 'Changed title' } })
  await waitFor(() => expect(native.save).toHaveBeenCalledOnce(), { timeout: 3000 })
  const first = native.save.mock.calls[0][0]
  fireEvent.change(title, { target: { value: 'Greeting' } })
  await waitFor(() => expect(title).toHaveValue('Greeting'))
  const quit = native.listen.mock.calls.find(([event]) => event === 'quit-requested')![1]
  act(() => quit({ payload: null }))
  await act(async () => {
    await Promise.resolve()
    await Promise.resolve()
  })
  expect(native.quit).not.toHaveBeenCalled()
  await act(async () =>
    finish({
      status: 'ok',
      data: { schemaVersion: 1, revision: 5, folders: first.folders, snippets: first.snippets },
    }),
  )
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(2), { timeout: 3000 })
  expect(native.save).toHaveBeenLastCalledWith(
    expect.objectContaining({
      expectedRevision: 5,
      snippets: expect.arrayContaining([expect.objectContaining({ title: 'Greeting' })]),
    }),
  )
  await waitFor(() => expect(native.quit).toHaveBeenCalledOnce())
})
it('blocks expansion starts for drafts and saves, then fails closed when the editor unmounts', async () => {
  const view = show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  expect(useUIStore.getState().expansionStartBlocked).toBe(false)
  await appendBody(' draft')
  await waitFor(() => expect(useUIStore.getState().expansionStartBlocked).toBe(true))
  let finish!: (value: unknown) => void
  native.save.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  await waitFor(() => expect(native.save).toHaveBeenCalledOnce(), { timeout: 3000 })
  expect(useUIStore.getState().expansionStartBlocked).toBe(true)
  await act(async () =>
    finish({
      status: 'ok',
      data: {
        ...initial,
        revision: 5,
        snippets: native.save.mock.calls[0][0].snippets,
      },
    }),
  )
  await waitFor(() => expect(useUIStore.getState().expansionStartBlocked).toBe(false))
  view.unmount()
  expect(useUIStore.getState().expansionStartBlocked).toBe(true)
})
it('filters favorites and keeps an edited body when usage metadata refreshes', async () => {
  const { client } = show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await appendBody(' draft')
  await userEvent.click(screen.getByRole('button', { name: 'Add to favorites' }))
  await waitFor(() => expect(native.favorite).toHaveBeenCalledWith(initial.snippets[0].id, 4, true))
  act(() =>
    client.setQueryData(
      ['library-metadata'],
      initial.snippets.map((snippet) => ({
        snippetId: snippet.id,
        favorite: snippet.id === initial.snippets[1].id,
        usageCount: 99,
        createdAt: 1,
        updatedAt: 2,
        lastUsedAt: 4,
      })),
    ),
  )
  expect(screen.getByLabelText('Body').textContent).toBe('Hello, नमस्ते 👋 draft')
  await userEvent.click(screen.getByRole('button', { name: 'Favorites' }))
  expect(screen.queryByRole('button', { name: /Greeting/ })).not.toBeInTheDocument()
  expect(screen.getByRole('button', { name: /Address/ })).toBeInTheDocument()
  expect(screen.getByLabelText('Body').textContent).toBe('Hello, नमस्ते 👋 draft')
})
it('navigates the list without deleting text drafts and dispatches find/save from the editor', async () => {
  const dispatch = createKeyboardDispatcher({
    platform: 'windows',
    openPreferences: vi.fn(),
    showToast: vi.fn(),
    windowAction: vi.fn().mockResolvedValue({ success: true }),
  })
  document.addEventListener('keydown', dispatch)
  try {
    show()
    const greeting = await screen.findByRole('button', { name: /Greeting/ })
    await userEvent.click(greeting)
    const body = screen.getByLabelText('Body')
    await appendBody(' draft')
    body.focus()
    await userEvent.keyboard('{Delete}')
    expect(native.save).not.toHaveBeenCalled()
    greeting.focus()
    await userEvent.keyboard('{ArrowDown}')
    expect(screen.getByRole('button', { name: /Address/ })).toHaveFocus()
    expect(body.textContent).toBe('Hello, नमस्ते 👋 draft')
    await userEvent.click(body)
    await userEvent.keyboard('{Control>}f{/Control}')
    expect(screen.getByLabelText('Find in body')).toBeInTheDocument()
    await userEvent.click(body)
    await userEvent.keyboard('{Control>}s{/Control}')
    await waitFor(() => expect(native.save).toHaveBeenCalledOnce())
  } finally {
    document.removeEventListener('keydown', dispatch)
  }
})
it('keeps a draft when saving fails and Quit is cancelled', async () => {
  native.save.mockResolvedValue({ status: 'error', error: { code: 'storage' } })
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await appendBody(' unsaved')
  await waitFor(() =>
    expect(native.listen.mock.calls.some(([event]) => event === 'quit-requested')).toBe(true),
  )
  const quit = native.listen.mock.calls.find(([event]) => event === 'quit-requested')![1]
  act(() => quit({ payload: null }))
  await userEvent.click(
    within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Keep editing' }),
  )
  expect(native.quit).not.toHaveBeenCalled()
  expect(screen.getByLabelText('Body').textContent).toBe('Hello, नमस्ते 👋 unsaved')
  act(() => quit({ payload: null }))
  await userEvent.click(
    within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Discard draft' }),
  )
  await waitFor(() => expect(native.quit).toHaveBeenCalledOnce())
})

it('acknowledges native Quit without a discard prompt when the editor is clean', async () => {
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await waitFor(() =>
    expect(native.listen.mock.calls.some(([event]) => event === 'quit-requested')).toBe(true),
  )
  const quit = native.listen.mock.calls.find(([event]) => event === 'quit-requested')![1]
  act(() => quit({ payload: null }))
  await waitFor(() => expect(native.quit).toHaveBeenCalledOnce())
  expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
})
it('automatically saves a valid new Unicode snippet', async () => {
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  expect(native.start).not.toHaveBeenCalled()
  await userEvent.click(screen.getByRole('button', { name: 'New snippet' }))
  expect(screen.getByRole('switch', { name: 'Enabled for expansion' })).toBeChecked()
  await userEvent.type(screen.getByLabelText('Title'), 'Sign off')
  await userEvent.type(screen.getByLabelText('Trigger'), ';bye')
  await appendBody('धन्यवाद 👋')
  expect(native.save).not.toHaveBeenCalled()
  await waitFor(
    () =>
      expect(native.save).toHaveBeenCalledWith(
        expect.objectContaining({
          expectedRevision: 4,
          snippets: expect.arrayContaining([
            expect.objectContaining({ title: 'Sign off', trigger: ';bye', body: 'धन्यवाद 👋' }),
          ]),
        }),
      ),
    { timeout: 3000 },
  )
  expect(localStorage.getItem('texpio.layout') ?? '').not.toContain('धन्यवाद')
})
it('can discard an incomplete new snippet without writing it to the library', async () => {
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await userEvent.click(screen.getByRole('button', { name: 'New snippet' }))
  await userEvent.type(screen.getByLabelText('Title'), 'Incomplete')
  await menuAction('Snippet actions', 'Discard draft')
  expect(screen.queryByLabelText('Title')).not.toBeInTheDocument()
  expect(native.save).not.toHaveBeenCalled()
})
it('focuses the title for a new snippet and preserves an unsaved body across editor sections', async () => {
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await userEvent.click(screen.getByRole('button', { name: 'New snippet' }))
  expect(screen.getByLabelText('Title')).toHaveFocus()
  await userEvent.type(screen.getByLabelText('Title'), 'A new reply')
  await userEvent.type(screen.getByLabelText('Trigger'), ';reply')
  await appendBody('Unfinished नमस्ते')
  await userEvent.click(screen.getByRole('radio', { name: 'Preview' }))
  expect(screen.getByRole('region', { name: 'Text preview' })).toHaveTextContent(
    'Unfinished नमस्ते',
  )
  await userEvent.click(screen.getByRole('radio', { name: 'Content' }))
  expect(screen.getByLabelText('Body').textContent).toBe('Unfinished नमस्ते')
  expect(native.save).not.toHaveBeenCalled()
})
it('saves the current editor before selecting another search result', async () => {
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await userEvent.type(screen.getByLabelText('Title'), ' draft')
  await userEvent.type(screen.getByRole('searchbox'), 'Example Road')
  expect(screen.queryByRole('button', { name: /Greeting/ })).not.toBeInTheDocument()
  expect(screen.getByRole('button', { name: /Address/ })).toBeInTheDocument()
  expect(screen.getByLabelText('Title')).toHaveValue('Greeting draft')
  await userEvent.click(screen.getByRole('button', { name: /Address/ }))
  await waitFor(() => expect(screen.getByLabelText('Title')).toHaveValue('Address'))
  expect(native.save).toHaveBeenCalledOnce()
})
it('keeps a conflicted draft and refuses invalid or duplicate active triggers before saving', async () => {
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await userEvent.clear(screen.getByLabelText('Trigger'))
  await userEvent.type(screen.getByLabelText('Trigger'), ';addr')
  expect(await screen.findByRole('alert', {}, { timeout: 3000 })).toHaveTextContent('already used')
  expect(native.save).not.toHaveBeenCalled()
  await userEvent.clear(screen.getByLabelText('Trigger'))
  await userEvent.type(screen.getByLabelText('Trigger'), ';new')
  native.save.mockResolvedValueOnce({ status: 'error', error: { code: 'conflict' } })
  await screen.findByText(
    'The library changed in another window. Your draft is kept. Reload before saving again.',
    {},
    { timeout: 3000 },
  )
  expect(screen.getByLabelText('Trigger')).toHaveValue(';new')
  expect(screen.getByText('Unsaved draft')).toBeVisible()
})
it('previews selected JSON before importing and uses the current revision for confirmation', async () => {
  native.open.mockResolvedValue('C:/selected/library.json')
  native.stat.mockResolvedValue({ isFile: true, size: 320 })
  native.read.mockResolvedValue('{"schemaVersion":1}')
  native.preview.mockResolvedValue({ status: 'ok', data: { folders: 1, snippets: 2, skipped: 3 } })
  native.import.mockResolvedValue({ status: 'ok', data: { ...initial, revision: 5 } })
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await menuAction('Library actions', 'Import JSON')
  const dialog = await screen.findByRole('dialog', { name: 'Import preview' })
  expect(dialog).toHaveTextContent('1 folders, 2 snippets, 3 unchanged records skipped')
  expect(native.import).not.toHaveBeenCalled()
  await userEvent.click(within(dialog).getByRole('button', { name: 'Import snippets' }))
  await waitFor(() => expect(native.import).toHaveBeenCalledWith('{"schemaVersion":1}', 4))
})
it('never replaces an unsaved draft when a payload-free change event refreshes the library', async () => {
  const { client } = show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await appendBody(' locally edited')
  act(() => client.setQueryData(['library'], { ...initial, revision: 5 }))
  expect(screen.getByLabelText('Body').textContent).toBe('Hello, नमस्ते 👋 locally edited')
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
  await menuAction('Snippet actions', 'Move to Trash')
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
  await menuAction('Library actions', 'Import JSON')
  const dialog = await screen.findByRole('dialog', { name: 'Import preview' })
  await userEvent.click(within(dialog).getByRole('button', { name: 'Import snippets' }))
  expect(await within(dialog).findByRole('alert')).toHaveTextContent('No records were imported')
})
it('duplicates as disabled, then trashes and restores without losing literal text', async () => {
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await menuAction('Snippet actions', 'Duplicate')
  expect(screen.getByLabelText('Title')).toHaveValue('Greeting (copy)')
  expect(screen.getByRole('switch')).not.toBeChecked()
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(1), { timeout: 3000 })
  await menuAction('Snippet actions', 'Move to Trash')
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(2))
  await userEvent.click(screen.getByRole('button', { name: 'Trash' }))
  await userEvent.click(await screen.findByRole('button', { name: /Greeting \(copy\)/ }))
  expect(screen.getByLabelText('Body').textContent).toBe('Hello, नमस्ते 👋')
  await menuAction('Snippet actions', 'Restore snippet')
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
it('duplicates a rich snippet without losing its saved rich document', async () => {
  const richBody = JSON.stringify({
    version: 1,
    doc: {
      type: 'doc',
      content: [{ type: 'paragraph', content: [{ type: 'text', text: initial.snippets[0].body }] }],
    },
  })
  native.load.mockResolvedValueOnce({
    status: 'ok',
    data: {
      ...initial,
      snippets: [{ ...initial.snippets[0], richBody }, initial.snippets[1]],
    },
  })
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  await menuAction('Snippet actions', 'Duplicate')
  await waitFor(
    () =>
      expect(native.save).toHaveBeenCalledWith(
        expect.objectContaining({
          snippets: expect.arrayContaining([
            expect.objectContaining({
              title: 'Greeting (copy)',
              body: initial.snippets[0].body,
              richBody,
              enabled: false,
            }),
          ]),
        }),
      ),
    { timeout: 3000 },
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
  await appendBody('Keep this text')
  await waitFor(() => expect(native.save).toHaveBeenCalledTimes(2), { timeout: 3000 })
  await waitFor(() => expect(screen.queryByText('Saving…')).not.toBeInTheDocument())
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

async function menuAction(menu: string, action: string) {
  await userEvent.click(screen.getByRole('button', { name: menu }))
  await userEvent.click(await screen.findByRole('menuitem', { name: action }))
}

it('saves the current draft before exporting the library', async () => {
  let finish!: (value: unknown) => void
  native.save.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  native.export.mockResolvedValue({ status: 'ok', data: '{}' })
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  fireEvent.change(screen.getByLabelText('Title'), { target: { value: 'Updated greeting' } })
  await menuAction('Library actions', 'Export JSON')
  await waitFor(() => expect(native.save).toHaveBeenCalledOnce())
  expect(native.export).not.toHaveBeenCalled()
  const input = native.save.mock.calls[0][0]
  expect(input.snippets).toEqual(
    expect.arrayContaining([expect.objectContaining({ title: 'Updated greeting' })]),
  )
  await act(async () =>
    finish({
      status: 'ok',
      data: { schemaVersion: 1, revision: 5, folders: input.folders, snippets: input.snippets },
    }),
  )
  await waitFor(() => expect(native.export).toHaveBeenCalledOnce())
})
it('returns to all snippets when a selected folder disappears after a library refresh', async () => {
  const { client } = show()
  await screen.findByRole('button', { name: /Greeting/ })
  await userEvent.click(screen.getByRole('button', { name: 'Personal' }))
  expect(screen.queryByRole('button', { name: /Greeting/ })).not.toBeInTheDocument()
  act(() => client.setQueryData(['library'], { ...initial, folders: [], revision: 5 }))
  expect(await screen.findByRole('button', { name: /Greeting/ })).toBeInTheDocument()
  expect(screen.getByRole('button', { name: 'All snippets' })).toHaveAttribute(
    'aria-current',
    'page',
  )
})
it('starts a new snippet outside search and Favorites so the saved record remains visible', async () => {
  show()
  await screen.findByRole('button', { name: /Greeting/ })
  await userEvent.click(screen.getByRole('button', { name: 'Favorites' }))
  await userEvent.type(screen.getByLabelText('Search snippets'), 'Address')
  await userEvent.click(screen.getByRole('button', { name: 'New snippet' }))
  expect(screen.getByRole('button', { name: 'All snippets' })).toHaveAttribute(
    'aria-current',
    'page',
  )
  expect(screen.getByLabelText('Search snippets')).toHaveValue('')
  await userEvent.type(screen.getByLabelText('Title'), 'New reply')
  await userEvent.type(screen.getByLabelText('Trigger'), ';reply')
  await appendBody('New body')
  expect(await screen.findByRole('button', { name: /New reply/ })).toBeVisible()
})
it('shows body validation during autosave outside the Content section', async () => {
  show()
  await userEvent.click(await screen.findByRole('button', { name: /Greeting/ }))
  const body = await screen.findByRole('textbox', { name: 'Body' })
  const editor = (body as HTMLElement & { editor: Editor }).editor
  act(() => {
    editor.commands.setContent({
      type: 'doc',
      content: [{ type: 'paragraph', content: [{ type: 'text', text: 'bad\u0001text' }] }],
    })
  })
  await waitFor(() => expect(body.textContent).toBe('bad\u0001text'))
  await userEvent.click(screen.getByRole('radio', { name: 'Preview' }))
  expect(await screen.findByRole('alert', {}, { timeout: 3000 })).toBeVisible()
  expect(native.save).not.toHaveBeenCalled()
})
