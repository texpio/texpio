import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { open } from '@tauri-apps/plugin-dialog'
import { useTranslation } from 'react-i18next'
import { commands, unwrapResult } from '@/lib/tauri-bindings'
import { approveExit } from '@/lib/exit-guard'
import { Button } from '@/components/ui/button'
import { FieldDescription, FieldLegend, FieldSet } from '@/components/ui/field'
import { Alert, AlertTitle } from '@/components/ui/alert'
import { libraryKeys } from '@/services/library'
import { attachmentKeys } from '@/services/attachments'

const key = ['library-location'] as const
export function LibraryLocation() {
  const { t } = useTranslation()
  const client = useQueryClient()
  const query = useQuery({
    queryKey: key,
    queryFn: async () => unwrapResult(await commands.libraryLocation()),
    retry: false,
  })
  const change = useMutation({
    mutationFn: async (shared: boolean) => {
      const selected = shared ? await open({ multiple: false, directory: true }) : null
      if ((shared && (!selected || Array.isArray(selected))) || !(await approveExit())) return
      return unwrapResult(
        await commands.librarySwitch(typeof selected === 'string' ? selected : null),
      )
    },
    onSuccess: async (location) => {
      if (!location) return
      client.setQueryData(key, location)
      await Promise.all([
        client.invalidateQueries({ queryKey: libraryKeys.all }),
        client.invalidateQueries({ queryKey: attachmentKeys.all }),
      ])
    },
  })
  const disabled = change.isPending || query.isPending
  return (
    <FieldSet>
      <FieldLegend>{t('location.title')}</FieldLegend>
      <FieldDescription>{t('location.help')}</FieldDescription>
      {(query.isError || query.data?.problem || change.isError) && (
        <Alert variant="destructive">
          <AlertTitle>{t('location.error')}</AlertTitle>
        </Alert>
      )}
      <p className="text-sm font-medium">
        {t(query.data?.shared ? 'location.shared' : 'location.personal')}
      </p>
      <p className="break-all text-xs text-muted-foreground">{query.data?.directory}</p>
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="outline"
          disabled={disabled}
          onClick={() => change.mutate(true)}
        >
          {t('location.select')}
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={disabled || (!query.data?.shared && !query.data?.problem)}
          onClick={() => change.mutate(false)}
        >
          {t('location.usePersonal')}
        </Button>
      </div>
    </FieldSet>
  )
}
