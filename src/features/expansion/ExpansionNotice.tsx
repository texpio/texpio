import { TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { useIsMacOS } from '@/hooks/use-platform'
import { useUIStore } from '@/store/ui-store'
import { useExpansion } from './use-expansion'

export function ExpansionNotice() {
  const { t } = useTranslation()
  const engine = useExpansion()
  const macos = useIsMacOS()
  const { status, error } = engine
  const phase = status.data?.phase
  const unavailable = status.isError
  const off = !unavailable && (phase === 'paused' || phase === 'stopped')
  const fault = !unavailable && phase === 'fault'
  const reason = error ?? status.data?.lastError
  // Starting cannot succeed until macOS grants input access, so route the user
  // to the in-app permission guide instead of offering a doomed retry.
  const permissionsMissing = macos && reason === 'inputBlocked'
  if (
    !engine.desktop ||
    (!unavailable && !off && !fault && !reason && !status.data?.clipboardIncomplete)
  )
    return null
  const title = unavailable
    ? 'expansion.unavailable'
    : fault
      ? 'expansion.fault'
      : phase === 'paused'
        ? 'expansion.paused'
        : phase === 'stopped'
          ? 'expansion.stopped'
          : 'library.engineNotice'
  return (
    <Alert
      variant={unavailable || fault || reason ? 'destructive' : off ? 'warning' : 'default'}
      className="shrink-0 rounded-none border-x-0 border-t-0 px-5 py-3"
    >
      <TriangleAlert aria-hidden="true" />
      <div className="flex min-w-0 flex-wrap items-center justify-between gap-x-6 gap-y-3">
        <div className="min-w-0 flex-1 basis-64">
          <AlertTitle>{t(title)}</AlertTitle>
          <AlertDescription className="mt-1">
            {off && <p>{t('expansion.offHelp')}</p>}
            {fault && <p>{t('expansion.faultHelp')}</p>}
            {unavailable && <p>{t('expansion.unavailableHelp')}</p>}
            {reason && <p>{t(`product.error.${reason}`)}</p>}
            {status.data?.clipboardIncomplete && <p>{t('product.clipboardIncomplete')}</p>}
            {off && engine.startHint && <p>{t(engine.startHint)}</p>}
          </AlertDescription>
        </div>
        {unavailable ? (
          <Button
            size="sm"
            variant="outline"
            disabled={status.isFetching || engine.pending}
            onClick={engine.retry}
          >
            {t('common.retry')}
          </Button>
        ) : permissionsMissing ? (
          <Button
            size="sm"
            variant="outline"
            onClick={() => {
              const state = useUIStore.getState()
              state.setPreferencesSection('expansion')
              state.setActiveView('preferences')
            }}
          >
            {t('product.permissions.review')}
          </Button>
        ) : fault ? (
          <Button
            size="sm"
            variant="outline"
            disabled={!engine.can('stop')}
            onClick={() => engine.act('stop')}
          >
            {t('expansion.reset')}
          </Button>
        ) : off ? (
          <Button size="sm" disabled={!engine.can('start')} onClick={() => engine.act('start')}>
            {t(engine.pendingAction === 'start' ? 'expansion.starting' : 'expansion.turnOn')}
          </Button>
        ) : null}
      </div>
    </Alert>
  )
}
