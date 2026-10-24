import { expect, it } from 'vitest'
import { validateSnippet } from './model'
import type { Library, Snippet } from '@/services/library'
const snippet: Snippet = {
  id: '20000000-0000-4000-8000-000000000000',
  folderId: null,
  title: 'Greeting',
  trigger: ';hello',
  body: 'Hello',
  enabled: true,
  deleted: false,
}
const library: Library = { schemaVersion: 1, revision: 0, folders: [], snippets: [snippet] }
it('permits whitespace-only text and counts Unicode scalars rather than UTF-16 units', () => {
  expect(validateSnippet({ ...snippet, body: '\t\r\n ' }, library)).toBeNull()
  expect(validateSnippet({ ...snippet, body: '👋'.repeat(16384) }, library)).toBeNull()
  expect(validateSnippet({ ...snippet, body: '👋'.repeat(16385) }, library)).toBe('body')
})
it.each(['\0', '\b', '\u001b', '\u007f', '\u0085'])(
  'refuses unsafe control text %j before a native save',
  (control) => {
    expect(validateSnippet({ ...snippet, body: `before${control}after` }, library)).toBe('body')
  },
)
