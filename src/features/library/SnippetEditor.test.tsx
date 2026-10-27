import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { SnippetEditor } from './SnippetEditor'

Range.prototype.getClientRects = () => [] as unknown as DOMRectList
Range.prototype.getBoundingClientRect = () => new DOMRect()
beforeEach(() =>
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  ),
)
const snippet = {
  id: 'snippet',
  folderId: null,
  title: 'Hello',
  trigger: ';hi',
  body: 'Hello',
  enabled: true,
  deleted: false,
}
const props = {
  snippet,
  original: snippet,
  folders: [],
  revision: 1,
  dirty: false,
  busy: false,
  stale: false,
  validation: null,
  favorite: false,
  favoritePending: false,
  findMode: null,
  onFind: vi.fn(),
  onEdit: vi.fn(),
  onSave: vi.fn(),
  onCopy: vi.fn(),
  onDuplicate: vi.fn(),
  onDelete: vi.fn(),
  onFavorite: vi.fn(),
}
it('provides formatting controls and updates text and rich document together', async () => {
  const onEdit = vi.fn()
  render(<SnippetEditor {...props} onEdit={onEdit} />)
  const body = await screen.findByRole('textbox', { name: 'Body' })
  body.focus()
  await userEvent.keyboard('{Control>}a{/Control}')
  await userEvent.click(screen.getByRole('button', { name: 'Bold' }))
  await waitFor(() =>
    expect(onEdit).toHaveBeenCalledWith(
      expect.objectContaining({ body: 'Hello', richBody: expect.any(String) }),
    ),
  )
  const change = onEdit.mock.lastCall![0]
  const saved = JSON.parse(change.richBody)
  expect(saved.doc.attrs).toBeUndefined()
  expect(saved.doc.content[0].attrs).toBeUndefined()
})
it('requires explicit conversion before editing legacy carriage-return text', async () => {
  const onEdit = vi.fn()
  render(
    <SnippetEditor {...props} onEdit={onEdit} snippet={{ ...snippet, body: 'Hello\r\nWorld' }} />,
  )
  const body = screen.getByLabelText('Body')
  expect(body.tagName).toBe('TEXTAREA')
  expect(body).toHaveAttribute('readonly')
  expect(screen.queryByRole('toolbar', { name: 'Formatting' })).not.toBeInTheDocument()
  await userEvent.click(screen.getByRole('button', { name: 'Convert line endings' }))
  expect(onEdit).not.toHaveBeenCalled()
  await userEvent.click(screen.getByRole('button', { name: 'Convert' }))
  expect(onEdit).toHaveBeenCalledWith({ body: 'Hello\nWorld', richBody: null })
})
