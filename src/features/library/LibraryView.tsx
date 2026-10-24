import { useEffect, useRef, useState } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { quitApp } from '@/lib/app-exit'
import { approveExit, registerExitGuard } from '@/lib/exit-guard'
import { useTranslation } from 'react-i18next'
import { Copy, Download, FolderPlus, Plus, RefreshCw, Trash2, Upload } from 'lucide-react'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty'
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { Switch } from '@/components/ui/switch'
import { Textarea } from '@/components/ui/textarea'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import {
  copySnippet,
  exportLibraryFile,
  LibraryCommandError,
  selectLibraryImport,
  useImportLibrary,
  useLibrary,
  useSaveLibrary,
  type ImportPreview,
  type LibraryErrorCode,
  type Library,
  type Snippet,
} from '@/services/library'
import { EngineControls } from './EngineControls'
import { AttachmentEditor } from './AttachmentEditor'
import { LibraryBackups } from './LibraryBackups'
import { searchSnippets, validateSnippet, type SnippetValidation } from './model'

type Draft = { snippet: Snippet; original: Snippet | null; base: Library }
type Notice = {
  kind: 'success' | 'error'
  key:
    | 'library.saved'
    | 'library.exported'
    | 'library.copied'
    | `library.error.${LibraryErrorCode}`
    | 'library.error.importConflict'
} | null
type FolderEditor = { id: string | null; name: string } | null
const same = (left: Snippet, right: Snippet) => JSON.stringify(left) === JSON.stringify(right)

export function LibraryView() {
  const { t } = useTranslation()
  const library = useLibrary()
  const save = useSaveLibrary()
  const importer = useImportLibrary()
  const [draft, setDraft] = useState<Draft | null>(null)
  const [query, setQuery] = useState('')
  const [folder, setFolder] = useState('all')
  const [tab, setTab] = useState('active')
  const [notice, setNotice] = useState<Notice>(null)
  const [validation, setValidation] = useState<SnippetValidation | null>(null)
  const [folderEditor, setFolderEditor] = useState<FolderEditor>(null)
  const [folderInvalid, setFolderInvalid] = useState(false)
  const [importPreview, setImportPreview] = useState<{
    json: string
    preview: ImportPreview
    revision: number
  } | null>(null)
  const [guardOpen, setGuardOpen] = useState(false)
  const pendingAction = useRef<(() => void) | null>(null)
  const pendingCancel = useRef<(() => void) | null>(null)
  const [busy, setBusy] = useState(false)
  const busyRef = useRef(false)
  const dirty = !!draft && (!draft.original || !same(draft.snippet, draft.original))
  const stale = !!draft && !!library.data && draft.base.revision !== library.data.revision
  useEffect(() => {
    const unregister = registerExitGuard(
      () =>
        new Promise<boolean>((resolve) => {
          if (busyRef.current || pendingAction.current) return resolve(false)
          if (!dirty) return resolve(true)
          pendingAction.current = () => resolve(true)
          pendingCancel.current = () => resolve(false)
          setGuardOpen(true)
        }),
    )
    return () => {
      unregister()
      pendingCancel.current?.()
      pendingCancel.current = null
    }
  }, [dirty])
  useEffect(() => {
    if (!isTauri()) return
    let cancelled = false
    let pending = false
    let unlisten: (() => void) | undefined
    void listen('quit-requested', () => {
      if (cancelled || pending) return
      pending = true
      void (async () => {
        try {
          if ((await approveExit()) && !cancelled) await quitApp()
        } catch {
          if (!cancelled) setNotice({ kind: 'error', key: 'library.error.unavailable' })
        } finally {
          pending = false
        }
      })()
    }).then(
      (stop) => {
        if (cancelled) stop()
        else unlisten = stop
      },
      () => {},
    )
    return () => {
      cancelled = true
      unlisten?.()
    }
  }, [])
  useEffect(() => {
    if (!dirty) return
    const guard = (event: BeforeUnloadEvent) => {
      event.preventDefault()
      event.returnValue = ''
    }
    window.addEventListener('beforeunload', guard)
    return () => window.removeEventListener('beforeunload', guard)
  }, [dirty])
  function guarded(action: () => void) {
    if (busyRef.current) return
    if (dirty) {
      pendingAction.current = action
      setGuardOpen(true)
    } else action()
  }
  function select(snippet: Snippet) {
    if (!library.data) return
    setDraft({ snippet: { ...snippet }, original: snippet, base: library.data })
    setValidation(null)
    setNotice(null)
  }
  function edit(patch: Partial<Snippet>) {
    setDraft((current) =>
      current ? { ...current, snippet: { ...current.snippet, ...patch } } : current,
    )
    setValidation(null)
    setNotice(null)
  }
  async function perform(operation: () => Promise<void>, context: 'save' | 'import' = 'save') {
    if (busyRef.current) return
    busyRef.current = true
    setBusy(true)
    setNotice(null)
    try {
      await operation()
    } catch (error) {
      const code = error instanceof LibraryCommandError ? error.code : 'unavailable'
      setNotice({
        kind: 'error',
        key:
          context === 'import' && code === 'conflict'
            ? 'library.error.importConflict'
            : `library.error.${code}`,
      })
    } finally {
      busyRef.current = false
      setBusy(false)
    }
  }
  function newSnippet(source?: Snippet) {
    if (!library.data) return
    setDraft({
      base: library.data,
      original: null,
      snippet: {
        id: crypto.randomUUID(),
        folderId:
          source?.folderId ??
          (library.data.folders.some((item) => item.id === folder) ? folder : null),
        title: source ? t('library.copyTitle', { title: source.title }) : '',
        trigger: source?.trigger ?? '',
        body: source?.body ?? '',
        enabled: !source,
        deleted: false,
      },
    })
    setValidation(null)
    setNotice(null)
  }
  async function saveDraft() {
    if (!draft) return
    const problem = validateSnippet(draft.snippet, draft.base)
    setValidation(problem)
    if (problem) return
    await perform(async () => {
      const current = draft
      const snippets = current.base.snippets.some((item) => item.id === current.snippet.id)
        ? current.base.snippets.map((item) =>
            item.id === current.snippet.id ? current.snippet : item,
          )
        : [...current.base.snippets, current.snippet]
      const saved = await save.mutateAsync({
        expectedRevision: current.base.revision,
        folders: current.base.folders,
        snippets,
      })
      const record = saved.snippets.find((item) => item.id === current.snippet.id)!
      setDraft({ snippet: { ...record }, original: record, base: saved })
      setNotice({ kind: 'success', key: 'library.saved' })
    })
  }
  function changeDeleted(snippet: Snippet, deleted: boolean) {
    guarded(() => {
      void perform(async () => {
        const base = library.data!
        const current = base.snippets.find((item) => item.id === snippet.id)
        if (!current) throw new LibraryCommandError('conflict')
        const next = { ...current, deleted }
        const problem = validateSnippet(next, base)
        if (problem) {
          setValidation(problem)
          return
        }
        const saved = await save.mutateAsync({
          expectedRevision: base.revision,
          folders: base.folders,
          snippets: base.snippets.map((item) => (item.id === next.id ? next : item)),
        })
        setDraft(null)
        setTab(deleted ? 'active' : 'trash')
        setValidation(null)
        setNotice({ kind: 'success', key: 'library.saved' })
        if (!saved.snippets.length) setFolder('all')
      })
    })
  }
  function reload() {
    guarded(() => {
      setDraft(null)
      setNotice(null)
      setValidation(null)
      void library.refetch()
    })
  }
  function beginImport() {
    guarded(() => {
      void perform(async () => {
        const selected = await selectLibraryImport()
        if (selected && library.data) {
          setDraft(null)
          setImportPreview({ ...selected, revision: library.data.revision })
        }
      }, 'import')
    })
  }
  async function confirmImport() {
    if (!importPreview) return
    await perform(async () => {
      await importer.mutateAsync({
        json: importPreview.json,
        expectedRevision: importPreview.revision,
      })
      setImportPreview(null)
      setDraft(null)
      setNotice({ kind: 'success', key: 'library.saved' })
    }, 'import')
  }
  async function saveFolder(remove = false) {
    if (!folderEditor || !library.data) return
    if (
      !remove &&
      (!folderEditor.name.trim() ||
        Array.from(folderEditor.name).length > 120 ||
        folderEditor.name.includes('\0'))
    ) {
      setFolderInvalid(true)
      return
    }
    await perform(async () => {
      const base = library.data!
      const id = folderEditor.id ?? crypto.randomUUID()
      const folders = remove
        ? base.folders.filter((item) => item.id !== id)
        : folderEditor.id
          ? base.folders.map((item) =>
              item.id === id ? { id, name: folderEditor.name.trim() } : item,
            )
          : [...base.folders, { id, name: folderEditor.name.trim() }]
      await save.mutateAsync({
        expectedRevision: base.revision,
        folders,
        snippets: remove
          ? base.snippets.map((item) => (item.folderId === id ? { ...item, folderId: null } : item))
          : base.snippets,
      })
      setFolder(remove ? 'all' : id)
      setFolderEditor(null)
      setDraft(null)
      setNotice({ kind: 'success', key: 'library.saved' })
    })
  }
  const data = library.data
  const visible = data
    ? searchSnippets(
        data.snippets.filter(
          (item) =>
            item.deleted === (tab === 'trash') &&
            (folder === 'all' ||
              (folder === 'unfiled' ? item.folderId === null : item.folderId === folder)),
        ),
        query,
      )
    : []
  return (
    <section className="flex min-w-0 flex-col gap-5 p-5 lg:p-7" aria-labelledby="library-heading">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h1 id="library-heading" className="text-2xl font-semibold tracking-tight">
            {t('library.title')}
          </h1>
          <p className="text-sm text-muted-foreground">{t('library.description')}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" disabled={busy || !data} onClick={beginImport}>
            <Upload data-icon="inline-start" />
            {t('library.import')}
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={busy || !data}
            onClick={() => {
              void perform(async () => {
                if (await exportLibraryFile())
                  setNotice({ kind: 'success', key: 'library.exported' })
              })
            }}
          >
            <Download data-icon="inline-start" />
            {t('library.export')}
          </Button>
          <LibraryBackups />
          <Button variant="outline" size="sm" disabled={busy} onClick={reload}>
            <RefreshCw data-icon="inline-start" />
            {t('library.reload')}
          </Button>
        </div>
      </header>
      <EngineControls
        dirty={dirty || busy}
        eligibleCount={
          data?.snippets.filter((snippet) => snippet.enabled && !snippet.deleted).length ?? 0
        }
      />
      {notice && (
        <Alert
          variant={notice.kind === 'error' ? 'destructive' : 'default'}
          role={notice.kind === 'error' ? 'alert' : 'status'}
        >
          <AlertTitle>{t(notice.kind === 'error' ? 'library.notice' : 'library.done')}</AlertTitle>
          <AlertDescription>{t(notice.key)}</AlertDescription>
        </Alert>
      )}
      {stale && notice?.key !== 'library.error.conflict' && (
        <Alert variant="destructive">
          <AlertTitle>{t('library.notice')}</AlertTitle>
          <AlertDescription>{t('library.error.conflict')}</AlertDescription>
        </Alert>
      )}
      {library.isPending ? (
        <Skeleton className="h-64 w-full" aria-label={t('library.loading')} />
      ) : library.isError ? (
        <Alert variant="destructive">
          <AlertTitle>{t('library.unavailable')}</AlertTitle>
          <AlertDescription>{t('library.error.unavailable')}</AlertDescription>
        </Alert>
      ) : (
        data && (
          <>
            <div className="flex flex-wrap items-end gap-3">
              <Field className="min-w-40 flex-1">
                <FieldLabel htmlFor="library-search">{t('library.search')}</FieldLabel>
                <Input
                  id="library-search"
                  type="search"
                  value={query}
                  placeholder={t('library.searchPlaceholder')}
                  onChange={(event) => setQuery(event.target.value)}
                />
              </Field>
              <Field className="w-auto">
                <FieldLabel htmlFor="library-folder">{t('library.folder')}</FieldLabel>
                <NativeSelect
                  id="library-folder"
                  value={folder}
                  onChange={(event) => setFolder(event.target.value)}
                >
                  <NativeSelectOption value="all">{t('library.allFolders')}</NativeSelectOption>
                  <NativeSelectOption value="unfiled">{t('library.unfiled')}</NativeSelectOption>
                  {data.folders.map((item) => (
                    <NativeSelectOption key={item.id} value={item.id}>
                      {item.name}
                    </NativeSelectOption>
                  ))}
                </NativeSelect>
              </Field>
              <Button
                size="sm"
                variant="outline"
                disabled={busy}
                onClick={() =>
                  guarded(() => {
                    setFolderEditor({ id: null, name: '' })
                    setFolderInvalid(false)
                  })
                }
              >
                <FolderPlus data-icon="inline-start" />
                {t('library.newFolder')}
              </Button>
              {data.folders.some((item) => item.id === folder) && (
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy}
                  onClick={() =>
                    guarded(() => {
                      const current = data.folders.find((item) => item.id === folder)!
                      setFolderEditor({ ...current })
                      setFolderInvalid(false)
                    })
                  }
                >
                  {t('library.manageFolder')}
                </Button>
              )}
              <Button size="sm" disabled={busy} onClick={() => guarded(() => newSnippet())}>
                <Plus data-icon="inline-start" />
                {t('library.new')}
              </Button>
            </div>
            <ToggleGroup
              type="single"
              variant="outline"
              value={tab}
              onValueChange={(value) => {
                if (value) setTab(value)
              }}
              aria-label={t('library.collection')}
            >
              <ToggleGroupItem value="active">{t('library.active')}</ToggleGroupItem>
              <ToggleGroupItem value="trash">{t('library.trash')}</ToggleGroupItem>
            </ToggleGroup>
            <div className="grid min-w-0 gap-5 xl:grid-cols-[minmax(12rem,0.8fr)_minmax(18rem,1.5fr)]">
              <section aria-label={t('library.snippets')} className="flex min-w-0 flex-col gap-2">
                <p className="text-xs text-muted-foreground">
                  {t('library.count', { count: visible.length })}
                </p>
                {visible.length ? (
                  visible.map((snippet) => (
                    <Button
                      key={snippet.id}
                      variant={draft?.snippet.id === snippet.id ? 'secondary' : 'outline'}
                      className="h-auto min-h-16 w-full justify-start whitespace-normal py-3 text-start"
                      disabled={busy}
                      onClick={() => guarded(() => select(snippet))}
                    >
                      <span className="flex min-w-0 flex-1 flex-col gap-1">
                        <span className="truncate font-medium">{snippet.title}</span>
                        <span className="font-mono text-xs text-muted-foreground">
                          {snippet.trigger}
                        </span>
                      </span>
                      {!snippet.enabled && <Badge variant="outline">{t('common.disabled')}</Badge>}
                    </Button>
                  ))
                ) : (
                  <Empty>
                    <EmptyHeader>
                      <EmptyTitle>{t('library.empty')}</EmptyTitle>
                      <EmptyDescription>{t('library.emptyDescription')}</EmptyDescription>
                    </EmptyHeader>
                  </Empty>
                )}
              </section>
              <section aria-label={t('library.editor')} className="min-w-0 rounded-lg border p-4">
                {draft ? (
                  <form
                    className="flex flex-col gap-5"
                    onSubmit={(event) => {
                      event.preventDefault()
                      void saveDraft()
                    }}
                  >
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <h2 className="font-semibold">{t('library.editor')}</h2>
                      {dirty && <Badge variant="secondary">{t('library.unsaved')}</Badge>}
                      {draft.snippet.deleted && (
                        <Badge variant="outline">{t('library.inTrash')}</Badge>
                      )}
                    </div>
                    <FieldGroup>
                      <Field data-invalid={validation === 'title'}>
                        <FieldLabel htmlFor="snippet-title">{t('library.field.title')}</FieldLabel>
                        <Input
                          id="snippet-title"
                          value={draft.snippet.title}
                          disabled={busy || draft.snippet.deleted}
                          aria-invalid={validation === 'title'}
                          onChange={(event) => edit({ title: event.target.value })}
                        />
                        {validation === 'title' && (
                          <FieldError>{t('library.validation.title')}</FieldError>
                        )}
                      </Field>
                      <Field
                        data-invalid={validation === 'trigger' || validation === 'duplicateTrigger'}
                      >
                        <FieldLabel htmlFor="snippet-trigger">
                          {t('library.field.trigger')}
                        </FieldLabel>
                        <Input
                          id="snippet-trigger"
                          value={draft.snippet.trigger}
                          autoComplete="off"
                          autoCapitalize="off"
                          spellCheck={false}
                          disabled={busy || draft.snippet.deleted}
                          aria-invalid={
                            validation === 'trigger' || validation === 'duplicateTrigger'
                          }
                          onChange={(event) => edit({ trigger: event.target.value })}
                        />
                        <FieldDescription>{t('library.triggerHelp')}</FieldDescription>
                        {(validation === 'trigger' || validation === 'duplicateTrigger') && (
                          <FieldError>{t(`library.validation.${validation}`)}</FieldError>
                        )}
                      </Field>
                      <Field>
                        <FieldLabel htmlFor="snippet-folder">
                          {t('library.field.folder')}
                        </FieldLabel>
                        <NativeSelect
                          id="snippet-folder"
                          value={draft.snippet.folderId ?? ''}
                          disabled={busy || draft.snippet.deleted}
                          onChange={(event) => edit({ folderId: event.target.value || null })}
                        >
                          <NativeSelectOption value="">{t('library.unfiled')}</NativeSelectOption>
                          {draft.base.folders.map((item) => (
                            <NativeSelectOption key={item.id} value={item.id}>
                              {item.name}
                            </NativeSelectOption>
                          ))}
                        </NativeSelect>
                      </Field>
                      <Field data-invalid={validation === 'body'}>
                        <FieldLabel htmlFor="snippet-body">{t('library.field.body')}</FieldLabel>
                        <Textarea
                          id="snippet-body"
                          value={draft.snippet.body}
                          rows={8}
                          className="resize-y"
                          disabled={busy || draft.snippet.deleted}
                          aria-invalid={validation === 'body'}
                          onChange={(event) => edit({ body: event.target.value })}
                        />
                        <FieldDescription>{t('library.bodyHelp')}</FieldDescription>
                        {validation === 'body' && (
                          <FieldError>{t('library.validation.body')}</FieldError>
                        )}
                      </Field>
                      <Field orientation="horizontal">
                        <FieldLabel htmlFor="snippet-enabled">{t('library.enabled')}</FieldLabel>
                        <Switch
                          id="snippet-enabled"
                          checked={draft.snippet.enabled}
                          disabled={busy || draft.snippet.deleted}
                          onCheckedChange={(enabled) => edit({ enabled })}
                        />
                      </Field>
                    </FieldGroup>
                    <div className="flex flex-wrap gap-2">
                      <Button type="submit" disabled={busy || !dirty || draft.snippet.deleted}>
                        {t(busy ? 'library.saving' : 'library.save')}
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        disabled={busy || !draft.snippet.body}
                        onClick={() => {
                          void perform(async () => {
                            await copySnippet(draft.snippet.body)
                            setNotice({ kind: 'success', key: 'library.copied' })
                          })
                        }}
                      >
                        <Copy data-icon="inline-start" />
                        {t('library.copy')}
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        disabled={busy}
                        onClick={() => guarded(() => newSnippet(draft.snippet))}
                      >
                        {t('library.duplicate')}
                      </Button>
                      {draft.original && (
                        <Button
                          type="button"
                          variant="outline"
                          disabled={busy}
                          onClick={() => changeDeleted(draft.original!, !draft.original!.deleted)}
                        >
                          <Trash2 data-icon="inline-start" />
                          {t(draft.snippet.deleted ? 'library.restore' : 'library.moveToTrash')}
                        </Button>
                      )}
                    </div>
                    <Separator />
                    {draft.original ? (
                      <AttachmentEditor
                        key={draft.snippet.id}
                        id={draft.snippet.id}
                        revision={draft.base.revision}
                        disabled={busy || stale || draft.snippet.deleted}
                      />
                    ) : (
                      <p className="text-sm text-muted-foreground">{t('attachments.saveFirst')}</p>
                    )}
                    <section
                      aria-label={t('library.preview')}
                      className="flex min-w-0 flex-col gap-2"
                    >
                      <h3 className="text-sm font-medium">{t('library.preview')}</h3>
                      <pre className="max-h-64 overflow-y-auto whitespace-pre-wrap break-words rounded-md bg-muted p-3 font-sans text-sm">
                        {draft.snippet.body || t('library.previewEmpty')}
                      </pre>
                    </section>
                  </form>
                ) : (
                  <Empty>
                    <EmptyHeader>
                      <EmptyTitle>{t('library.choose')}</EmptyTitle>
                      <EmptyDescription>{t('library.chooseDescription')}</EmptyDescription>
                    </EmptyHeader>
                  </Empty>
                )}
              </section>
            </div>
          </>
        )
      )}
      <AlertDialog
        open={guardOpen}
        onOpenChange={(open) => {
          setGuardOpen(open)
          if (!open) {
            pendingAction.current = null
            pendingCancel.current?.()
            pendingCancel.current = null
          }
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('library.discardTitle')}</AlertDialogTitle>
            <AlertDialogDescription>{t('library.discardDescription')}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('library.keepEditing')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const action = pendingAction.current
                pendingAction.current = null
                pendingCancel.current = null
                setDraft(null)
                setValidation(null)
                action?.()
              }}
            >
              {t('library.discard')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      <Dialog
        open={!!folderEditor}
        onOpenChange={(open) => {
          if (!open && !busy) setFolderEditor(null)
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {t(folderEditor?.id ? 'library.manageFolder' : 'library.newFolder')}
            </DialogTitle>
            <DialogDescription>{t('library.folderHelp')}</DialogDescription>
          </DialogHeader>
          {notice?.kind === 'error' && (
            <Alert variant="destructive">
              <AlertTitle>{t('library.notice')}</AlertTitle>
              <AlertDescription>{t(notice.key)}</AlertDescription>
            </Alert>
          )}
          <form
            className="flex flex-col gap-5"
            onSubmit={(event) => {
              event.preventDefault()
              void saveFolder()
            }}
          >
            <FieldGroup>
              <Field data-invalid={folderInvalid}>
                <FieldLabel htmlFor="folder-name">{t('library.folderName')}</FieldLabel>
                <Input
                  id="folder-name"
                  value={folderEditor?.name ?? ''}
                  disabled={busy}
                  aria-invalid={folderInvalid}
                  onChange={(event) => {
                    setFolderEditor((current) =>
                      current ? { ...current, name: event.target.value } : current,
                    )
                    setFolderInvalid(false)
                  }}
                />
                {folderInvalid && <FieldError>{t('library.validation.title')}</FieldError>}
              </Field>
            </FieldGroup>
            <DialogFooter>
              {folderEditor?.id && (
                <Button
                  type="button"
                  variant="outline"
                  disabled={busy}
                  onClick={() => {
                    void saveFolder(true)
                  }}
                >
                  {t('library.deleteFolder')}
                </Button>
              )}
              <Button type="submit" disabled={busy}>
                {t('library.saveFolder')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
      <Dialog
        open={!!importPreview}
        onOpenChange={(open) => {
          if (!open && !busy) setImportPreview(null)
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('library.importPreview')}</DialogTitle>
            <DialogDescription>{t('library.importHelp')}</DialogDescription>
          </DialogHeader>
          {notice?.kind === 'error' && (
            <Alert variant="destructive">
              <AlertTitle>{t('library.notice')}</AlertTitle>
              <AlertDescription>{t(notice.key)}</AlertDescription>
            </Alert>
          )}
          {importPreview && <p>{t('library.importCounts', importPreview.preview)}</p>}
          <DialogFooter>
            <Button variant="outline" disabled={busy} onClick={() => setImportPreview(null)}>
              {t('library.cancel')}
            </Button>
            <Button
              disabled={busy}
              onClick={() => {
                void confirmImport()
              }}
            >
              {t('library.confirmImport')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </section>
  )
}
