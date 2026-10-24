import {
  validSnippetBody,
  validSnippetTrigger,
  type Library,
  type Snippet,
} from '@/services/library'

export type SnippetValidation = 'title' | 'trigger' | 'body' | 'duplicateTrigger'
export function resolveFolderSelection(folders: Library['folders'], selected: string) {
  return selected === 'all' ||
    selected === 'unfiled' ||
    folders.some((folder) => folder.id === selected)
    ? selected
    : 'all'
}
export function validateSnippet(snippet: Snippet, library: Library): SnippetValidation | null {
  if (
    !snippet.title.trim() ||
    Array.from(snippet.title).length > 120 ||
    snippet.title.includes('\0')
  )
    return 'title'
  if (!validSnippetTrigger(snippet.trigger)) return 'trigger'
  if (!validSnippetBody(snippet.body)) return 'body'
  if (
    snippet.enabled &&
    !snippet.deleted &&
    library.snippets.some(
      (item) =>
        item.id !== snippet.id && item.enabled && !item.deleted && item.trigger === snippet.trigger,
    )
  )
    return 'duplicateTrigger'
  return null
}
export function searchSnippets(snippets: Snippet[], query: string) {
  const text = query.trim().toLocaleLowerCase()
  return snippets.filter(
    (item) =>
      !text ||
      [item.title, item.trigger, item.body].some((value) =>
        value.toLocaleLowerCase().includes(text),
      ),
  )
}
