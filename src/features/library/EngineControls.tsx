import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  getProductStatus,
  pauseProduct,
  ProductCommandError,
  stopProduct,
  startProduct,
} from '@/services/product'
import { libraryKeys } from '@/services/library'

export function EngineControls({
  dirty,
  eligibleCount,
}: {
  dirty: boolean
  eligibleCount: number
}) {
  const { t } = useTranslation()
  const client = useQueryClient()
  const [error, setError] = useState<string | null>(null)
  const status = useQuery({
    queryKey: libraryKeys.engine,
    queryFn: getProductStatus,
    retry: false,
    refetchInterval: 1000,
  })
  const mutation = useMutation({
    onMutate: () => client.cancelQueries({ queryKey: libraryKeys.engine }),
    mutationFn: (action: 'start' | 'stop' | 'pause') =>
      action === 'start'
        ? startProduct()
        : action === 'stop'
          ? stopProduct()
          : status.data?.phase === 'paused'
            ? startProduct()
            : pauseProduct(),
    onSuccess: (next) => {
      client.setQueryData(libraryKeys.engine, next)
      setError(null)
    },
    onError: (reason) =>
      setError(
        reason instanceof ProductCommandError
          ? t(`product.error.${reason.code}`)
          : t('library.error.unavailable'),
      ),
  })
  const active = status.data?.phase === 'running' || status.data?.phase === 'paused'
  const stoppable = active || status.data?.phase === 'fault'
  return (
    <section aria-label={t('library.engine')} className="flex flex-col gap-3 rounded-lg border p-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-semibold">{t('library.engine')}</h2>
          <Badge variant="secondary" role="status">
            {status.isError
              ? t('library.unavailable')
              : status.data
                ? t(`probe.phase.${status.data.phase}`)
                : t('library.loading')}
          </Badge>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            disabled={
              status.data?.phase !== 'stopped' ||
              !!status.error ||
              mutation.isPending ||
              dirty ||
              eligibleCount === 0
            }
            onClick={() => mutation.mutate('start')}
          >
            {t('library.start')}
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={!active || mutation.isPending}
            onClick={() => mutation.mutate('pause')}
          >
            {t(status.data?.phase === 'paused' ? 'library.resume' : 'library.pause')}
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={!stoppable || mutation.isPending}
            onClick={() => mutation.mutate('stop')}
          >
            {t('library.stop')}
          </Button>
        </div>
      </div>
      <p className="text-xs text-muted-foreground">{t('library.support')}</p>
      {status.data?.clipboardIncomplete && (
        <Alert>
          <AlertDescription>{t('product.clipboardIncomplete')}</AlertDescription>
        </Alert>
      )}
      {(error || status.data?.lastError) && (
        <Alert variant="destructive">
          <AlertTitle>{t('library.engineNotice')}</AlertTitle>
          <AlertDescription>
            {error ?? t(`product.error.${status.data!.lastError!}`)}
          </AlertDescription>
        </Alert>
      )}
    </section>
  )
}
