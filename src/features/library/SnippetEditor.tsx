import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Copy, Files, MoreHorizontal, Search, Star, Trash2 } from 'lucide-react'
import type { Folder, Snippet } from '@/services/library'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Switch } from '@/components/ui/switch'
import { Textarea } from '@/components/ui/textarea'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { AttachmentEditor } from './AttachmentEditor'
import { EditorFind } from './EditorFind'
import type { SnippetValidation } from './model'

interface SnippetEditorProps {
  snippet: Snippet
  original: Snippet | null
  folders: Folder[]
  revision: number
  dirty: boolean
  busy: boolean
  stale: boolean
  validation: SnippetValidation | null
  favorite: boolean
  favoritePending: boolean
  findMode: 'find' | 'replace' | null
  onFind: (mode: 'find' | 'replace' | null) => void
  onEdit: (patch: Partial<Snippet>) => void
  onSave: () => void
  onCopy: () => void
  onDuplicate: () => void
  onDelete: () => void
  onFavorite: () => void
}
export function SnippetEditor({
  snippet,
  original,
  folders,
  revision,
  dirty,
  busy,
  stale,
  validation,
  favorite,
  favoritePending,
  findMode,
  onFind,
  onEdit,
  onSave,
  onCopy,
  onDuplicate,
  onDelete,
  onFavorite,
}: SnippetEditorProps) {
  const { t } = useTranslation()
  const title = useRef<HTMLInputElement>(null)
  const body = useRef<HTMLTextAreaElement>(null)
  const [section, setSection] = useState('content')
  const selectedSection = findMode ? 'content' : section
  const isNew = !original
  useEffect(() => {
    if (isNew) {
      title.current?.focus()
      title.current?.select()
    }
  }, [snippet.id, isNew])
  return (
    <form
      className="flex h-full min-h-0 flex-col"
      onSubmit={(event) => {
        event.preventDefault()
        onSave()
      }}
    >
      <header className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b px-5 py-3">
        <div className="flex min-w-0 items-center gap-2">
          <h2 className="truncate text-sm font-semibold">
            {isNew ? t('library.new') : snippet.title || t('workspace.untitled')}
          </h2>
          {dirty && <Badge variant="secondary">{t('library.unsaved')}</Badge>}
          {snippet.deleted && <Badge variant="outline">{t('library.inTrash')}</Badge>}
        </div>
        <div className="flex items-center gap-1">
          {original && (
            <Button
              type="button"
              size="icon-sm"
              variant="ghost"
              aria-label={t(favorite ? 'library.unfavorite' : 'library.favorite')}
              aria-pressed={favorite}
              disabled={busy || stale || favoritePending || snippet.deleted}
              onClick={onFavorite}
            >
              <Star className={favorite ? 'fill-current' : undefined} />
            </Button>
          )}
          <Button
            type="button"
            size="icon-sm"
            variant="ghost"
            aria-label={t('library.copy')}
            disabled={busy || !snippet.body}
            onClick={onCopy}
          >
            <Copy />
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                aria-label={t('workspace.snippetActions')}
                disabled={busy}
              >
                <MoreHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuGroup>
                <DropdownMenuItem disabled={busy} onSelect={onDuplicate}>
                  <Files />
                  {t('library.duplicate')}
                </DropdownMenuItem>
                {original && (
                  <DropdownMenuItem disabled={busy} onSelect={onDelete}>
                    <Trash2 />
                    {t(snippet.deleted ? 'library.restore' : 'library.moveToTrash')}
                  </DropdownMenuItem>
                )}
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
          {!snippet.deleted && (
            <Button type="submit" size="sm" disabled={busy || !dirty}>
              {t(busy ? 'library.saving' : 'library.save')}
            </Button>
          )}
        </div>
      </header>
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4 lg:p-5">
        <FieldGroup className="gap-3">
          <Field data-invalid={validation === 'title'}>
            <FieldLabel htmlFor="snippet-title">{t('library.field.title')}</FieldLabel>
            <Input
              id="snippet-title"
              dir="auto"
              ref={title}
              value={snippet.title}
              disabled={busy || snippet.deleted}
              aria-invalid={validation === 'title'}
              onChange={(event) => onEdit({ title: event.target.value })}
            />
            {validation === 'title' && <FieldError>{t('library.validation.title')}</FieldError>}
          </Field>
          <div className="grid gap-4 md:grid-cols-2">
            <Field data-invalid={validation === 'trigger' || validation === 'duplicateTrigger'}>
              <FieldLabel htmlFor="snippet-trigger">{t('library.field.trigger')}</FieldLabel>
              <Input
                id="snippet-trigger"
                dir="auto"
                className="font-mono"
                value={snippet.trigger}
                autoComplete="off"
                autoCapitalize="off"
                spellCheck={false}
                disabled={busy || snippet.deleted}
                aria-invalid={validation === 'trigger' || validation === 'duplicateTrigger'}
                onChange={(event) => onEdit({ trigger: event.target.value })}
              />
              {(validation === 'trigger' || validation === 'duplicateTrigger') && (
                <FieldError>{t(`library.validation.${validation}`)}</FieldError>
              )}
            </Field>
            <Field>
              <FieldLabel htmlFor="snippet-folder">{t('library.field.folder')}</FieldLabel>
              <NativeSelect
                id="snippet-folder"
                value={snippet.folderId ?? ''}
                disabled={busy || snippet.deleted}
                onChange={(event) => onEdit({ folderId: event.target.value || null })}
              >
                <NativeSelectOption value="">{t('library.unfiled')}</NativeSelectOption>
                {folders.map((folder) => (
                  <NativeSelectOption key={folder.id} value={folder.id}>
                    {folder.name}
                  </NativeSelectOption>
                ))}
              </NativeSelect>
            </Field>
          </div>
          {isNew && <p className="text-xs text-muted-foreground">{t('library.triggerHelp')}</p>}
          <Field orientation="horizontal">
            <FieldLabel htmlFor="snippet-enabled">{t('library.enabled')}</FieldLabel>
            <Switch
              id="snippet-enabled"
              checked={snippet.enabled}
              disabled={busy || snippet.deleted}
              onCheckedChange={(enabled) => onEdit({ enabled })}
            />
          </Field>
        </FieldGroup>
        <ToggleGroup
          type="single"
          value={selectedSection}
          variant="outline"
          size="sm"
          aria-label={t('workspace.snippetSections')}
          onValueChange={(value) => {
            if (value) {
              onFind(null)
              setSection(value)
            }
          }}
        >
          <ToggleGroupItem value="content" className="flex-none px-3">
            {t('workspace.content')}
          </ToggleGroupItem>
          <ToggleGroupItem value="preview" className="flex-none px-3">
            {t('workspace.preview')}
          </ToggleGroupItem>
          <ToggleGroupItem value="attachments" className="flex-none px-3">
            {t('attachments.title')}
          </ToggleGroupItem>
        </ToggleGroup>
        {validation === 'body' && selectedSection !== 'content' && (
          <FieldError>{t('library.validation.body')}</FieldError>
        )}
        {selectedSection === 'content' && (
          <div className="flex min-h-0 flex-1 flex-col gap-3">
            <div className="flex items-center justify-between gap-2">
              <FieldLabel htmlFor="snippet-body">{t('library.field.body')}</FieldLabel>
              <div className="flex gap-1">
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  disabled={busy || snippet.deleted}
                  onClick={() => onFind('find')}
                >
                  <Search data-icon="inline-start" />
                  {t('editor.find')}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  disabled={busy || snippet.deleted}
                  onClick={() => onFind('replace')}
                >
                  {t('editor.findReplace')}
                </Button>
              </div>
            </div>
            {findMode && (
              <EditorFind
                key={snippet.id}
                mode={findMode}
                value={snippet.body}
                onChange={(value) => onEdit({ body: value })}
                textarea={body}
                onClose={() => onFind(null)}
              />
            )}
            <Field data-invalid={validation === 'body'} className="min-h-0 flex-1">
              <Textarea
                id="snippet-body"
                dir="auto"
                ref={body}
                value={snippet.body}
                rows={10}
                className="min-h-52 flex-1 resize-y font-mono"
                disabled={busy || snippet.deleted}
                aria-invalid={validation === 'body'}
                onChange={(event) => onEdit({ body: event.target.value })}
                onKeyDown={(event) => {
                  if (event.key === 'Escape' && findMode) {
                    event.preventDefault()
                    onFind(null)
                  }
                }}
              />
              <FieldDescription>{t('library.bodyHelp')}</FieldDescription>
              {validation === 'body' && <FieldError>{t('library.validation.body')}</FieldError>}
            </Field>
          </div>
        )}
        {selectedSection === 'preview' && (
          <section
            aria-label={t('library.preview')}
            className="min-h-52 rounded-md border bg-muted/40 p-4"
          >
            <pre
              dir="auto"
              className="whitespace-pre-wrap break-words font-sans text-sm leading-relaxed"
            >
              {snippet.body || t('library.previewEmpty')}
            </pre>
          </section>
        )}
        {selectedSection === 'attachments' &&
          (original ? (
            <AttachmentEditor
              id={snippet.id}
              revision={revision}
              disabled={busy || stale || snippet.deleted}
            />
          ) : (
            <p className="text-sm text-muted-foreground">{t('attachments.saveFirst')}</p>
          ))}
      </div>
    </form>
  )
}
