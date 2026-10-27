import { StrictMode } from 'react'
import { act, fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, it, vi } from 'vitest'
import { RichBodyEditor } from './RichBodyEditor'
import { projectRichBody } from './rich-body-projection'

// jsdom does not implement the measurements used by ProseMirror scrolling.
Range.prototype.getClientRects = () => [] as unknown as DOMRectList
Range.prototype.getBoundingClientRect = () => new DOMRect()

it('opens legacy text literally without emitting a change', async () => {
  const onChange = vi.fn()
  const body = '<b>literal</b>\r\n\tनमस्ते 👋'
  render(
    <StrictMode>
      <RichBodyEditor id="body" label="Body" body={body} richBody={null} onChange={onChange} />
    </StrictMode>,
  )
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  expect(editor).toHaveTextContent('<b>literal</b>')
  expect(editor.querySelector('b')).toBeNull()
  editor.focus()
  expect(onChange).not.toHaveBeenCalled()
})

it('projects lists and hard breaks deterministically', () => {
  expect(
    projectRichBody({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'Hi' },
            { type: 'hardBreak' },
            { type: 'text', text: 'there' },
          ],
        },
        {
          type: 'orderedList',
          attrs: { start: 3 },
          content: [
            {
              type: 'listItem',
              content: [
                { type: 'paragraph', content: [{ type: 'text', text: 'one' }] },
                {
                  type: 'bulletList',
                  content: [
                    {
                      type: 'listItem',
                      content: [{ type: 'paragraph', content: [{ type: 'text', text: 'nested' }] }],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ],
    }),
  ).toBe('Hi\nthere\n3. one\n  - nested')
})

it('preserves plain-text ordered-list punctuation and blank lines on paste', async () => {
  const onChange = vi.fn()
  render(<RichBodyEditor id="body" label="Body" body="" richBody={null} onChange={onChange} />)
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  editor.focus()
  fireEvent.paste(editor, {
    clipboardData: {
      getData: (type: string) => (type === 'text/plain' ? '1) first\n\n2) second' : ''),
      types: ['text/plain'],
    },
  })
  expect(onChange.mock.lastCall?.[0]).toBe('1) first\n\n2) second')
})

it('preserves formatting when pasting an HTML ordered list', async () => {
  const onChange = vi.fn()
  render(<RichBodyEditor id="body" label="Body" body="" richBody={null} onChange={onChange} />)
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  editor.focus()
  fireEvent.paste(editor, {
    clipboardData: {
      getData: (type: string) =>
        type === 'text/html'
          ? '<ol><li><p><strong>first</strong></p></li></ol>'
          : type === 'text/plain'
            ? 'first\r\n'
            : '',
      types: ['text/html', 'text/plain'],
    },
  })
  expect(onChange.mock.lastCall?.[0]).toBe('1. first')
  expect(editor.querySelector('ol strong')).toHaveTextContent('first')
  expect(screen.queryByRole('alert')).not.toBeInTheDocument()
})

it('requires confirmation before converting pasted carriage returns', async () => {
  const onChange = vi.fn()
  render(<RichBodyEditor id="body" label="Body" body="" richBody={null} onChange={onChange} />)
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  editor.focus()
  fireEvent.paste(editor, {
    clipboardData: {
      getData: (type: string) => (type === 'text/plain' ? 'first\r\n\rsecond' : ''),
      types: ['text/plain'],
    },
  })
  expect(onChange).not.toHaveBeenCalled()
  expect(screen.getByRole('alert')).toBeInTheDocument()
  await userEvent.click(
    screen.getByRole('button', { name: /Convert and paste|editor.pasteConvert/ }),
  )
  expect(onChange.mock.lastCall?.[0]).toBe('first\n\nsecond')
})

it('guards plain-mode paste with HTML and CRLF before replacing the selection', async () => {
  const onChange = vi.fn()
  const onEditorChange = vi.fn()
  render(
    <RichBodyEditor
      id="body"
      label="Body"
      body="Keep"
      richBody={null}
      onChange={onChange}
      onEditorChange={onEditorChange}
    />,
  )
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  const instance = onEditorChange.mock.calls.find(([value]) => value)?.[0]
  editor.focus()
  act(() => instance.commands.setTextSelection({ from: 1, to: 5 }))
  fireEvent.keyDown(editor, { key: 'v', keyCode: 86, ctrlKey: true, shiftKey: true })
  fireEvent.paste(editor, {
    clipboardData: {
      getData: (type: string) =>
        type === 'text/plain' ? 'first\r\nsecond' : type === 'text/html' ? '<b>first</b>' : '',
      types: ['text/plain', 'text/html'],
    },
  })
  expect(onChange).not.toHaveBeenCalled()
  expect(editor).toHaveTextContent('Keep')
  expect(instance.state.selection.from).toBe(1)
  expect(instance.state.selection.to).toBe(5)
  expect(screen.getByRole('alert')).toBeInTheDocument()
  await userEvent.click(
    screen.getByRole('button', { name: /Convert and paste|editor.pasteConvert/ }),
  )
  expect(onChange.mock.lastCall?.[0]).toBe('first\nsecond')
})

it('cancels a carriage-return paste without changing the body', async () => {
  const onChange = vi.fn()
  render(<RichBodyEditor id="body" label="Body" body="Keep" richBody={null} onChange={onChange} />)
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  editor.focus()
  fireEvent.paste(editor, {
    clipboardData: {
      getData: (type: string) => (type === 'text/plain' ? 'first\r\nsecond' : ''),
      types: ['text/plain'],
    },
  })
  await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
  expect(onChange).not.toHaveBeenCalled()
  expect(editor).toHaveTextContent('Keep')
  expect(screen.queryByRole('alert')).not.toBeInTheDocument()
})

it('cannot convert a carriage-return paste after the document changes', async () => {
  const onChange = vi.fn()
  const onEditorChange = vi.fn()
  render(
    <RichBodyEditor
      id="body"
      label="Body"
      body="Keep"
      richBody={null}
      onChange={onChange}
      onEditorChange={onEditorChange}
    />,
  )
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  editor.focus()
  fireEvent.paste(editor, {
    clipboardData: {
      getData: (type: string) => (type === 'text/plain' ? 'first\r\nsecond' : ''),
      types: ['text/plain'],
    },
  })
  const instance = onEditorChange.mock.calls.find(([value]) => value)?.[0]
  act(() => instance.commands.insertContent('!'))
  expect(
    screen.getByRole('button', { name: /Convert and paste|editor.pasteConvert/ }),
  ).toBeDisabled()
  expect(onChange.mock.lastCall?.[0]).not.toContain('first')
})

it('formats a selection and emits a versioned document with the same plain text', async () => {
  const onChange = vi.fn()
  render(<RichBodyEditor id="body" label="Body" body="Hello" richBody={null} onChange={onChange} />)
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  editor.focus()
  await userEvent.keyboard('{Control>}a{/Control}')
  await userEvent.click(screen.getByRole('button', { name: 'Bold' }))
  expect(onChange).toHaveBeenCalled()
  const [plain, rich] = onChange.mock.lastCall!
  expect(plain).toBe('Hello')
  expect(JSON.parse(rich)).toMatchObject({
    version: 1,
    doc: { type: 'doc', content: [{ content: [{ marks: [{ type: 'bold' }] }] }] },
  })
})

it('fails closed for bodies containing carriage returns', async () => {
  const body = 'Hello\r\n\tनमस्ते 👋'
  const onChange = vi.fn()
  render(<RichBodyEditor id="body" label="Body" body={body} richBody={null} onChange={onChange} />)
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  expect(editor).toHaveAttribute('contenteditable', 'false')
  expect(screen.getByRole('button', { name: 'Bold' })).toBeDisabled()
  editor.focus()
  await userEvent.keyboard('X')
  expect(onChange).not.toHaveBeenCalled()
})

it('disables editing and formatting', async () => {
  render(
    <RichBodyEditor
      id="body"
      label="Body"
      body="Hello"
      richBody={null}
      onChange={() => {}}
      disabled
    />,
  )
  expect(await screen.findByRole('textbox', { name: 'Body' })).toHaveAttribute(
    'contenteditable',
    'false',
  )
  expect(screen.getByRole('button', { name: 'Bold' })).toBeDisabled()
})

it('offers an accessible read-only preview without formatting controls', async () => {
  render(
    <RichBodyEditor
      id="preview"
      label="Preview"
      body="Hello"
      richBody={null}
      onChange={() => {}}
      readOnlyPreview
    />,
  )
  expect(await screen.findByRole('textbox', { name: 'Preview' })).toHaveAttribute(
    'contenteditable',
    'false',
  )
  expect(screen.queryByRole('toolbar')).not.toBeInTheDocument()
})

it('exposes the current editor and clears it on identity change and unmount in StrictMode', async () => {
  const onEditorChange = vi.fn()
  const onChange = vi.fn()
  const { rerender, unmount } = render(
    <StrictMode>
      <RichBodyEditor
        id="first"
        label="Body"
        body="One"
        richBody={null}
        onChange={onChange}
        onEditorChange={onEditorChange}
      />
    </StrictMode>,
  )
  await screen.findByRole('textbox', { name: 'Body' })
  const first = onEditorChange.mock.calls.find(([value]) => value)?.[0]
  expect(first).toBeTruthy()
  rerender(
    <StrictMode>
      <RichBodyEditor
        id="second"
        label="Body"
        body="Two"
        richBody={null}
        onChange={onChange}
        onEditorChange={onEditorChange}
      />
    </StrictMode>,
  )
  const editors = onEditorChange.mock.calls.map(([value]) => value).filter(Boolean)
  expect(editors[editors.length - 1]).not.toBe(first)
  expect(onEditorChange.mock.calls.some(([value]) => value === null)).toBe(true)
  unmount()
  expect(onEditorChange.mock.lastCall?.[0]).toBeNull()
})

it('rejects unsafe link entry and does not render unsafe links from saved documents', async () => {
  const richBody = JSON.stringify({
    version: 1,
    doc: {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            {
              type: 'text',
              text: 'Hello',
              marks: [{ type: 'link', attrs: { href: 'javascript:alert(1)' } }],
            },
          ],
        },
      ],
    },
  })
  render(
    <StrictMode>
      <RichBodyEditor id="body" label="Body" body="Hello" richBody={richBody} onChange={() => {}} />
    </StrictMode>,
  )
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  expect(editor.querySelector('a')).toBeNull()
  await userEvent.click(screen.getByRole('button', { name: 'Link' }))
  await userEvent.type(screen.getByLabelText('Link URL'), 'javascript:alert(1)')
  expect(screen.getByRole('button', { name: 'Apply link' })).toBeDisabled()
})

it('rejects Unicode whitespace, C1 controls and links over the UTF-8 byte limit', async () => {
  render(<RichBodyEditor id="body" label="Body" body="Hello" richBody={null} onChange={() => {}} />)
  await screen.findByRole('textbox', { name: 'Body' })
  await userEvent.click(screen.getByRole('button', { name: 'Link' }))
  const input = screen.getByLabelText('Link URL')
  const apply = screen.getByRole('button', { name: 'Apply link' })
  for (const href of [
    'https://example.com/\u2003',
    'https://example.com/\u0085',
    `https://example.com/${'é'.repeat(2040)}`,
  ]) {
    fireEvent.change(input, { target: { value: href } })
    expect(apply).toBeDisabled()
  }
  const prefix = 'https://example.com/'
  fireEvent.change(input, { target: { value: prefix + 'a'.repeat(4096 - prefix.length) } })
  expect(apply).toBeEnabled()
  fireEvent.change(input, { target: { value: prefix + 'a'.repeat(4097 - prefix.length) } })
  expect(apply).toBeDisabled()
})

it('creates a safe link without navigating when it is clicked', async () => {
  const onChange = vi.fn()
  render(<RichBodyEditor id="body" label="Body" body="Hello" richBody={null} onChange={onChange} />)
  const editor = await screen.findByRole('textbox', { name: 'Body' })
  editor.focus()
  await userEvent.keyboard('{Control>}a{/Control}')
  await userEvent.click(screen.getByRole('button', { name: 'Link' }))
  await userEvent.type(screen.getByLabelText('Link URL'), 'https://example.com')
  await userEvent.click(screen.getByRole('button', { name: 'Apply link' }))
  const link = editor.querySelector('a')!
  expect(link).toHaveAttribute('href', 'https://example.com')
  const click = new MouseEvent('click', { bubbles: true, cancelable: true })
  fireEvent(link, click)
  expect(click.defaultPrevented).toBe(true)
})

it('replaces a case-sensitive match without dropping its formatting', async () => {
  const doc = {
    type: 'doc',
    content: [
      { type: 'paragraph', content: [{ type: 'text', text: 'Hello', marks: [{ type: 'bold' }] }] },
    ],
  }
  const onEditorChange = vi.fn()
  const onChange = vi.fn()
  render(
    <RichBodyEditor
      id="body"
      label="Body"
      body="Hello"
      richBody={JSON.stringify({ version: 1, doc })}
      onChange={onChange}
      onEditorChange={onEditorChange}
    />,
  )
  await screen.findByRole('textbox', { name: 'Body' })
  const editor = onEditorChange.mock.calls.find(([value]) => value)?.[0]
  act(() => {
    editor.commands.setSearchTerm('ell')
    editor.commands.setReplaceTerm('ipp')
    editor.commands.replaceAll()
  })
  expect(onChange.mock.lastCall?.[0]).toBe('Hippo')
  expect(JSON.parse(onChange.mock.lastCall?.[1]).doc.content[0].content[0].marks).toContainEqual({
    type: 'bold',
  })
})

it('changes document when the identity changes', async () => {
  const onChange = vi.fn()
  const { rerender } = render(
    <RichBodyEditor id="first" label="Body" body="One" richBody={null} onChange={onChange} />,
  )
  expect(await screen.findByRole('textbox', { name: 'Body' })).toHaveTextContent('One')
  rerender(
    <RichBodyEditor id="second" label="Body" body="Two" richBody={null} onChange={onChange} />,
  )
  expect(await screen.findByRole('textbox', { name: 'Body' })).toHaveTextContent('Two')
  expect(onChange).not.toHaveBeenCalled()
})

it('falls back to literal text for an invalid saved document', async () => {
  const onChange = vi.fn()
  const richBody = JSON.stringify({
    version: 1,
    doc: { type: 'doc', content: [{ type: 'unknown' }] },
  })
  render(
    <RichBodyEditor
      id="body"
      label="Body"
      body="Keep me"
      richBody={richBody}
      onChange={onChange}
    />,
  )
  expect(await screen.findByRole('textbox', { name: 'Body' })).toHaveTextContent('Keep me')
  expect(onChange).not.toHaveBeenCalled()
})
