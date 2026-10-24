import { useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { Star } from 'lucide-react'
import type { Snippet } from '@/services/library'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Field, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty'

interface SnippetListProps {
  snippets: Snippet[]
  selected: string | null
  favorites: Set<string>
  busy: boolean
  query: string
  onQuery: (query: string) => void
  onSelect: (snippet: Snippet) => void
  onDelete: (snippet: Snippet) => void
  onClearSelection: () => void
}
export function SnippetList({
  snippets,
  selected,
  favorites,
  busy,
  query,
  onQuery,
  onSelect,
  onDelete,
  onClearSelection,
}: SnippetListProps) {
  const { t } = useTranslation()
  const list = useRef<HTMLElement>(null)
  return (
    <section
      ref={list}
      aria-label={t('library.snippets')}
      className="flex h-full min-w-0 flex-col"
      onKeyDown={(event) => {
        if (busy || event.altKey || event.ctrlKey || event.metaKey || event.nativeEvent.isComposing)
          return
        const button =
          event.target instanceof Element
            ? event.target.closest<HTMLElement>('button[data-snippet-id]')
            : null
        const index = snippets.findIndex((snippet) => snippet.id === button?.dataset.snippetId)
        if (!button || index < 0) return
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
          event.preventDefault()
          const next =
            snippets[
              (index + (event.key === 'ArrowDown' ? 1 : -1) + snippets.length) % snippets.length
            ]
          list.current?.querySelector<HTMLElement>(`button[data-snippet-id="${next.id}"]`)?.focus()
        } else if (event.key === 'Delete' && !snippets[index].deleted) {
          event.preventDefault()
          onDelete(snippets[index])
        } else if (event.key === 'Escape') {
          event.preventDefault()
          onClearSelection()
        }
      }}
    >
      <div className="flex shrink-0 flex-col gap-3 border-b p-4">
        <Field>
          <FieldLabel htmlFor="library-search" className="sr-only">
            {t('library.search')}
          </FieldLabel>
          <Input
            id="library-search"
            type="search"
            value={query}
            placeholder={t('library.searchPlaceholder')}
            onChange={(event) => onQuery(event.target.value)}
          />
        </Field>
        <p className="text-xs text-muted-foreground">
          {t('library.count', { count: snippets.length })}
        </p>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto p-2">
        {snippets.length ? (
          snippets.map((snippet) => (
            <Button
              key={snippet.id}
              data-snippet-id={snippet.id}
              variant={selected === snippet.id ? 'secondary' : 'ghost'}
              aria-current={selected === snippet.id ? 'true' : undefined}
              className="mb-1 h-auto min-h-20 w-full justify-start px-3 py-3 text-start"
              disabled={busy}
              onClick={() => onSelect(snippet)}
            >
              <span className="flex min-w-0 flex-1 flex-col gap-1">
                <span className="flex min-w-0 items-center justify-between gap-2">
                  <span className="truncate font-medium">{snippet.title}</span>
                  {favorites.has(snippet.id) && (
                    <Star className="size-3 shrink-0 fill-current" aria-hidden="true" />
                  )}
                </span>
                <span className="truncate font-mono text-xs text-muted-foreground">
                  {snippet.trigger}
                </span>
                <span className="truncate text-xs font-normal text-muted-foreground">
                  {snippet.body.slice(0, 120).replace(/\s+/g, ' ') || t('library.previewEmpty')}
                </span>
              </span>
              {!snippet.enabled && (
                <Badge variant="outline" className="ms-2">
                  {t('common.disabled')}
                </Badge>
              )}
            </Button>
          ))
        ) : (
          <Empty className="min-h-48 p-4">
            <EmptyHeader>
              <EmptyTitle>{t(query ? 'workspace.noResults' : 'workspace.emptyList')}</EmptyTitle>
              <EmptyDescription>
                {t(query ? 'workspace.noResultsHelp' : 'workspace.emptyListHelp')}
              </EmptyDescription>
            </EmptyHeader>
            {query && (
              <Button variant="outline" size="sm" onClick={() => onQuery('')}>
                {t('workspace.clearSearch')}
              </Button>
            )}
          </Empty>
        )}
      </div>
    </section>
  )
}
