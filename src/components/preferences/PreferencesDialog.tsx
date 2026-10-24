import { useTranslation } from 'react-i18next'
import { isTauri } from '@tauri-apps/api/core'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Alert, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { usePreferences, useSavePreferences } from '@/services/preferences'
import { useUIStore } from '@/store/ui-store'
import type { AppPreferences } from '@/lib/tauri-bindings'
import { QuickPaneShortcut } from './QuickPaneShortcut'
import { ExpansionSettings } from './ExpansionSettings'
import { useCommandContext } from '@/hooks/use-command-context'
import { runCommand } from '@/lib/commands'

function PreferencesContent() {
  const context = useCommandContext()
  const { t } = useTranslation()
  const query = usePreferences()
  const save = useSavePreferences()
  const disabled = !query.data || query.isError || save.isPending
  return (
    <DialogContent className="max-h-[85svh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle>{t('preferences.title')}</DialogTitle>
        <DialogDescription>{t('preferences.description')}</DialogDescription>
      </DialogHeader>
      {(query.isError || save.isError) && (
        <Alert variant="destructive">
          <AlertTitle>
            {t('preferences.loadError', { defaultValue: 'Preferences are unavailable.' })}
          </AlertTitle>
          <Button
            variant="outline"
            onClick={() => {
              save.reset()
              void query.refetch()
            }}
          >
            {t('common.retry', { defaultValue: 'Retry' })}
          </Button>
        </Alert>
      )}
      <FieldGroup>
        {isTauri() && (
          <QuickPaneShortcut value={query.data?.quick_pane_shortcut ?? null} disabled={disabled} />
        )}
        <Field>
          <FieldLabel htmlFor="preference-theme">{t('preferences.appearance.theme')}</FieldLabel>
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
        </Field>
        <Field>
          <FieldLabel htmlFor="preference-language">
            {t('preferences.appearance.language')}
          </FieldLabel>
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
        </Field>
      </FieldGroup>
      {isTauri() && <ExpansionSettings />}
      <footer className="flex items-center justify-between gap-4 border-t pt-4 text-sm text-muted-foreground">
        <span>Texpio {__APP_VERSION__}</span>
        {isTauri() && (
          <Button
            variant="outline"
            onClick={() => {
              void runCommand('check-for-updates', context)
            }}
          >
            {t('menu.checkForUpdates')}
          </Button>
        )}
      </footer>
    </DialogContent>
  )
}
export function PreferencesDialog() {
  const open = useUIStore((state) => state.preferencesOpen)
  return (
    <Dialog open={open} onOpenChange={(value) => useUIStore.getState().setPreferencesOpen(value)}>
      {open && <PreferencesContent />}
    </Dialog>
  )
}
