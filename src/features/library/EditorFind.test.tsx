import { act, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useRef, useState } from 'react'
import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import FindAndReplace from '@tiptap/extension-find-and-replace'
import { expect, it, vi } from 'vitest'
import { EditorFind } from './EditorFind'
it('replaces literal Unicode and dollar signs and can undo a replacement', async () => {
  function Fixture() {
    const [body, setBody] = useState('👋 a.b 👋 a.b')
    const ref = useRef<HTMLTextAreaElement>(null)
    return (
      <>
        <EditorFind
          mode="replace"
          value={body}
          onChange={setBody}
          textarea={ref}
          onClose={() => {}}
        />
        <textarea
          aria-label="Body"
          ref={ref}
          value={body}
          onChange={(event) => setBody(event.target.value)}
        />
      </>
    )
  }
  render(<Fixture />)
  await userEvent.type(screen.getByLabelText('Find in body'), '👋 a.b')
  await userEvent.type(screen.getByLabelText('Replace with'), '$&')
  await userEvent.click(screen.getByRole('button', { name: 'Replace all' }))
  expect(screen.getByLabelText('Body')).toHaveValue('$& $&')
  await userEvent.click(screen.getByRole('button', { name: 'Undo replacement' }))
  expect(screen.getByLabelText('Body')).toHaveValue('👋 a.b 👋 a.b')
})

function richEditor() {
  return new Editor({
    extensions: [
      StarterKit,
      FindAndReplace.configure({ caseSensitive: true, searchDebounceMs: 0, injectCSS: false }),
    ],
    content: {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'ab', marks: [{ type: 'bold' }] },
            { type: 'text', text: 'cd', marks: [{ type: 'italic' }] },
          ],
        },
      ],
    },
  })
}

it('replaces a match across styled runs and undo restores the original document', async () => {
  const editor = richEditor()
  const original = editor.getJSON()
  const onChange = vi.fn()
  render(
    <EditorFind
      mode="replace"
      value="abcd"
      onChange={onChange}
      textarea={{ current: null }}
      richMode
      richEditor={editor}
      onClose={() => {}}
    />,
  )
  await userEvent.type(screen.getByLabelText('Find in body'), 'bc')
  await userEvent.type(screen.getByLabelText('Replace with'), 'X')
  await userEvent.click(screen.getByRole('button', { name: 'Replace all' }))
  expect(editor.getText()).toBe('aXd')
  expect(editor.getJSON().content?.[0].content?.[0].marks).toContainEqual({ type: 'bold' })
  expect(editor.getJSON().content?.[0].content?.[1].marks).toContainEqual({ type: 'italic' })
  expect(onChange).not.toHaveBeenCalled()
  await userEvent.click(screen.getByRole('button', { name: 'Undo replacement' }))
  expect(editor.getJSON()).toEqual(original)
  editor.destroy()
})

it('reacts to document updates and clears search on unmount', async () => {
  const editor = richEditor()
  const { unmount } = render(
    <EditorFind
      mode="find"
      value="abcd"
      onChange={() => {}}
      textarea={{ current: null }}
      richMode
      richEditor={editor}
      onClose={() => {}}
    />,
  )
  await userEvent.type(screen.getByLabelText('Find in body'), 'bc')
  expect(screen.getByRole('button', { name: 'Next match' })).toBeEnabled()
  act(() => {
    editor.commands.clearContent()
  })
  expect(screen.getByRole('button', { name: 'Next match' })).toBeDisabled()
  unmount()
  expect(editor.storage.findAndReplace.searchTerm).toBe('')
  editor.destroy()
})

it('disables rich search while the editor is unavailable', async () => {
  render(
    <EditorFind
      mode="replace"
      value="abcd"
      onChange={() => {}}
      textarea={{ current: null }}
      richMode
      richEditor={null}
      onClose={() => {}}
    />,
  )
  await userEvent.type(screen.getByLabelText('Find in body'), 'ab')
  expect(screen.getByRole('button', { name: 'Next match' })).toBeDisabled()
  expect(screen.getByRole('button', { name: 'Replace all' })).toBeDisabled()
})

it('uses case-sensitive matches and selects the next structured result', async () => {
  const editor = richEditor()
  editor.view.setProps({ handleScrollToSelection: () => true })
  render(
    <EditorFind
      mode="find"
      value="abcd"
      onChange={() => {}}
      textarea={{ current: null }}
      richMode
      richEditor={editor}
      onClose={() => {}}
    />,
  )
  const find = screen.getByLabelText('Find in body')
  await userEvent.type(find, 'BC')
  expect(screen.getByRole('button', { name: 'Next match' })).toBeDisabled()
  await userEvent.clear(find)
  await userEvent.type(find, 'bc')
  await userEvent.click(screen.getByRole('button', { name: 'Next match' }))
  expect(editor.state.selection.from).toBe(2)
  expect(editor.state.selection.to).toBe(4)
  editor.destroy()
})

it('isolates replacement undo from earlier typing and disables it after later edits', async () => {
  const editor = richEditor()
  act(() => {
    editor.commands.insertContentAt(5, '!')
  })
  const before = editor.getJSON()
  render(
    <EditorFind
      mode="replace"
      value="abcd!"
      onChange={() => {}}
      textarea={{ current: null }}
      richMode
      richEditor={editor}
      onClose={() => {}}
    />,
  )
  await userEvent.type(screen.getByLabelText('Find in body'), 'bc')
  await userEvent.type(screen.getByLabelText('Replace with'), 'X')
  await userEvent.click(screen.getByRole('button', { name: 'Replace all' }))
  await userEvent.click(screen.getByRole('button', { name: 'Undo replacement' }))
  expect(editor.getJSON()).toEqual(before)
  await userEvent.click(screen.getByRole('button', { name: 'Replace all' }))
  expect(screen.getByRole('button', { name: 'Undo replacement' })).toBeEnabled()
  act(() => {
    editor.commands.insertContent('z')
  })
  expect(screen.getByRole('button', { name: 'Undo replacement' })).toBeDisabled()
  editor.destroy()
})

it('clears search for the previous editor when switching editors', async () => {
  const first = richEditor()
  const second = richEditor()
  const { rerender, unmount } = render(
    <EditorFind
      mode="find"
      value="abcd"
      onChange={() => {}}
      textarea={{ current: null }}
      richMode
      richEditor={first}
      onClose={() => {}}
    />,
  )
  await userEvent.type(screen.getByLabelText('Find in body'), 'bc')
  expect(first.storage.findAndReplace.searchTerm).toBe('bc')
  rerender(
    <EditorFind
      mode="find"
      value="abcd"
      onChange={() => {}}
      textarea={{ current: null }}
      richMode
      richEditor={second}
      onClose={() => {}}
    />,
  )
  expect(first.storage.findAndReplace.searchTerm).toBe('')
  expect(second.storage.findAndReplace.searchTerm).toBe('bc')
  unmount()
  expect(second.storage.findAndReplace.searchTerm).toBe('')
  first.destroy()
  second.destroy()
})
