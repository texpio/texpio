import { useTranslation } from 'react-i18next'
import type { Snippet, SnippetInfo } from '@/lib/bindings'
export function LibraryDashboard({
  snippets,
  metadata,
}: {
  snippets: Snippet[]
  metadata: SnippetInfo[]
}) {
  const { t } = useTranslation()
  const active = snippets.filter((s) => !s.deleted)
  const ids = new Set(active.map((s) => s.id))
  const infos = metadata.filter((info) => ids.has(info.snippetId))
  const titles = new Map(active.map((s) => [s.id, s.title]))
  const recent = infos
    .filter((info) => info.lastUsedAt !== null)
    .sort((a, b) => (b.lastUsedAt ?? 0) - (a.lastUsedAt ?? 0))
    .slice(0, 3)
  const frequent = infos
    .filter((info) => info.usageCount > 0)
    .sort((a, b) => b.usageCount - a.usageCount)
    .slice(0, 3)
  return (
    <section aria-label={t('dashboard.title')} className="grid gap-3 sm:grid-cols-3">
      <div className="rounded-lg border p-3">
        <p className="text-xs text-muted-foreground">{t('dashboard.snippets')}</p>
        <p className="text-xl font-semibold">{active.length}</p>
        <p className="text-xs text-muted-foreground">
          {t('dashboard.expansions', {
            count: infos.reduce((sum, info) => sum + info.usageCount, 0),
          })}
        </p>
      </div>
      {[
        { label: 'dashboard.recent', items: recent },
        { label: 'dashboard.frequent', items: frequent },
      ].map((group) => (
        <div key={group.label} className="rounded-lg border p-3">
          <h2 className="text-sm font-medium">
            {t(group.label as 'dashboard.recent' | 'dashboard.frequent')}
          </h2>
          {group.items.length ? (
            <ul className="mt-2 space-y-1 text-xs text-muted-foreground">
              {group.items.map((info) => (
                <li key={info.snippetId} className="truncate">
                  {titles.get(info.snippetId)}{' '}
                  <span className="tabular-nums">({info.usageCount})</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="mt-2 text-xs text-muted-foreground">{t('dashboard.empty')}</p>
          )}
        </div>
      ))}
    </section>
  )
}
