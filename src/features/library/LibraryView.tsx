import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { quitApp } from '@/lib/app-exit'
import { approveExit, registerExitGuard } from '@/lib/exit-guard'
import { useTranslation } from 'react-i18next'
import { Download, MoreHorizontal, Plus, RefreshCw, Upload } from 'lucide-react'
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
import { Field, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
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
import { SnippetEditor } from './SnippetEditor'
import { SnippetList } from './SnippetList'
import { useUIStore } from '@/store/ui-store'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { LibraryDashboard } from './LibraryDashboard'
import { useLibraryMetadata, useSetFavorite } from '@/services/library-metadata'
import { bindLibraryActions } from '@/lib/commands/library-commands'
import {
  searchSnippets,
  validateSnippet,
  resolveFolderSelection,
  type SnippetValidation,
} from './model'

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
  const metadata = useLibraryMetadata()
  const favorite = useSetFavorite()
  const favoriteIds = new Set(
    metadata.data?.filter((info) => info.favorite).map((info) => info.snippetId),
  )
  const [findMode, setFindMode] = useState<'find' | 'replace' | null>(null)
  const [draft, setDraft] = useState<Draft | null>(null)
  const [query, setQuery] = useState('')
  const requestedFolder = useUIStore((state) => state.libraryFolder)
  const folder = resolveFolderSelection(library.data?.folders ?? [], requestedFolder)
  const tab = useUIStore((state) => state.libraryCollection)
  const listPercent = useUIStore((state) => state.layout.listPercent)
  const listWidth = useUIStore((state) => state.setSnippetListWidth)
  function setFolder(value: string) {
    useUIStore.getState().setLibraryFilter(useUIStore.getState().libraryCollection, value)
  }
  function setTab(value: 'active' | 'favorites' | 'trash') {
    useUIStore.getState().setLibraryFilter(value, useUIStore.getState().libraryFolder)
  }
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
  useLayoutEffect(() => {
    useUIStore.getState().setExpansionStartBlocked(dirty || busy)
    return () => useUIStore.getState().setExpansionStartBlocked(true)
  }, [dirty, busy])
  const stale = !!draft && !!library.data && draft.base.revision !== library.data.revision
  useEffect(() =>
    bindLibraryActions((action) => {
      if (busyRef.current) return
      if (action === 'new') guarded(() => newSnippet())
      else if (action === 'new-folder')
        guarded(() => {
          setFolderEditor({ id: null, name: '' })
          setFolderInvalid(false)
        })
      else if (action === 'manage-folder')
        guarded(() => {
          const current = library.data?.folders.find((item) => item.id === folder)
          if (current) {
            setFolderEditor({ ...current })
            setFolderInvalid(false)
          }
        })
      else if (action === 'save') {
        if (draft && !draft.snippet.deleted) void saveDraft()
      } else if (action === 'search' || !draft) document.getElementById('library-search')?.focus()
      else if (!draft.snippet.deleted) setFindMode(action === 'replace' ? 'replace' : 'find')
    }),
  )
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
    setFindMode(null)
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
    setQuery('')
    const nextFolder =
      source && folder !== 'all' && folder !== (source.folderId ?? 'unfiled') ? 'all' : folder
    useUIStore.getState().setLibraryFilter('active', nextFolder)
    setDraft({
      base: library.data,
      original: null,
      snippet: {
        id: crypto.randomUUID(),
        folderId: source
          ? source.folderId
          : library.data.folders.some((item) => item.id === folder)
            ? folder
            : null,
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
            (tab !== 'favorites' || favoriteIds.has(item.id)) &&
            (folder === 'all' ||
              (folder === 'unfiled' ? item.folderId === null : item.folderId === folder)),
        ),
        query,
      )
    : []
  const folderName = data?.folders.find((item) => item.id === folder)?.name
  const heading =
    folderName ??
    t(
      folder === 'unfiled'
        ? 'library.unfiled'
        : tab === 'favorites'
          ? 'library.favorites'
          : tab === 'trash'
            ? 'library.trash'
            : 'workspace.allSnippets',
    )
  return (
    <section className="flex h-full min-w-0 flex-col" aria-labelledby="library-heading">
      <header className="flex min-h-16 shrink-0 flex-wrap items-center justify-between gap-3 border-b px-5 py-3">
        <div className="min-w-0">
          <h1 id="library-heading" className="truncate text-lg font-semibold tracking-tight">
            {heading}
          </h1>
        </div>
        <div className="flex items-center gap-2">
          <Button size="sm" disabled={busy || !data} onClick={() => guarded(() => newSnippet())}>
            <Plus data-icon="inline-start" />
            {t('library.new')}
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button size="icon-sm" variant="ghost" aria-label={t('workspace.libraryActions')}>
                <MoreHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuGroup>
                <DropdownMenuItem disabled={busy || !data} onSelect={beginImport}>
                  <Upload />
                  {t('library.import')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  disabled={busy || !data}
                  onSelect={() => {
                    void perform(async () => {
                      if (await exportLibraryFile())
                        setNotice({ kind: 'success', key: 'library.exported' })
                    })
                  }}
                >
                  <Download />
                  {t('library.export')}
                </DropdownMenuItem>
                <DropdownMenuItem disabled={busy} onSelect={reload}>
                  <RefreshCw />
                  {t('library.reload')}
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>
      {(metadata.isError || favorite.isError) && (
        <Alert variant="destructive">
          <AlertTitle>{t('library.metadataError')}</AlertTitle>
        </Alert>
      )}
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
          <ResizablePanelGroup
            direction="horizontal"
            className="min-h-0 flex-1"
            onLayout={(values) => {
              if (values.length === 2) listWidth(values[0])
            }}
          >
            <ResizablePanel
              id="snippets"
              order={1}
              defaultSize={listPercent}
              minSize={25}
              maxSize={50}
            >
              <SnippetList
                snippets={visible}
                selected={draft?.snippet.id ?? null}
                favorites={favoriteIds}
                busy={busy}
                query={query}
                onQuery={setQuery}
                onSelect={(snippet) => guarded(() => select(snippet))}
                onDelete={(snippet) => changeDeleted(snippet, true)}
                onClearSelection={() => guarded(() => setDraft(null))}
              />
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel id="editor" order={2} defaultSize={100 - listPercent} minSize={45}>
              <section aria-label={t('library.editor')} className="h-full min-w-0">
                {draft ? (
                  <SnippetEditor
                    key={draft.snippet.id}
                    snippet={draft.snippet}
                    original={draft.original}
                    folders={draft.base.folders}
                    revision={draft.base.revision}
                    dirty={dirty}
                    busy={busy}
                    stale={stale}
                    validation={validation}
                    favorite={favoriteIds.has(draft.snippet.id)}
                    favoritePending={favorite.isPending}
                    findMode={findMode}
                    onFind={setFindMode}
                    onEdit={edit}
                    onSave={() => {
                      void saveDraft()
                    }}
                    onCopy={() => {
                      void perform(async () => {
                        await copySnippet(draft.snippet.body)
                        setNotice({ kind: 'success', key: 'library.copied' })
                      })
                    }}
                    onDuplicate={() => guarded(() => newSnippet(draft.snippet))}
                    onDelete={() => changeDeleted(draft.original!, !draft.original!.deleted)}
                    onFavorite={() =>
                      favorite.mutate({
                        id: draft.snippet.id,
                        revision: draft.base.revision,
                        favorite: !favoriteIds.has(draft.snippet.id),
                      })
                    }
                  />
                ) : (
                  <div className="flex h-full flex-col gap-5 overflow-y-auto p-5 lg:p-6">
                    <Empty className="flex-1">
                      <EmptyHeader>
                        <EmptyTitle>{t('library.choose')}</EmptyTitle>
                        <EmptyDescription>{t('library.chooseDescription')}</EmptyDescription>
                      </EmptyHeader>
                    </Empty>
                    {metadata.data && (
                      <LibraryDashboard snippets={data.snippets} metadata={metadata.data} />
                    )}
                  </div>
                )}
              </section>
            </ResizablePanel>
          </ResizablePanelGroup>
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
