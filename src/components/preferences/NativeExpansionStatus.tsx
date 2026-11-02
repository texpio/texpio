import { useTranslation } from 'react-i18next'
import { usePlatform } from '@/hooks/use-platform'
import { useNativePermissions } from '@/hooks/use-native-permissions'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { NativePermissionGuide } from './NativePermissionGuide'

export function NativeExpansionStatus() {
  const { t } = useTranslation()
  const platform = usePlatform()
  const guide = useNativePermissions(platform === 'macos' || platform === 'linux')
  if (platform !== 'macos' && platform !== 'linux') return null
  const state = guide.unavailable ? 'unavailable' : guide.capabilities?.readiness
  if (!state) return null
  if (platform === 'macos' && state === 'permissionRequired' && guide.capabilities) {
    return (
      <NativePermissionGuide
        permissions={guide.capabilities.permissions}
        busy={guide.busy}
        checking={guide.checking}
        openFailed={guide.openFailed}
        onRequest={guide.request}
        onOpen={guide.open}
        onRecheck={guide.recheck}
      />
    )
  }
  return (
    <Alert>
      <AlertTitle>{t('product.platform.title')}</AlertTitle>
      <AlertDescription className="flex flex-col gap-2">
        {guide.capabilities && <p>{t(`product.platform.${guide.capabilities.backend}`)}</p>}
        <p>{t(`product.platform.${state}`)}</p>
        <div className="flex flex-wrap gap-2">
          {state === 'permissionRequired' && (
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={guide.busy}
              onClick={guide.request}
            >
              {t('product.platform.permissions')}
            </Button>
          )}
          <Button
            type="button"
            size="sm"
            variant="outline"
            disabled={guide.checking}
            onClick={guide.recheck}
          >
            {t('product.platform.recheck')}
          </Button>
        </div>
      </AlertDescription>
    </Alert>
  )
}
