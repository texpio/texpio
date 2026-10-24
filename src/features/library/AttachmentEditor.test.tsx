import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, expect, it, vi } from 'vitest'
import { AttachmentEditor } from './AttachmentEditor'

const native = vi.hoisted(() => ({
  list: vi.fn(),
  settings: vi.fn(),
  add: vi.fn(),
  remove: vi.fn(),
  reorder: vi.fn(),
  save: vi.fn(),
  open: vi.fn(),
}))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: native.open }))
vi.mock('@/lib/bindings', () => ({
  commands: {
    attachmentList: native.list,
    attachmentSettings: native.settings,
    attachmentAdd: native.add,
    attachmentRemove: native.remove,
    attachmentReorder: native.reorder,
    attachmentSaveSettings: native.save,
  },
}))
const files = ['one.txt', 'two.png'].map((name, index) => ({
  id: `file-${index}`,
  snippetId: 'snippet',
  name,
  digest: 'a'.repeat(64),
  bytes: 12,
  position: index,
  present: index === 0,
}))
beforeEach(() => {
  vi.clearAllMocks()
  native.list.mockResolvedValue({ status: 'ok', data: files })
  native.settings.mockResolvedValue({
    status: 'ok',
    data: { attachmentsFirst: false, strictOrder: false },
  })
  native.open.mockResolvedValue(null)
  for (const operation of [native.remove, native.reorder, native.save])
    operation.mockResolvedValue({ status: 'ok', data: null })
  native.add.mockResolvedValue({ status: 'ok', data: files })
})
function show() {
  render(
    <QueryClientProvider
      client={
        new QueryClient({
          defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
        })
      }
    >
      <AttachmentEditor id="snippet" revision={4} disabled={false} />
    </QueryClientProvider>,
  )
}
it('does not send cancelled file selection and preserves native order and missing-file state', async () => {
  show()
  await screen.findByText('two.png')
  expect(screen.getByText('Missing file')).toBeInTheDocument()
  await userEvent.click(screen.getByRole('button', { name: 'Add files' }))
  expect(native.add).not.toHaveBeenCalled()
  await userEvent.click(screen.getByRole('button', { name: 'Move two.png up' }))
  await waitFor(() =>
    expect(native.reorder).toHaveBeenCalledWith('snippet', 4, ['file-1', 'file-0']),
  )
  await userEvent.click(screen.getByRole('button', { name: 'Remove one.txt' }))
  await waitFor(() => expect(native.remove).toHaveBeenCalledWith('snippet', 4, 'file-0'))
})
it('uses only selected paths and saves ordering independently of the text draft', async () => {
  native.open.mockResolvedValue(['C:\\fixture\\one.txt'])
  show()
  await screen.findByText('one.txt')
  await userEvent.click(screen.getByRole('button', { name: 'Add files' }))
  await waitFor(() =>
    expect(native.add).toHaveBeenCalledWith('snippet', 4, ['C:\\fixture\\one.txt']),
  )
  await userEvent.click(screen.getByRole('switch', { name: 'Files before text' }))
  await waitFor(() =>
    expect(native.save).toHaveBeenCalledWith('snippet', 4, {
      attachmentsFirst: true,
      strictOrder: false,
    }),
  )
})
