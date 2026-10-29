import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { emit } from '@tauri-apps/api/event'
import { getDefaultQuickPaneShortcut, updateQuickPaneShortcut } from '@/services/quick-pane'
import { preferencesQueryKeys } from '@/services/preferences'
import { SettingsFields, SettingsRow } from './SettingsLayout'
import { ShortcutPicker } from './ShortcutPicker'
export function QuickPaneShortcut({
  value,
  disabled,
}: {
  value: string | null
  disabled: boolean
}) {
  const { t } = useTranslation()
  const client = useQueryClient()
  const defaults = useQuery({
    queryKey: ['quick-pane-default-shortcut'],
    queryFn: getDefaultQuickPaneShortcut,
    retry: false,
  })
  const update = useMutation({
    mutationFn: updateQuickPaneShortcut,
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: preferencesQueryKeys.all })
      try {
        await emit('preferences-changed')
      } catch {
        /* Saved state remains authoritative. */
      }
    },
  })
  return (
    <SettingsFields>
      <SettingsRow
        label={t('preferences.general.quickPaneShortcut')}
        description={t('preferences.general.quickPaneShortcutDescription')}
        descriptionId="quick-pane-shortcut-description"
        disabled={disabled || update.isPending}
      >
        {defaults.data && (
          <ShortcutPicker
            value={value ?? defaults.data}
            defaultValue={defaults.data}
            descriptionId="quick-pane-shortcut-description"
            disabled={disabled || update.isPending}
            onChange={(chord) => update.mutate(chord)}
          />
        )}
      </SettingsRow>
      {(defaults.isError || update.isError) && (
        <p role="alert" className="text-sm text-destructive">
          {t('shortcut.unavailable')}
        </p>
      )}
    </SettingsFields>
  )
}
