import { useTranslation } from 'react-i18next'
import { isTauri } from '@tauri-apps/api/core'
import { FieldSet, FieldLegend } from '@/components/ui/field'
import { SettingsFields, SettingsRow } from './SettingsLayout'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Alert, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { usePreferences, useSavePreferences } from '@/services/preferences'
import type { AppPreferences } from '@/lib/tauri-bindings'

export function AppearanceSettings() {
  const { t } = useTranslation()
  const query = usePreferences()
  const save = useSavePreferences()
  const disabled = !query.data || query.isError || save.isPending
  return (
    <FieldSet>
      <FieldLegend>{t('preferences.appearance.title')}</FieldLegend>
      {(query.isError || save.isError) && (
        <Alert variant="destructive">
          <AlertTitle>{t('preferences.loadError')}</AlertTitle>
          <Button
            variant="outline"
            onClick={() => {
              save.reset()
              void query.refetch()
            }}
          >
            {t('common.retry')}
          </Button>
        </Alert>
      )}
      <SettingsFields>
        <SettingsRow
          id="preference-theme"
          label={t('preferences.appearance.theme')}
          disabled={disabled}
        >
          <NativeSelect
            id="preference-theme"
            value={query.data?.theme ?? 'system'}
            disabled={disabled}
            onChange={(event) =>
              save.mutate({ theme: event.target.value as AppPreferences['theme'] })
            }
          >
            {(['system', 'light', 'dark'] as const).map((value) => (
              <NativeSelectOption key={value} value={value}>
                {t(`preferences.appearance.theme.${value}`)}
              </NativeSelectOption>
            ))}
          </NativeSelect>
        </SettingsRow>
        <SettingsRow
          id="preference-language"
          label={t('preferences.appearance.language')}
          disabled={disabled || !isTauri()}
        >
          <NativeSelect
            id="preference-language"
            value={query.data?.language ?? 'system'}
            disabled={disabled || !isTauri()}
            onChange={(event) =>
              save.mutate({
                language:
                  event.target.value === 'system'
                    ? null
                    : (event.target.value as AppPreferences['language']),
              })
            }
          >
            <NativeSelectOption value="system">
              {t('preferences.appearance.language.system')}
            </NativeSelectOption>
            <NativeSelectOption value="en">English</NativeSelectOption>
            <NativeSelectOption value="fr">Français</NativeSelectOption>
            <NativeSelectOption value="ar">العربية</NativeSelectOption>
          </NativeSelect>
        </SettingsRow>
      </SettingsFields>
    </FieldSet>
  )
}
