import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQueryClient } from '@tanstack/react-query'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Copy } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { dismissQuickPane } from '@/services/quick-pane'
import { copySnippet, libraryKeys, useLibrary } from '@/services/library'
import type { Snippet } from '@/services/library'
import { searchSnippets } from '@/features/library/model'

export function QuickPaneApp() {
  const { t } = useTranslation()
  const client = useQueryClient()
  const library = useLibrary()
  const [query, setQuery] = useState('')
  const [failed, setFailed] = useState(false)
  const [copying, setCopying] = useState(false)
  const input = useRef<HTMLInputElement>(null)
  const generation = useRef(0)
  const busy = useRef<number | null>(null)
  const mounted = useRef(false)
  const nextInteraction = useCallback(() => {
    busy.current = null
    return ++generation.current
  }, [])
  const dismiss = useCallback(async () => {
    const owner = nextInteraction()
    setQuery('')
    setFailed(false)
    setCopying(false)
    try {
      await dismissQuickPane()
    } catch {
      if (mounted.current && generation.current === owner) setFailed(true)
    }
  }, [nextInteraction])
  useEffect(() => {
    mounted.current = true
    let cancelled = false
    let stop: (() => void) | undefined
    void getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (cancelled) return
        if (focused) {
          nextInteraction()
          setFailed(false)
          setCopying(false)
          input.current?.focus()
          void client.invalidateQueries({ queryKey: libraryKeys.all })
        } else void dismiss()
      })
      .then(
        (unlisten) => {
          if (cancelled) unlisten()
          else stop = unlisten
        },
        () => {
          if (!cancelled) setFailed(true)
        },
      )
    input.current?.focus()
    return () => {
      cancelled = true
      mounted.current = false
      nextInteraction()
      stop?.()
    }
  }, [client, dismiss, nextInteraction])
  async function copy(snippet: Snippet) {
    if (busy.current !== null) return
    const owner = nextInteraction()
    busy.current = owner
    setCopying(true)
    setFailed(false)
    try {
      await copySnippet(snippet.body)
      if (mounted.current && generation.current === owner) await dismissQuickPane()
    } catch {
      if (mounted.current && generation.current === owner) setFailed(true)
    } finally {
      if (busy.current === owner) {
        busy.current = null
        if (mounted.current) setCopying(false)
      }
    }
  }
  const snippets = searchSnippets(
    library.data?.snippets.filter((item) => item.enabled && !item.deleted) ?? [],
    query,
  )
  return (
    <section
      className="flex h-svh w-full flex-col gap-3 overflow-hidden rounded-[var(--app-corner-radius)] border bg-background p-4 text-foreground shadow-lg"
      aria-label={t('library.picker')}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          event.preventDefault()
          void dismiss()
        }
      }}
    >
      <header className="flex items-center justify-between gap-3">
        <h1 className="text-sm font-semibold">{t('library.picker')}</h1>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => {
            void dismiss()
          }}
        >
          {t('common.close')}
        </Button>
      </header>
      <Input
        ref={input}
        type="search"
        value={query}
        aria-label={t('library.search')}
        placeholder={t('library.searchPlaceholder')}
        autoComplete="off"
        autoCorrect="off"
        autoCapitalize="off"
        spellCheck={false}
        onChange={(event) => {
          setQuery(event.target.value)
          setFailed(false)
        }}
        onKeyDown={(event) => {
          if (event.key === 'Enter') {
            event.preventDefault()
            if (!event.repeat && !event.nativeEvent.isComposing && snippets[0])
              void copy(snippets[0])
          }
        }}
      />
      <p className="text-xs text-muted-foreground">{t('library.pickerHelp')}</p>
      {failed || library.isError ? (
        <Alert variant="destructive">
          <AlertTitle>{t('library.unavailable')}</AlertTitle>
          <AlertDescription>{t('library.error.unavailable')}</AlertDescription>
        </Alert>
      ) : null}
      <div
        className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto"
        aria-label={t('library.snippets')}
      >
        {library.isPending ? (
          <p role="status">{t('library.loading')}</p>
        ) : snippets.length ? (
          snippets.slice(0, 100).map((snippet) => (
            <Button
              key={snippet.id}
              variant="outline"
              className="h-auto min-h-14 justify-start py-3 text-start"
              disabled={copying}
              onClick={() => {
                void copy(snippet)
              }}
              aria-label={t('library.copyNamed', { title: snippet.title })}
            >
              <span className="flex min-w-0 flex-1 flex-col gap-1">
                <span className="truncate">{snippet.title}</span>
                <span className="truncate font-mono text-xs text-muted-foreground">
                  {snippet.trigger}
                </span>
              </span>
              <Copy data-icon="inline-end" />
            </Button>
          ))
        ) : (
          !library.isError && (
            <Empty>
              <EmptyHeader>
                <EmptyTitle>{t('library.empty')}</EmptyTitle>
                <EmptyDescription>{t('library.emptyDescription')}</EmptyDescription>
              </EmptyHeader>
            </Empty>
          )
        )}
      </div>
    </section>
  )
}
