import { useEffect, useRef, useState } from 'react'
import type { Editor } from '@tiptap/core'
import { useTranslation } from 'react-i18next'
import { Copy, Files, MoreHorizontal, Search, Star, Trash2, TriangleAlert } from 'lucide-react'
import type { Folder, Snippet } from '@/services/library'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertTitle } from '@/components/ui/alert'
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Switch } from '@/components/ui/switch'
import { Textarea } from '@/components/ui/textarea'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
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
import { RichBodyEditor } from './RichBodyEditor'
import type { SnippetValidation } from './model'

interface SnippetEditorProps {
  snippet: Snippet
  original: Snippet | null
  folders: Folder[]
  revision: number
  dirty: boolean
  busy: boolean
  saving: boolean
  stale: boolean
  validation: SnippetValidation | null
  favorite: boolean
  favoritePending: boolean
  findMode: 'find' | 'replace' | null
  onFind: (mode: 'find' | 'replace' | null) => void
  onEdit: (patch: Partial<Snippet>) => void
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
  saving,
  stale,
  validation,
  favorite,
  favoritePending,
  findMode,
  onFind,
  onEdit,
  onCopy,
  onDuplicate,
  onDelete,
  onFavorite,
}: SnippetEditorProps) {
  const { t } = useTranslation()
  const title = useRef<HTMLInputElement>(null)
  const body = useRef<HTMLTextAreaElement>(null)
  const [richEditor, setRichEditor] = useState<Editor | null>(null)
  const richMode = !snippet.body.includes('\r')
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
    <form className="flex h-full min-h-0 flex-col" onSubmit={(event) => event.preventDefault()}>
      <header className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b px-5 py-3">
        <div className="flex min-w-0 items-center gap-2">
          <h2 className="truncate text-sm font-semibold">
            {isNew ? t('library.new') : snippet.title || t('workspace.untitled')}
          </h2>
          {(dirty || saving) && (
            <Badge variant="secondary">{t(saving ? 'library.saving' : 'library.unsaved')}</Badge>
          )}
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
                <DropdownMenuItem disabled={busy} onSelect={onDelete}>
                  <Trash2 />
                  {t(
                    !original
                      ? 'library.discard'
                      : snippet.deleted
                        ? 'library.restore'
                        : 'library.moveToTrash',
                  )}
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>
      {!snippet.enabled && !snippet.deleted && (
        <Alert
          id="snippet-disabled-warning"
          variant="warning"
          className="shrink-0 rounded-none border-x-0 border-t-0"
        >
          <TriangleAlert aria-hidden="true" />
          <AlertTitle>{t('library.snippetDisabledWarning')}</AlertTitle>
        </Alert>
      )}
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4 lg:p-5">
        <FieldGroup className="gap-3">
          <Field data-invalid={validation === 'title'}>
            <FieldLabel htmlFor="snippet-title">{t('library.field.title')}</FieldLabel>
            <Input
              id="snippet-title"
              dir="auto"
              ref={title}
              value={snippet.title}
              disabled={(busy && !saving) || snippet.deleted}
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
                disabled={(busy && !saving) || snippet.deleted}
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
                disabled={(busy && !saving) || snippet.deleted}
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
          <div className="flex flex-1 flex-col gap-3">
            <div className="flex items-center justify-between gap-2">
              <FieldLabel htmlFor="snippet-body">{t('library.field.body')}</FieldLabel>
              <div className="flex gap-1">
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  disabled={(busy && !saving) || snippet.deleted}
                  onClick={() => onFind('find')}
                >
                  <Search data-icon="inline-start" />
                  {t('editor.find')}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  disabled={(busy && !saving) || snippet.deleted || !richMode}
                  onClick={() => onFind('replace')}
                >
                  {t('editor.findReplace')}
                </Button>
              </div>
            </div>
            {findMode && (richMode || findMode === 'find') && (
              <EditorFind
                key={snippet.id}
                mode={findMode}
                value={richMode ? snippet.body : snippet.body.replace(/\r\n?/g, '\n')}
                onChange={() => {}}
                textarea={body}
                richEditor={richEditor}
                richMode={richMode}
                onClose={() => onFind(null)}
              />
            )}
            <Field data-invalid={validation === 'body'} className="flex-1">
              {!richMode ? (
                <Textarea
                  id="snippet-body"
                  dir="auto"
                  ref={body}
                  value={snippet.body}
                  rows={10}
                  className="min-h-52 flex-1 resize-y font-mono"
                  disabled={(busy && !saving) || snippet.deleted}
                  readOnly
                  aria-invalid={validation === 'body'}
                  onKeyDown={(event) => {
                    if (event.key === 'Escape' && findMode) {
                      event.preventDefault()
                      onFind(null)
                    }
                  }}
                />
              ) : (
                <RichBodyEditor
                  key={snippet.id}
                  id="snippet-body"
                  label={t('library.field.body')}
                  body={snippet.body}
                  richBody={snippet.richBody ?? null}
                  disabled={(busy && !saving) || snippet.deleted}
                  invalid={validation === 'body'}
                  onChange={(value, richBody) => onEdit({ body: value, richBody })}
                  onEditorChange={setRichEditor}
                />
              )}
              <FieldDescription>{t('library.bodyHelp')}</FieldDescription>
              {!richMode && (
                <div className="flex flex-wrap items-center gap-2">
                  <FieldDescription>{t('editor.legacyLineEndings')}</FieldDescription>
                  <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        disabled={(busy && !saving) || snippet.deleted}
                      >
                        {t('editor.convertLineEndings')}
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>{t('editor.convertLineEndingsTitle')}</AlertDialogTitle>
                        <AlertDialogDescription>
                          {t('editor.convertLineEndingsHelp')}
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel type="button">{t('common.cancel')}</AlertDialogCancel>
                        <AlertDialogAction
                          type="button"
                          onClick={() =>
                            onEdit({ body: snippet.body.replace(/\r\n?/g, '\n'), richBody: null })
                          }
                        >
                          {t('editor.convertLineEndingsConfirm')}
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </div>
              )}
              {snippet.richBody && (
                <FieldDescription>{t('editor.richPastePending')}</FieldDescription>
              )}
              {validation === 'body' && <FieldError>{t('library.validation.body')}</FieldError>}
            </Field>
          </div>
        )}
        {selectedSection === 'preview' && (
          <section
            aria-label={t('library.preview')}
            className="min-h-52 rounded-md border bg-muted/40 p-4"
          >
            {snippet.richBody && richMode ? (
              <RichBodyEditor
                key={`${snippet.id}-preview`}
                id="snippet-preview-body"
                label={t('library.preview')}
                body={snippet.body}
                richBody={snippet.richBody}
                readOnlyPreview
                onChange={() => {}}
              />
            ) : (
              <pre
                dir="auto"
                className="whitespace-pre-wrap break-words font-sans text-sm leading-relaxed"
              >
                {snippet.body || t('library.previewEmpty')}
              </pre>
            )}
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
        <Field orientation="horizontal" className="border-t pt-4">
          <FieldLabel htmlFor="snippet-enabled">{t('library.enabled')}</FieldLabel>
          <Switch
            id="snippet-enabled"
            checked={snippet.enabled}
            disabled={(busy && !saving) || snippet.deleted}
            aria-describedby={
              !snippet.enabled && !snippet.deleted ? 'snippet-disabled-warning' : undefined
            }
            onCheckedChange={(enabled) => onEdit({ enabled })}
          />
        </Field>
      </div>
    </form>
  )
}
