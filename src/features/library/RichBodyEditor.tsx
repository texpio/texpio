import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { EditorContent, useEditor, useEditorState } from '@tiptap/react'
import { getSchema, type Editor, type JSONContent } from '@tiptap/core'
import type { EditorProps, EditorView } from '@tiptap/pm/view'
import { Fragment, Slice, type Node as ProseMirrorNode } from '@tiptap/pm/model'
import type { Selection } from '@tiptap/pm/state'
import StarterKit from '@tiptap/starter-kit'
import OrderedList from '@tiptap/extension-ordered-list'
import FindAndReplace from '@tiptap/extension-find-and-replace'
import { Bold, Italic, Underline, Link, List, ListOrdered } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { projectRichBody } from './rich-body-projection'

function safeHref(value: string): boolean {
  return (
    /^(https?:\/\/|mailto:)/i.test(value) &&
    new TextEncoder().encode(value).length <= 4096 &&
    !/\s/u.test(value) &&
    !Array.from(value).some((char) => {
      const code = char.codePointAt(0)!
      return code < 32 || (code >= 127 && code <= 159)
    })
  )
}

const extensions = [
  StarterKit.configure({
    blockquote: false,
    code: false,
    codeBlock: false,
    heading: false,
    horizontalRule: false,
    strike: false,
    trailingNode: false,
    paragraph: { HTMLAttributes: { dir: 'auto' } },
    listItem: { HTMLAttributes: { dir: 'auto' } },
    orderedList: false,
    link: { openOnClick: false, autolink: false, linkOnPaste: false, isAllowedUri: safeHref },
  }),
  OrderedList.extend({ addProseMirrorPlugins: () => [] }),
  FindAndReplace.configure({ caseSensitive: true, searchDebounceMs: 0, injectCSS: false }),
]
const schema = getSchema(extensions)

function plainTextSlice(text: string, view: EditorView): Slice {
  const { schema, selection } = view.state
  const paragraphs = text
    .split('\n')
    .map((line) =>
      schema.nodes.paragraph.create(null, line ? schema.text(line, selection.$from.marks()) : null),
    )
  return Slice.maxOpen(Fragment.fromArray(paragraphs))
}

function editorProps(
  id: string,
  label: string,
  readOnly: boolean,
  invalid: boolean,
  onCarriagePaste: (text: string, view: EditorView) => void,
): EditorProps {
  let parsedCarriagePaste: string | null = null
  return {
    attributes: {
      id,
      dir: 'auto',
      role: 'textbox',
      'aria-label': label,
      'aria-multiline': 'true',
      'aria-readonly': String(readOnly),
      'aria-invalid': String(invalid),
      class:
        'min-h-52 rounded-md border bg-background px-3 py-2 outline-none focus-visible:ring-3 focus-visible:ring-ring/50 whitespace-pre-wrap [&_ul]:list-disc [&_ol]:list-decimal [&_ul]:ps-6 [&_ol]:ps-6',
    },
    clipboardTextParser: (text, _context, _plain, view) => {
      parsedCarriagePaste = text.includes('\r') ? text : null
      return parsedCarriagePaste ? Slice.empty : plainTextSlice(text, view)
    },
    handlePaste: (view) => {
      const text = parsedCarriagePaste
      parsedCarriagePaste = null
      if (!text) return false
      onCarriagePaste(text, view)
      return true
    },
    handleDOMEvents: {
      paste: (view, event) => {
        parsedCarriagePaste = null
        const data = event.clipboardData
        const text = data?.getData('text/plain') ?? ''
        if (!data?.getData('text/html') && text.includes('\r')) {
          event.preventDefault()
          onCarriagePaste(text, view)
          return true
        }
        return false
      },
      click: (_view, event) => {
        if (event.target instanceof Element && event.target.closest('a')) {
          event.preventDefault()
          return true
        }
        return false
      },
    },
  }
}

function legacyDocument(body: string): JSONContent {
  return {
    type: 'doc',
    content: body.split('\n').map((line) => ({
      type: 'paragraph',
      ...(line ? { content: [{ type: 'text', text: line }] } : {}),
    })),
  }
}

function hasUnsafeLinks(node: JSONContent): boolean {
  return (
    (node.marks ?? []).some(
      (mark) => mark.type === 'link' && !safeHref(String(mark.attrs?.href ?? '')),
    ) || (node.content ?? []).some(hasUnsafeLinks)
  )
}

function contentFor(body: string, richBody: string | null): JSONContent {
  if (richBody) {
    try {
      const parsed: unknown = JSON.parse(richBody)
      if (
        typeof parsed === 'object' &&
        parsed !== null &&
        'version' in parsed &&
        parsed.version === 1 &&
        'doc' in parsed &&
        typeof parsed.doc === 'object' &&
        parsed.doc !== null &&
        'type' in parsed.doc &&
        parsed.doc.type === 'doc'
      ) {
        const doc = parsed.doc as JSONContent
        if (!hasUnsafeLinks(doc)) {
          schema.nodeFromJSON(doc).check()
          return doc
        }
      }
    } catch {
      /* fall back to literal body */
    }
  }
  return legacyDocument(body)
}

type RichBodyEditorProps = {
  body: string
  richBody: string | null
  onChange: (body: string, richBody: string | null) => void
  disabled?: boolean
  invalid?: boolean
  readOnlyPreview?: boolean
  onEditorChange?: (editor: Editor | null) => void
  id: string
  label: string
}

export function RichBodyEditor(props: RichBodyEditorProps) {
  return <RichBodyEditorInstance key={props.id} {...props} />
}

function RichBodyEditorInstance({
  body,
  richBody,
  onChange,
  disabled = false,
  invalid = false,
  readOnlyPreview = false,
  onEditorChange,
  id,
  label,
}: RichBodyEditorProps) {
  const { t } = useTranslation()
  const readOnly = disabled || readOnlyPreview || body.includes('\r')
  const [linkOpen, setLinkOpen] = useState(false)
  const [href, setHref] = useState('')
  const [pendingPaste, setPendingPaste] = useState<{
    text: string
    doc: ProseMirrorNode
    selection: Selection
  } | null>(null)
  const onCarriagePaste = useCallback((text: string, view: EditorView) => {
    setPendingPaste({ text, doc: view.state.doc, selection: view.state.selection })
  }, [])
  const emitted = useRef({ body, richBody, id })
  const editor = useEditor({
    extensions,
    enableInputRules: false,
    enablePasteRules: false,
    content: contentFor(body, richBody),
    editable: !readOnly,
    editorProps: editorProps(id, label, readOnly, invalid, onCarriagePaste),
    onUpdate: ({ editor: current }) => {
      const doc = current.getJSON()
      const nextBody = projectRichBody(doc)
      if (nextBody.includes('\r')) {
        current.setEditable(false, false)
        return
      }
      const nextRich = JSON.stringify({ version: 1, doc })
      emitted.current = { body: nextBody, richBody: nextRich, id }
      onChange(nextBody, nextRich)
    },
  })
  const state = useEditorState({
    editor,
    selector: ({ editor: current }) => ({
      bold: current?.isActive('bold') ?? false,
      italic: current?.isActive('italic') ?? false,
      underline: current?.isActive('underline') ?? false,
      bullet: current?.isActive('bulletList') ?? false,
      ordered: current?.isActive('orderedList') ?? false,
      link: current?.isActive('link') ?? false,
      doc: current?.state.doc ?? null,
      selection: current?.state.selection ?? null,
    }),
  })
  useEffect(() => {
    if (!editor) return
    editor.setEditable(!readOnly, false)
  }, [editor, readOnly])
  useEffect(() => {
    if (!editor || !onEditorChange) return
    onEditorChange(editor)
    return () => onEditorChange(null)
  }, [editor, onEditorChange])
  useEffect(() => {
    if (!editor) return
    editor.setOptions({
      editorProps: editorProps(id, label, readOnly, invalid, onCarriagePaste),
    })
  }, [editor, id, label, readOnly, invalid, onCarriagePaste])
  useEffect(() => {
    if (!editor) return
    const last = emitted.current
    if (last.id !== id || last.body !== body || last.richBody !== richBody) {
      editor.commands.setContent(contentFor(body, richBody), { emitUpdate: false })
      emitted.current = { id, body, richBody }
    }
  }, [editor, id, body, richBody])
  const controls = [
    {
      key: 'bold',
      icon: Bold,
      active: state?.bold,
      run: () => editor?.chain().focus().toggleBold().run(),
    },
    {
      key: 'italic',
      icon: Italic,
      active: state?.italic,
      run: () => editor?.chain().focus().toggleItalic().run(),
    },
    {
      key: 'underline',
      icon: Underline,
      active: state?.underline,
      run: () => editor?.chain().focus().toggleUnderline().run(),
    },
    {
      key: 'bulletList',
      icon: List,
      active: state?.bullet,
      run: () => editor?.chain().focus().toggleBulletList().run(),
    },
    {
      key: 'orderedList',
      icon: ListOrdered,
      active: state?.ordered,
      run: () => editor?.chain().focus().toggleOrderedList().run(),
    },
  ] as const
  return (
    <div className="flex shrink-0 flex-col gap-2">
      {pendingPaste && (
        <div role="alert" className="flex flex-wrap items-center gap-2">
          <span>{t('editor.pasteLineEndings')}</span>
          <Button
            type="button"
            size="sm"
            disabled={
              readOnly ||
              !editor ||
              state?.doc !== pendingPaste.doc ||
              !state?.selection?.eq(pendingPaste.selection)
            }
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => {
              if (
                !editor ||
                editor.state.doc !== pendingPaste.doc ||
                !editor.state.selection.eq(pendingPaste.selection)
              )
                return
              const slice = plainTextSlice(pendingPaste.text.replace(/\r\n?/g, '\n'), editor.view)
              editor.view.dispatch(editor.state.tr.replaceSelection(slice).scrollIntoView())
              setPendingPaste(null)
              editor.commands.focus()
            }}
          >
            {t('editor.pasteConvert')}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => setPendingPaste(null)}>
            {t('common.cancel')}
          </Button>
        </div>
      )}
      {!readOnlyPreview && (
        <div role="toolbar" aria-label={t('editor.formatting')} className="flex flex-wrap gap-1">
          {controls.map(({ key, icon: Icon, active, run }) => (
            <Button
              key={key}
              type="button"
              variant={active ? 'secondary' : 'ghost'}
              size="icon-sm"
              disabled={readOnly || !editor}
              aria-label={t(`editor.${key}`)}
              aria-pressed={active}
              onMouseDown={(event) => event.preventDefault()}
              onClick={run}
            >
              <Icon />
            </Button>
          ))}
          <Button
            type="button"
            variant={state?.link ? 'secondary' : 'ghost'}
            size="icon-sm"
            disabled={readOnly || !editor}
            aria-label={t('editor.link')}
            aria-pressed={state?.link}
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => {
              setHref(editor?.getAttributes('link').href ?? '')
              setLinkOpen(true)
            }}
          >
            <Link />
          </Button>
        </div>
      )}
      {!readOnlyPreview && linkOpen && (
        <div className="flex flex-wrap gap-2">
          <Input
            aria-label={t('editor.linkUrl')}
            value={href}
            onChange={(event) => setHref(event.target.value)}
            disabled={readOnly}
          />
          <Button
            type="button"
            size="sm"
            disabled={readOnly || !safeHref(href)}
            onClick={() => {
              editor?.chain().focus().extendMarkRange('link').setLink({ href }).run()
              setLinkOpen(false)
            }}
          >
            {t('editor.applyLink')}
          </Button>
          <Button
            type="button"
            size="sm"
            variant="outline"
            disabled={readOnly}
            onClick={() => {
              editor?.chain().focus().extendMarkRange('link').unsetLink().run()
              setLinkOpen(false)
            }}
          >
            {t('editor.removeLink')}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => setLinkOpen(false)}>
            {t('editor.close')}
          </Button>
        </div>
      )}
      <EditorContent editor={editor} />
    </div>
  )
}
