import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { useUIStore } from '@/store/ui-store'
import {
  checkForUpdates,
  cancelUpdate,
  installUpdate,
  safeRelaunch,
  type UpdateCheckResult,
} from '@/lib/updater'
type State = UpdateCheckResult | { status: 'checking' | 'installing' | 'installed' }
function OpenUpdateDialog() {
  const { t } = useTranslation()
  const [state, setState] = useState<State>({ status: 'checking' })
  const [percent, setPercent] = useState<number | null>(null)
  const busy = state.status === 'installing'
  useEffect(() => {
    let active = true
    let started = false
    // StrictMode can replay setup/cleanup before this microtask. Only the live owner checks.
    void Promise.resolve().then(async () => {
      if (!active) return
      started = true
      const result = await checkForUpdates()
      if (active) setState(result)
      else void cancelUpdate()
    })
    return () => {
      active = false
      if (started) void cancelUpdate()
    }
  }, [])
  const close = () => {
    if (!busy) useUIStore.getState().setUpdateDialogOpen(false)
  }
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) close()
      }}
    >
      <DialogContent
        showCloseButton={!busy}
        onEscapeKeyDown={(event) => {
          if (busy) event.preventDefault()
        }}
        onInteractOutside={(event) => {
          if (busy) event.preventDefault()
        }}
      >
        <DialogHeader>
          <DialogTitle>{t('updates.title', { defaultValue: 'Updates' })}</DialogTitle>
          <DialogDescription>
            {t(`updates.${state.status}`, { defaultValue: state.status })}
          </DialogDescription>
        </DialogHeader>
        {state.status === 'available' && (
          <>
            <p>
              {t('updates.version', {
                version: state.version,
                defaultValue: 'Version {{version}} is available.',
              })}
            </p>
            <p className="text-sm text-muted-foreground">
              {t('updates.confirm', {
                defaultValue:
                  'Installation may restart Texpio. The expansion probe stops before installation.',
              })}
            </p>
          </>
        )}
        {busy && (
          <progress
            max={100}
            value={percent ?? undefined}
            className="h-2 w-full accent-primary"
            aria-label={t('updates.progress', { defaultValue: 'Downloading update' })}
          />
        )}
        {state.status === 'error' && (
          <p role="alert">
            {t('updates.failed', { defaultValue: 'Update unavailable. Try again.' })}
          </p>
        )}
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={close}>
            {t('common.cancel', { defaultValue: 'Cancel' })}
          </Button>
          {state.status === 'available' && (
            <Button
              onClick={() => {
                setState({ status: 'installing' })
                let total: number | undefined
                let downloaded = 0
                void installUpdate((event) => {
                  if (event.event === 'Started') total = event.data.contentLength
                  else if (event.event === 'Progress') {
                    downloaded += event.data.chunkLength
                    if (total && Number.isFinite(downloaded))
                      setPercent(Math.min(100, Math.round((downloaded / total) * 100)))
                  }
                }).then((result) =>
                  setState(
                    result.success
                      ? { status: 'installed' }
                      : { status: 'error', code: 'unavailable' },
                  ),
                )
              }}
            >
              {t('updates.install', { defaultValue: 'Download and install' })}
            </Button>
          )}
          {state.status === 'installed' && (
            <Button
              onClick={() => {
                void safeRelaunch().then((result) => {
                  if (!result.success) setState({ status: 'error', code: 'unavailable' })
                })
              }}
            >
              {t('error.restart', { defaultValue: 'Restart Texpio' })}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
export function UpdateDialog() {
  const open = useUIStore((state) => state.updateDialogOpen)
  return open ? <OpenUpdateDialog /> : null
}
