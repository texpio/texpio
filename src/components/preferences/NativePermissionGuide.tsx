import { useTranslation } from 'react-i18next'
import { ExternalLink } from 'lucide-react'
import type { NativePermissions } from '@/lib/bindings'
import type { PermissionPane } from '@/services/product-platform'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

type BodyProps = {
  permissions: NativePermissions
  openFailed: boolean
  onOpen: (pane: PermissionPane) => void
}

/**
 * Per-grant status plus the steps macOS requires. Shared by the prompt shown at
 * launch and the permanent card in Settings so both stay identical.
 */
export function NativePermissionBody({ permissions, openFailed, onOpen }: BodyProps) {
  const { t } = useTranslation()
  const rows: {
    pane: PermissionPane
    granted: boolean
    label: 'product.permissions.accessibility' | 'product.permissions.inputMonitoring'
    detail: 'product.permissions.accessibilityDetail' | 'product.permissions.inputMonitoringDetail'
  }[] = [
    {
      pane: 'accessibility',
      granted: permissions.accessibility,
      label: 'product.permissions.accessibility',
      detail: 'product.permissions.accessibilityDetail',
    },
  ]
  // Accessibility trust normally covers listening too. Only surface Input
  // Monitoring when macOS withholds listening from an Accessibility-trusted
  // process, so the first-run prompt asks for one grant, not two.
  if (permissions.accessibility && !permissions.listen) {
    rows.push({
      pane: 'inputMonitoring',
      granted: false,
      label: 'product.permissions.inputMonitoring',
      detail: 'product.permissions.inputMonitoringDetail',
    })
  }
  return (
    <div className="flex flex-col gap-3">
      <ul className="flex flex-col gap-2">
        {rows.map((row) => (
          <li
            key={row.pane}
            className="flex flex-col gap-2 rounded-md border border-border p-3 sm:flex-row sm:items-center sm:justify-between"
          >
            <div className="flex flex-col gap-1">
              <div className="flex flex-wrap items-center gap-2">
                <span className="text-sm font-medium">{t(row.label)}</span>
                <Badge variant={row.granted ? 'secondary' : 'outline'}>
                  {t(row.granted ? 'product.permissions.allowed' : 'product.permissions.needed')}
                </Badge>
              </div>
              <p className="text-xs text-muted-foreground">{t(row.detail)}</p>
            </div>
            <Button type="button" size="sm" variant="outline" onClick={() => onOpen(row.pane)}>
              {t('product.permissions.open')}
              <ExternalLink aria-hidden />
            </Button>
          </li>
        ))}
      </ul>
      <ol className="flex list-decimal flex-col gap-1 ps-5 text-xs text-muted-foreground">
        <li>{t('product.permissions.stepEnable')}</li>
        <li>{t('product.permissions.stepToggle')}</li>
        <li>{t('product.permissions.stepRecheck')}</li>
      </ol>
      {openFailed && (
        <p role="alert" className="text-xs text-destructive">
          {t('product.permissions.openFailed')}
        </p>
      )}
      <p className="text-xs text-muted-foreground">{t('product.permissions.relaunch')}</p>
    </div>
  )
}

type Props = BodyProps & {
  busy: boolean
  checking: boolean
  onRequest: () => void
  onRecheck: () => void
}

/** The permanent Settings card for missing macOS input grants. */
export function NativePermissionGuide({
  permissions,
  busy,
  checking,
  openFailed,
  onRequest,
  onOpen,
  onRecheck,
}: Props) {
  const { t } = useTranslation()
  return (
    <Card className="gap-3">
      <CardHeader>
        <CardTitle>{t('product.permissions.title')}</CardTitle>
        <CardDescription>{t('product.permissions.description')}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <NativePermissionBody permissions={permissions} openFailed={openFailed} onOpen={onOpen} />
        <div className="flex flex-wrap gap-2">
          <Button type="button" size="sm" disabled={busy} onClick={onRequest}>
            {t('product.permissions.request')}
          </Button>
          <Button type="button" size="sm" variant="outline" disabled={checking} onClick={onRecheck}>
            {t('product.platform.recheck')}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
