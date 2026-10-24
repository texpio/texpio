import { useEffect, useRef, useState, type RefObject } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Field, FieldLabel } from '@/components/ui/field'
export function EditorFind({
  mode,
  value,
  onChange,
  textarea,
  onClose,
}: {
  mode: 'find' | 'replace'
  value: string
  onChange: (value: string) => void
  textarea: RefObject<HTMLTextAreaElement | null>
  onClose: () => void
}) {
  const { t } = useTranslation()
  const [query, setQuery] = useState('')
  const [replacement, setReplacement] = useState('')
  const [undo, setUndo] = useState<{ before: string; after: string } | null>(null)
  const input = useRef<HTMLInputElement>(null)
  useEffect(() => {
    input.current?.focus()
  }, [mode])
  function next() {
    if (!query) return -1
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
  return (
    <section
      aria-label={t('editor.findReplace')}
      className="flex flex-col gap-3 rounded-md border p-3"
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          event.preventDefault()
          onClose()
          textarea.current?.focus()
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
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="outline"
          disabled={!query || !value.includes(query)}
          onClick={next}
        >
          {t('editor.next')}
        </Button>
        {mode === 'replace' && (
          <>
            <Button
              type="button"
              variant="outline"
              disabled={!query || !value.includes(query)}
              onClick={() => replace(false)}
            >
              {t('editor.replaceNext')}
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!query || !value.includes(query)}
              onClick={() => replace(true)}
            >
              {t('editor.replaceAll')}
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!undo || undo.after !== value}
              onClick={() => {
                if (undo) {
                  onChange(undo.before)
                  setUndo(null)
                }
              }}
            >
              {t('editor.undo')}
            </Button>
          </>
        )}
        <Button type="button" variant="ghost" onClick={onClose}>
          {t('editor.close')}
        </Button>
      </div>
    </section>
  )
}
