import type { JSONContent } from '@tiptap/core'

export function projectRichBody(node: JSONContent): string {
  const children = node.content ?? []
  if (node.type === 'text') return node.text ?? ''
  if (node.type === 'hardBreak') return '\n'
  if (node.type === 'paragraph') return children.map(projectRichBody).join('')
  if (node.type === 'bulletList' || node.type === 'orderedList') {
    const start = typeof node.attrs?.start === 'number' ? node.attrs.start : 1
    return children
      .map((item, index) => {
        const lines = projectRichBody(item).split('\n')
        const prefix = node.type === 'bulletList' ? '- ' : `${start + index}. `
        return (
          prefix +
          lines[0] +
          lines
            .slice(1)
            .map((line) => `\n  ${line}`)
            .join('')
        )
      })
      .join('\n')
  }
  return children.map(projectRichBody).join('\n')
}
