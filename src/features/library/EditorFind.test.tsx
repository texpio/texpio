import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useRef, useState } from 'react'
import { expect, it } from 'vitest'
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
