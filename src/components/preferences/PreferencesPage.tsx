import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { isTauri } from '@tauri-apps/api/core'
import { Database, Keyboard, Search, SlidersHorizontal, Zap } from 'lucide-react'
import { useUIStore, type PreferencesSection } from '@/store/ui-store'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group'
import { Button } from '@/components/ui/button'
import { BrandLogo } from '@/components/brand/BrandLogo'
import { usePreferences } from '@/services/preferences'
import { useCommandContext } from '@/hooks/use-command-context'
import { runCommand } from '@/lib/commands'
import { LibraryBackups } from '@/features/library/LibraryBackups'
import { EngineControls } from '@/features/library/EngineControls'
import { AppearanceSettings } from './AppearanceSettings'
import { ExpansionSettings } from './ExpansionSettings'
import { QuickPaneShortcut } from './QuickPaneShortcut'
import { LibraryLocation } from './LibraryLocation'
import { SettingsFields, SettingsRow } from './SettingsLayout'

const sections = [
  {
    value: 'general',
    icon: SlidersHorizontal,
    labels: [
      'preferences.appearance.title',
      'preferences.appearance.theme',
      'preferences.appearance.language',
      'product.field.autostart',
      'product.field.closeToTray',
    ],
  },
  {
    value: 'expansion',
    icon: Zap,
    labels: [
      'library.engine',
      'library.pause',
      'library.stop',
      'product.field.insertion',
      'product.field.activation',
      'product.field.boundary',
      'product.field.typingSpeed',
      'product.field.cancelKey',
      'product.field.preserveTerminator',
      'product.field.restoreClipboard',
      'product.field.allowSoftwareInput',
      'product.field.attachmentDelayMs',
    ],
  },
  {
    value: 'shortcuts',
    icon: Keyboard,
    labels: [
      'preferences.general.quickPaneShortcut',
      'product.field.clipboardMode',
      'product.field.clipboardEnabled',
      'product.field.toggleShortcut',
      'product.field.clipboardShortcut',
    ],
  },
  {
    value: 'data',
    icon: Database,
    labels: [
      'location.title',
      'location.personal',
      'location.shared',
      'backups.title',
      'preferences.dataHelp',
    ],
  },
] as const

export function PreferencesPage() {
  const { t } = useTranslation()
  const context = useCommandContext()
  const section = useUIStore((state) => state.preferencesSection)
  const choose = useUIStore((state) => state.setPreferencesSection)
  const preferences = usePreferences()
  const [search, setSearch] = useState('')
  function matchingSections(value: string) {
    const words = value.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean)
    return sections.filter((item) => {
      const text = [t(`preferences.section.${item.value}`), ...item.labels.map((key) => t(key))]
        .join(' ')
        .toLocaleLowerCase()
      return words.every((word) => text.includes(word))
    })
  }
  const matches = matchingSections(search)
  return (
    <section
      aria-labelledby="preferences-heading"
      className="@container/preferences flex h-full min-h-0 overflow-hidden"
    >
      <aside className="flex w-48 min-w-0 shrink-0 flex-col border-e bg-sidebar/30 @3xl/preferences:w-56">
        <div className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto p-4">
          <h1 id="preferences-heading" className="text-xl font-semibold tracking-tight">
            {t('preferences.title')}
          </h1>
          <InputGroup>
            <InputGroupAddon>
              <Search aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput
              type="search"
              aria-label={t('preferences.search')}
              placeholder={t('preferences.search')}
              value={search}
              onChange={(event) => {
                const value = event.target.value
                setSearch(value)
                const next = matchingSections(value)
                if (next.length && !next.some((item) => item.value === section))
                  choose(next[0].value)
              }}
            />
          </InputGroup>
          <nav aria-label={t('preferences.sections')}>
            <ToggleGroup
              type="single"
              orientation="vertical"
              value={section}
              className="w-full flex-col items-stretch gap-1"
              aria-label={t('preferences.sections')}
              onValueChange={(value) => {
                if (matches.some((item) => item.value === value))
                  choose(value as PreferencesSection)
              }}
            >
              {matches.map((item) => (
                <ToggleGroupItem
                  key={item.value}
                  value={item.value}
                  className="w-full flex-none justify-start rounded-lg px-3 text-start"
                >
                  <item.icon aria-hidden="true" />
                  <span className="min-w-0 truncate">{t(`preferences.section.${item.value}`)}</span>
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
            {!matches.length && (
              <p role="status" className="py-3 text-sm text-muted-foreground">
                {t('preferences.noMatches')}
              </p>
            )}
          </nav>
        </div>
        <footer className="flex shrink-0 flex-col gap-2 border-t p-4 text-xs text-muted-foreground">
          <BrandLogo className="h-8 w-36" />
          <span>{__APP_VERSION__}</span>
          {isTauri() && (
            <Button
              variant="ghost"
              size="sm"
              className="justify-start px-0"
              onClick={() => {
                void runCommand('check-for-updates', context)
              }}
            >
              {t('menu.checkForUpdates')}
            </Button>
          )}
        </footer>
      </aside>
      <div className="min-w-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex w-full max-w-4xl flex-col gap-8 p-5 @3xl/preferences:p-8">
          <header className="flex flex-col gap-2">
            <h2 className="text-2xl font-semibold tracking-tight">
              {t(`preferences.section.${section}`)}
            </h2>
            <p className="text-sm text-muted-foreground">
              {t(`preferences.description.${section}`)}
            </p>
          </header>
          <div
            className="flex min-w-0 flex-col gap-8"
            aria-label={t(`preferences.section.${section}`)}
          >
            {section === 'general' && <AppearanceSettings />}
            {section === 'expansion' && isTauri() && <EngineControls />}
            {section === 'shortcuts' && isTauri() && (
              <QuickPaneShortcut
                value={preferences.data?.quick_pane_shortcut ?? null}
                disabled={!preferences.data || preferences.isError}
              />
            )}
            {section === 'data' && isTauri() && (
              <>
                <LibraryLocation />
                <SettingsFields>
                  <SettingsRow label={t('backups.title')} description={t('preferences.dataHelp')}>
                    <LibraryBackups />
                  </SettingsRow>
                </SettingsFields>
              </>
            )}
            {section !== 'general' && !isTauri() && (
              <p className="text-sm text-muted-foreground">{t('preferences.desktopOnly')}</p>
            )}
            {isTauri() && <ExpansionSettings key="product-settings" section={section} />}
          </div>
        </div>
      </div>
    </section>
  )
}
