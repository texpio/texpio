import { useEffect, useRef } from 'react'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useTranslation } from 'react-i18next'
import { ArrowDown, ArrowUp, Paperclip, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldLabel, FieldSet, FieldLegend } from '@/components/ui/field'
import { Switch } from '@/components/ui/switch'
import { Alert, AlertTitle } from '@/components/ui/alert'
import {
  addDroppedFiles,
  addSelectedPaths,
  attachmentKeys,
  listAttachments,
  loadFileSettings,
  removeAttachment,
  reorderAttachments,
  saveFileSettings,
  selectAttachments,
} from '@/services/attachments'

export function AttachmentEditor({
  id,
  revision,
  disabled,
}: {
  id: string
  revision: number
  disabled: boolean
}) {
  const { t } = useTranslation()
  const client = useQueryClient()
  const zone = useRef<HTMLFieldSetElement>(null)
  const key = attachmentKeys.snippet(id, revision)
  const files = useQuery({
    queryKey: [...key, 'files'],
    queryFn: () => listAttachments(id),
    retry: false,
  })
  const settings = useQuery({
    queryKey: [...key, 'settings'],
    queryFn: () => loadFileSettings(id),
    retry: false,
  })
  const mutation = useMutation({
    mutationFn: (operation: () => Promise<unknown>) => operation(),
    onSettled: () => client.invalidateQueries({ queryKey: key }),
  })
  useEffect(() => {
    if (!isTauri()) return
    let cancelled = false
    let stop: (() => void) | undefined
    void listen('library-files-changed', () => {
      void client.invalidateQueries({ queryKey: attachmentKeys.all })
    }).then(
      (unlisten) => {
        if (cancelled) unlisten()
        else stop = unlisten
      },
      () => {},
    )
    return () => {
      cancelled = true
      stop?.()
    }
  }, [client])
  const busy = disabled || mutation.isPending || !files.data || !settings.data
  const { mutate } = mutation
  useEffect(() => {
    if (!isTauri() || busy) return
    let cancelled = false
    let stop: (() => void) | undefined
    try {
      void getCurrentWebview()
        .onDragDropEvent(({ payload }) => {
          if (cancelled || payload.type !== 'drop') return
          const rect = zone.current?.getBoundingClientRect()
          const x = payload.position.x / window.devicePixelRatio
          const y = payload.position.y / window.devicePixelRatio
          if (rect && x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom)
            mutate(() => addSelectedPaths(id, revision, payload.paths))
        })
        .then(
          (unlisten) => {
            if (cancelled) unlisten()
            else stop = unlisten
          },
          () => {},
        )
    } catch {
      /* Native drop events are unavailable in browser previews. */
    }
    return () => {
      cancelled = true
      stop?.()
    }
  }, [busy, id, revision, mutate])
  function move(index: number, offset: number) {
    const order = files.data!.map((file) => file.id)
    ;[order[index], order[index + offset]] = [order[index + offset], order[index]]
    mutation.mutate(() => reorderAttachments(id, revision, order))
  }
  return (
    <FieldSet
      ref={zone}
      onPaste={(event) => {
        const pasted = Array.from(event.clipboardData.files)
        if (!busy && pasted.length) {
          event.preventDefault()
          mutation.mutate(() => addDroppedFiles(id, revision, pasted))
        }
      }}
    >
      <FieldLegend>{t('attachments.title')}</FieldLegend>
      <FieldDescription>{t('attachments.help')}</FieldDescription>
      {(files.isError || settings.isError || mutation.isError) && (
        <Alert variant="destructive">
          <AlertTitle>{t('attachments.error')}</AlertTitle>
        </Alert>
      )}
      <Button
        type="button"
        variant="outline"
        disabled={busy}
        onClick={() => mutation.mutate(() => selectAttachments(id, revision))}
      >
        <Paperclip data-icon="inline-start" />
        {t('attachments.add')}
      </Button>
      <ul className="flex flex-col gap-2">
        {files.data?.map((file, index) => (
          <li key={file.id} className="flex items-center gap-2 rounded-md border p-2 text-sm">
            <span className="min-w-0 flex-1 break-words">
              {file.name}
              {!file.present && (
                <span className="block text-destructive">{t('attachments.missing')}</span>
              )}
            </span>
            <Button
              type="button"
              size="icon-sm"
              variant="ghost"
              aria-label={t('attachments.up', { name: file.name })}
              disabled={busy || index === 0}
              onClick={() => move(index, -1)}
            >
              <ArrowUp />
            </Button>
            <Button
              type="button"
              size="icon-sm"
              variant="ghost"
              aria-label={t('attachments.down', { name: file.name })}
              disabled={busy || index === files.data!.length - 1}
              onClick={() => move(index, 1)}
            >
              <ArrowDown />
            </Button>
            <Button
              type="button"
              size="icon-sm"
              variant="ghost"
              aria-label={t('attachments.remove', { name: file.name })}
              disabled={busy}
              onClick={() => mutation.mutate(() => removeAttachment(id, revision, file.id))}
            >
              <X />
            </Button>
          </li>
        ))}
      </ul>
      {(['attachmentsFirst', 'strictOrder'] as const).map((field) => (
        <Field key={field} orientation="horizontal">
          <FieldLabel htmlFor={`${id}-${field}`}>{t(`attachments.${field}`)}</FieldLabel>
          <Switch
            id={`${id}-${field}`}
            disabled={busy}
            checked={settings.data?.[field] ?? false}
            onCheckedChange={(checked) =>
              mutation.mutate(() =>
                saveFileSettings(id, revision, { ...settings.data!, [field]: checked }),
              )
            }
          />
        </Field>
      ))}
    </FieldSet>
  )
}
