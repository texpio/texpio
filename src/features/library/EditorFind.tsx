import { useEffect, useRef, useState, type RefObject } from 'react'
import { useTranslation } from 'react-i18next'
import type { Editor } from '@tiptap/core'
import { useEditorState } from '@tiptap/react'
import { FindAndReplacePluginKey, findNextIndex } from '@tiptap/extension-find-and-replace'
import type { Node as ProseMirrorNode } from '@tiptap/pm/model'
import { closeHistory } from '@tiptap/pm/history'
import { TextSelection } from '@tiptap/pm/state'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
export function EditorFind({
  mode,
  value,
  onChange,
  textarea,
  richMode = false,
  richEditor = null,
  onClose,
}: {
  mode: 'find' | 'replace'
  value: string
  onChange: (value: string) => void
  textarea: RefObject<HTMLTextAreaElement | null>
  richMode?: boolean
  richEditor?: Editor | null
  onClose: () => void
}) {
  const { t } = useTranslation()
  const [query, setQuery] = useState('')
  const [replacement, setReplacement] = useState('')
  const [undo, setUndo] = useState<{ before: string; after: string } | null>(null)
  const [richUndoDoc, setRichUndoDoc] = useState<ProseMirrorNode | null>(null)
  const input = useRef<HTMLInputElement>(null)
  const selectedEditor = richMode ? richEditor : null
  const editorState = useEditorState({
    editor: selectedEditor,
    selector: ({ editor }) => ({
      doc: editor?.state.doc ?? null,
      matches: editor?.storage.findAndReplace?.results.length ?? 0,
      editable: editor?.isEditable ?? false,
    }),
  })
  useEffect(() => {
    input.current?.focus()
  }, [mode])
  useEffect(() => {
    if (!selectedEditor || selectedEditor.isDestroyed) return
    selectedEditor.commands.setSearchTerm(query)
  }, [selectedEditor, query])
  useEffect(() => {
    if (!selectedEditor || selectedEditor.isDestroyed) return
    selectedEditor.commands.setReplaceTerm(replacement)
  }, [selectedEditor, replacement])
  useEffect(() => {
    if (!selectedEditor) return
    return () => {
      if (!selectedEditor.isDestroyed) selectedEditor.commands.clearSearch()
    }
  }, [selectedEditor])

  function selectNextRich() {
    if (!selectedEditor || selectedEditor.isDestroyed) return false
    const plugin = FindAndReplacePluginKey.getState(selectedEditor.state)
    if (!plugin) return false
    const index = findNextIndex(plugin.results, selectedEditor.state.selection.to)
    if (index === null) return false
    const result = plugin.results[index]
    const tr = selectedEditor.state.tr
      .setSelection(TextSelection.create(selectedEditor.state.doc, result.from, result.to))
      .setMeta(FindAndReplacePluginKey, { currentIndex: index })
      .scrollIntoView()
    selectedEditor.view.dispatch(tr)
    return true
  }
  function next() {
    if (!query) return -1
    if (richMode) {
      if (selectNextRich()) selectedEditor?.commands.focus()
      return -1
    }
    const start = textarea.current?.selectionEnd ?? 0
    let index = value.indexOf(query, start)
    if (index < 0) index = value.indexOf(query)
    if (index >= 0) {
      textarea.current?.focus()
      textarea.current?.setSelectionRange(index, index + query.length)
    }
    return index
  }
  function replace(all: boolean) {
    if (!query) return
    if (richMode) {
      if (!selectedEditor || selectedEditor.isDestroyed || !selectedEditor.isEditable) return
      if (!all) {
        const plugin = FindAndReplacePluginKey.getState(selectedEditor.state)
        const current = plugin?.results[plugin.currentIndex ?? 0]
        const selection = selectedEditor.state.selection
        if (!current || selection.from !== current.from || selection.to !== current.to) {
          if (!selectNextRich()) return
        }
      }
      const before = selectedEditor.state.doc
      selectedEditor.view.dispatch(closeHistory(selectedEditor.state.tr))
      if (all) selectedEditor.commands.replaceAll()
      else selectedEditor.commands.replace()
      selectedEditor.view.dispatch(closeHistory(selectedEditor.state.tr))
      if (selectedEditor.state.doc !== before) setRichUndoDoc(selectedEditor.state.doc)
      return
    }
    let after: string
    if (all) after = value.split(query).join(replacement)
    else {
      const selected = textarea.current
      const start = selected?.selectionStart ?? 0
      const index = selected && value.slice(start, selected.selectionEnd) === query ? start : next()
      if (index < 0) return
      after = value.slice(0, index) + replacement + value.slice(index + query.length)
    }
    if (after !== value) {
      setUndo({ before: value, after })
      onChange(after)
    }
  }
  const hasMatches = richMode ? !!selectedEditor && !!editorState?.matches : value.includes(query)
  const canReplace = hasMatches && (!richMode || !!editorState?.editable)
  const canUndo = richMode
    ? !!selectedEditor &&
      !!richUndoDoc &&
      editorState?.doc === richUndoDoc &&
      !!editorState.editable
    : !!undo && undo.after === value
  function close() {
    if (selectedEditor && !selectedEditor.isDestroyed) {
      selectedEditor.commands.clearSearch()
      selectedEditor.commands.focus()
    } else if (!richMode) textarea.current?.focus()
    onClose()
  }
  return (
    <section
      aria-label={t('editor.findReplace')}
      className="flex flex-col gap-3 rounded-md border p-3"
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          event.preventDefault()
          close()
        }
      }}
    >
      <Field>
        <FieldLabel htmlFor="editor-find">{t('editor.find')}</FieldLabel>
        <Input
          id="editor-find"
          ref={input}
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
      </Field>
      {mode === 'replace' && (
        <Field>
          <FieldLabel htmlFor="editor-replace">{t('editor.replacement')}</FieldLabel>
          <Input
            id="editor-replace"
            value={replacement}
            onChange={(event) => setReplacement(event.target.value)}
          />
        </Field>
      )}
      {richMode && <FieldDescription>{t('editor.richFindHelp')}</FieldDescription>}
      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" disabled={!query || !hasMatches} onClick={next}>
          {t('editor.next')}
        </Button>
        {mode === 'replace' && (
          <>
            <Button
              type="button"
              variant="outline"
              disabled={!query || !canReplace}
              onClick={() => replace(false)}
            >
              {t('editor.replaceNext')}
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!query || !canReplace}
              onClick={() => replace(true)}
            >
              {t('editor.replaceAll')}
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!canUndo}
              onClick={() => {
                if (richMode && selectedEditor && canUndo) {
                  selectedEditor.commands.undo()
                  setRichUndoDoc(null)
                } else if (!richMode && undo) {
                  onChange(undo.before)
                  setUndo(null)
                }
              }}
            >
              {t('editor.undo')}
            </Button>
          </>
        )}
        <Button type="button" variant="ghost" onClick={close}>
          {t('editor.close')}
        </Button>
      </div>
    </section>
  )
}
