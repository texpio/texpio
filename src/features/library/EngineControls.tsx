import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { useExpansion } from '@/features/expansion/use-expansion'

export function EngineControls() {
  const { t } = useTranslation()
  const engine = useExpansion()
  const phase = engine.status.data?.phase
  const active = phase === 'running' || phase === 'paused'
  return (
    <section aria-label={t('library.engine')}>
      <Card>
        <CardHeader>
          <div className="flex flex-wrap items-center justify-between gap-2">
            <CardTitle>{t('library.engine')}</CardTitle>
            <Badge variant="secondary" role="status">
              {t(
                engine.status.isError
                  ? 'library.unavailable'
                  : phase
                    ? `probe.phase.${phase}`
                    : 'library.loading',
              )}
            </Badge>
          </div>
          <CardDescription>{t('expansion.settingsHelp')}</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <div className="flex flex-wrap gap-2">
            {!active && (
              <Button size="sm" disabled={!engine.can('start')} onClick={() => engine.act('start')}>
                {t('library.start')}
              </Button>
            )}
            {active && (
              <Button
                size="sm"
                variant="outline"
                disabled={!engine.can(phase === 'paused' ? 'start' : 'pause')}
                onClick={() => engine.act(phase === 'paused' ? 'start' : 'pause')}
              >
                {t(phase === 'paused' ? 'library.resume' : 'library.pause')}
              </Button>
            )}
            {(active || phase === 'fault') && (
              <Button
                size="sm"
                variant="outline"
                disabled={!engine.can('stop')}
                onClick={() => engine.act('stop')}
              >
                {t('library.stop')}
              </Button>
            )}
          </div>
          {(phase === 'stopped' || phase === 'paused') && engine.startHint && (
            <p className="text-sm text-muted-foreground">{t(engine.startHint)}</p>
          )}
        </CardContent>
      </Card>
    </section>
  )
}
