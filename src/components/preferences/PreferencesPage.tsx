import { useTranslation } from 'react-i18next'
import { isTauri } from '@tauri-apps/api/core'
import { useUIStore, type PreferencesSection } from '@/store/ui-store'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { usePreferences } from '@/services/preferences'
import { useCommandContext } from '@/hooks/use-command-context'
import { runCommand } from '@/lib/commands'
import { LibraryBackups } from '@/features/library/LibraryBackups'
import { AppearanceSettings } from './AppearanceSettings'
import { ExpansionSettings } from './ExpansionSettings'
import { QuickPaneShortcut } from './QuickPaneShortcut'
import { LibraryLocation } from './LibraryLocation'

const sections: PreferencesSection[] = ['general', 'expansion', 'shortcuts', 'data']
export function PreferencesPage() {
  const { t } = useTranslation()
  const context = useCommandContext()
  const section = useUIStore((state) => state.preferencesSection)
  const choose = useUIStore((state) => state.setPreferencesSection)
  const preferences = usePreferences()
  return (
    <section aria-labelledby="preferences-heading" className="h-full overflow-y-auto">
      <div className="mx-auto flex max-w-4xl flex-col gap-7 p-6 lg:p-8">
        <header className="flex flex-col gap-1">
          <h1 id="preferences-heading" className="text-2xl font-semibold tracking-tight">
            {t('preferences.title')}
          </h1>
          <p className="text-sm text-muted-foreground">{t('preferences.pageDescription')}</p>
        </header>
        <ToggleGroup
          type="single"
          value={section}
          variant="outline"
          className="max-w-full flex-wrap"
          aria-label={t('preferences.sections')}
          onValueChange={(value) => {
            if (sections.includes(value as PreferencesSection)) choose(value as PreferencesSection)
          }}
        >
          {sections.map((value) => (
            <ToggleGroupItem key={value} value={value} className="flex-none px-3">
              {t(`preferences.section.${value}`)}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
        <div className="flex flex-col gap-7" aria-label={t(`preferences.section.${section}`)}>
          {section === 'general' && <AppearanceSettings />}
          {section === 'shortcuts' && isTauri() && (
            <>
              <QuickPaneShortcut
                value={preferences.data?.quick_pane_shortcut ?? null}
                disabled={!preferences.data || preferences.isError}
              />
            </>
          )}
          {section === 'data' && isTauri() && (
            <>
              <LibraryLocation />
              <Separator />
              <section className="flex flex-col items-start gap-3">
                <h2 className="font-medium">{t('backups.title')}</h2>
                <p className="text-sm text-muted-foreground">{t('preferences.dataHelp')}</p>
                <LibraryBackups />
              </section>
            </>
          )}
          {section !== 'general' && !isTauri() && (
            <p className="text-sm text-muted-foreground">{t('preferences.desktopOnly')}</p>
          )}
          {isTauri() && (
            <>
              {(section === 'general' || section === 'shortcuts') && <Separator />}
              <ExpansionSettings section={section} />
            </>
          )}
        </div>
        <footer className="flex items-center justify-between gap-4 border-t pt-5 text-sm text-muted-foreground">
          <span>Texpio {__APP_VERSION__}</span>
          {isTauri() && (
            <Button
              variant="ghost"
              onClick={() => {
                void runCommand('check-for-updates', context)
              }}
            >
              {t('menu.checkForUpdates')}
            </Button>
          )}
        </footer>
      </div>
    </section>
  )
}
