import { useTranslation } from 'react-i18next'
import { usePlatform } from '@/hooks/use-platform'
import { useNativePermissions } from '@/hooks/use-native-permissions'
import { useUIStore } from '@/store/ui-store'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { NativePermissionBody } from '@/components/preferences/NativePermissionGuide'

/**
 * Shown inside the app, never fullscreen, while macOS withholds input access.
 * It mirrors the system privacy panes, then disappears as soon as the grants
 * report as allowed.
 */
export function NativePermissionPrompt() {
  const { t } = useTranslation()
  const platform = usePlatform()
  const dismissed = useUIStore((state) => state.permissionPromptDismissed)
  const dismiss = useUIStore((state) => state.dismissPermissionPrompt)
  const guide = useNativePermissions(platform === 'macos')
  if (platform !== 'macos' || dismissed || !guide.permissionsMissing || !guide.capabilities)
    return null
  return (
    <Dialog
      open
      onOpenChange={(next) => {
        if (!next) dismiss()
      }}
    >
      <DialogContent className="max-w-xl" aria-describedby="native-permission-description">
        <DialogHeader>
          <DialogTitle>{t('product.permissions.title')}</DialogTitle>
          <DialogDescription id="native-permission-description">
            {t('product.permissions.description')}
          </DialogDescription>
        </DialogHeader>
        <NativePermissionBody
          permissions={guide.capabilities.permissions}
          openFailed={guide.openFailed}
          onOpen={guide.open}
        />
        <DialogFooter>
          <Button type="button" variant="outline" onClick={dismiss}>
            {t('product.permissions.later')}
          </Button>
          <Button type="button" disabled={guide.busy} onClick={guide.request}>
            {t('product.permissions.request')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
