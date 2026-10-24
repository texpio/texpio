import { useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { approveExit } from '@/lib/exit-guard'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogCancel,
  AlertDialogAction,
} from '@/components/ui/alert-dialog'
import { Alert, AlertTitle } from '@/components/ui/alert'
import { Field, FieldLabel } from '@/components/ui/field'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import {
  discardBackup,
  exportBackup,
  importBackup,
  recoverLibrary,
  recoveryStatus,
  selectBackup,
  type BackupFormat,
  type BackupPreview,
  type ImportMode,
} from '@/services/backups'
import { libraryKeys } from '@/services/library'
import { attachmentKeys } from '@/services/attachments'

export function LibraryBackups() {
  const { t } = useTranslation()
  const client = useQueryClient()
  const [open, setOpen] = useState(false)
  const [format, setFormat] = useState<BackupFormat>('archive')
  const [mode, setMode] = useState<ImportMode>('merge')
  const [preview, setPreview] = useState<BackupPreview | null>(null)
  const [recover, setRecover] = useState(false)
  const [notice, setNotice] = useState<'error' | 'exported' | 'imported' | 'recovered' | null>(null)
  const recovery = useQuery({
    queryKey: ['library-recovery'],
    queryFn: recoveryStatus,
    enabled: open,
    retry: false,
  })
  const action = useMutation({
    mutationFn: (operation: () => Promise<void>) => operation(),
    onMutate: () => setNotice(null),
    onError: () => setNotice('error'),
  })
  const token = preview?.token
  useEffect(
    () => () => {
      if (token) void discardBackup(token).catch(() => {})
    },
    [token],
  )
  async function refresh() {
    await Promise.all([
      client.invalidateQueries({ queryKey: libraryKeys.all }),
      client.invalidateQueries({ queryKey: attachmentKeys.all }),
      client.invalidateQueries({ queryKey: ['library-metadata'] }),
      recovery.refetch(),
    ])
  }
  return (
    <>
      <Button type="button" variant="outline" onClick={() => setOpen(true)}>
        {t('backups.title')}
      </Button>
      <Dialog
        open={open}
        onOpenChange={(value) => {
          if (!action.isPending) {
            setOpen(value)
            if (!value) setPreview(null)
          }
        }}
      >
        <DialogContent className="max-h-[85svh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{t('backups.title')}</DialogTitle>
            <DialogDescription>{t('backups.help')}</DialogDescription>
          </DialogHeader>
          {notice && (
            <Alert variant={notice === 'error' ? 'destructive' : 'default'}>
              <AlertTitle>{t(`backups.${notice}`)}</AlertTitle>
            </Alert>
          )}
          <Field>
            <FieldLabel htmlFor="backup-format">{t('backups.format')}</FieldLabel>
            <NativeSelect
              id="backup-format"
              value={format}
              disabled={action.isPending}
              onChange={(event) => setFormat(event.target.value as BackupFormat)}
            >
              <NativeSelectOption value="archive">ZIP</NativeSelectOption>
              <NativeSelectOption value="json">JSON</NativeSelectOption>
              <NativeSelectOption value="yaml">YAML</NativeSelectOption>
            </NativeSelect>
          </Field>
          <Button
            type="button"
            variant="outline"
            disabled={action.isPending}
            onClick={() =>
              action.mutate(async () => {
                if (await exportBackup(format)) setNotice('exported')
              })
            }
          >
            {t('backups.export')}
          </Button>
          <Field>
            <FieldLabel htmlFor="backup-mode">{t('backups.mode')}</FieldLabel>
            <NativeSelect
              id="backup-mode"
              value={mode}
              disabled={action.isPending || !!preview}
              onChange={(event) => setMode(event.target.value as ImportMode)}
            >
              <NativeSelectOption value="merge">{t('backups.merge')}</NativeSelectOption>
              <NativeSelectOption value="replace">{t('backups.replace')}</NativeSelectOption>
            </NativeSelect>
          </Field>
          <p className="text-xs text-muted-foreground">{t('backups.mergeHelp')}</p>
          <Button
            type="button"
            variant="outline"
            disabled={action.isPending}
            onClick={() =>
              action.mutate(async () => {
                const selected = await selectBackup(mode)
                if (selected) setPreview(selected)
              })
            }
          >
            {t('backups.choose')}
          </Button>
          {preview && (
            <section className="flex flex-col gap-3 rounded-md border p-3">
              <p>{t('backups.counts', preview)}</p>
              {preview.replace && (
                <p className="text-sm text-destructive">{t('backups.replaceHelp')}</p>
              )}
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant={preview.replace ? 'destructive' : 'default'}
                  disabled={action.isPending}
                  onClick={() =>
                    action.mutate(async () => {
                      if (!(await approveExit())) return
                      await importBackup(preview.token)
                      setPreview(null)
                      await refresh()
                      setNotice('imported')
                    })
                  }
                >
                  {t(preview.replace ? 'backups.replace' : 'backups.confirmMerge')}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  disabled={action.isPending}
                  onClick={() => setPreview(null)}
                >
                  {t('library.cancel')}
                </Button>
              </div>
            </section>
          )}
          <p className="text-sm text-muted-foreground">{t('backups.recoveryHelp')}</p>
          <Button
            type="button"
            variant="outline"
            disabled={action.isPending || !recovery.data?.canRecover}
            onClick={() => setRecover(true)}
          >
            {t('backups.recover')}
          </Button>
        </DialogContent>
      </Dialog>
      <AlertDialog
        open={recover}
        onOpenChange={(value) => {
          if (!action.isPending) setRecover(value)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('backups.recover')}</AlertDialogTitle>
            <AlertDialogDescription>{t('backups.recoveryConfirm')}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={action.isPending}>{t('library.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              disabled={action.isPending}
              onClick={(event) => {
                event.preventDefault()
                action.mutate(async () => {
                  if (!(await approveExit())) return
                  await recoverLibrary()
                  setRecover(false)
                  await refresh()
                  setNotice('recovered')
                })
              }}
            >
              {t('backups.recover')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
