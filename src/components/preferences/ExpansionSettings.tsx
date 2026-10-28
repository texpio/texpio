import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { loadProductSettings, saveProductSettings } from '@/services/product'
import type { ProductSettings } from '@/lib/bindings'
import { FieldDescription, FieldSet, FieldLegend } from '@/components/ui/field'
import { SettingsFields, SettingsRow } from './SettingsLayout'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { Alert, AlertTitle, AlertDescription } from '@/components/ui/alert'
import { ShortcutKeys } from '@/components/ShortcutKeys'
import { usePlatform } from '@/hooks/use-platform'

function ShortcutInput({
  id,
  value,
  disabled,
  onCommit,
}: {
  id: string
  value: string
  disabled: boolean
  onCommit: (value: string) => void
}) {
  const platform = usePlatform()
  const [draft, setDraft] = useState(value)
  return (
    <div className="group relative min-w-0 w-full">
      <Input
        id={id}
        disabled={disabled}
        value={draft}
        className="text-transparent transition-[box-shadow] focus:text-foreground"
        maxLength={128}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={() => {
          const next = draft.trim()
          if (next && next !== value) onCommit(next)
          else setDraft(value)
        }}
      />
      <span
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 flex items-center overflow-hidden px-3 group-focus-within:invisible"
      >
        <ShortcutKeys shortcut={draft} platform={platform} />
      </span>
    </div>
  )
}

const key = ['expansion-settings'] as const
const optionLabels = {
  paste: 'product.option.paste',
  type: 'product.option.type',
  immediate: 'product.option.immediate',
  terminator: 'product.option.terminator',
  word: 'product.option.word',
  anywhere: 'product.option.anywhere',
  fast: 'product.option.fast',
  balanced: 'product.option.balanced',
  careful: 'product.option.careful',
} as const
export function ExpansionSettings({
  section = 'all',
}: {
  section?: 'all' | 'general' | 'expansion' | 'shortcuts' | 'data'
}) {
  const { t } = useTranslation()
  const client = useQueryClient()
  const query = useQuery({ queryKey: key, queryFn: loadProductSettings, retry: false })
  const save = useMutation({
    mutationFn: (settings: ProductSettings) => saveProductSettings(settings),
    onSuccess: (settings) => client.setQueryData(key, settings),
  })
  const disabled = !query.data || query.isError || save.isPending
  const terminatorDisabled = disabled || query.data?.activation !== 'terminator'
  function update(patch: Partial<ProductSettings>) {
    if (query.data) save.mutate({ ...query.data, ...patch })
  }
  const choices = [
    { name: 'insertion', values: ['paste', 'type'] },
    { name: 'activation', values: ['immediate', 'terminator'] },
    { name: 'boundary', values: ['word', 'anywhere'] },
    { name: 'typingSpeed', values: ['fast', 'balanced', 'careful'] },
    { name: 'clipboardMode', values: ['paste', 'type'] },
    {
      name: 'cancelKey',
      values: [
        'Escape',
        'Pause',
        'ScrollLock',
        ...Array.from({ length: 12 }, (_, index) => `F${index + 1}`),
      ],
    },
  ] as const
  const shownChoices = choices.filter(
    (field) =>
      section === 'all' ||
      (section === 'shortcuts'
        ? field.name === 'clipboardMode'
        : section === 'expansion' && field.name !== 'clipboardMode'),
  )
  const shownSwitches = (
    [
      'preserveTerminator',
      'restoreClipboard',
      'clipboardEnabled',
      'autostart',
      'closeToTray',
    ] as const
  ).filter(
    (name) =>
      section === 'all' ||
      (section === 'general'
        ? ['autostart', 'closeToTray'].includes(name)
        : section === 'shortcuts'
          ? name === 'clipboardEnabled'
          : section === 'expansion' && ['preserveTerminator', 'restoreClipboard'].includes(name)),
  )
  // Keep this mutation owner alive even while Data or another main view is shown.
  if (section === 'data' && !query.isError && !save.isError) return null
  return (
    <FieldSet>
      <FieldLegend>
        {t(
          section === 'all' || section === 'data'
            ? 'product.settings'
            : `preferences.group.${section}`,
        )}
      </FieldLegend>
      {(query.isError || save.isError) && (
        <Alert variant="destructive">
          <AlertTitle>
            {t(
              save.error && 'code' in save.error && save.error.code === 'shortcutUnavailable'
                ? 'product.error.shortcutUnavailable'
                : 'product.settingsError',
            )}
          </AlertTitle>
        </Alert>
      )}
      <SettingsFields>
        {shownChoices.map((field) => (
          <SettingsRow
            key={field.name}
            id={`expansion-${field.name}`}
            label={t(`product.field.${field.name}`)}
            disabled={disabled}
            description={field.name === 'activation' ? t('product.activationHelp') : undefined}
            descriptionId={field.name === 'activation' ? 'expansion-activation-help' : undefined}
          >
            <NativeSelect
              id={`expansion-${field.name}`}
              aria-describedby={
                field.name === 'activation' ? 'expansion-activation-help' : undefined
              }
              disabled={disabled}
              value={query.data?.[field.name] ?? field.values[0]}
              onChange={(event) => update({ [field.name]: event.target.value })}
            >
              {field.values.map((value) => (
                <NativeSelectOption key={value} value={value}>
                  {field.name === 'cancelKey'
                    ? value
                    : t(optionLabels[value as keyof typeof optionLabels])}
                </NativeSelectOption>
              ))}
            </NativeSelect>
          </SettingsRow>
        ))}
        {shownSwitches.map((name) => (
          <SettingsRow
            key={name}
            id={`expansion-${name}`}
            label={t(`product.field.${name}`)}
            compact
            disabled={name === 'preserveTerminator' ? terminatorDisabled : disabled}
          >
            <Switch
              id={`expansion-${name}`}
              checked={
                query.data?.[name] ??
                (name === 'preserveTerminator' ||
                  name === 'restoreClipboard' ||
                  name === 'clipboardEnabled' ||
                  name === 'closeToTray')
              }
              disabled={name === 'preserveTerminator' ? terminatorDisabled : disabled}
              onCheckedChange={(value) => update({ [name]: value })}
            />
          </SettingsRow>
        ))}
        {(section === 'all' || section === 'expansion') && (
          <SettingsRow
            id="expansion-allowSoftwareInput"
            label={t('product.field.allowSoftwareInput')}
            description={t('product.softwareInputHelp')}
            descriptionId="expansion-software-input-help"
            compact
            disabled={disabled}
          >
            <Switch
              id="expansion-allowSoftwareInput"
              aria-describedby="expansion-software-input-help"
              checked={query.data?.allowSoftwareInput ?? false}
              disabled={disabled}
              onCheckedChange={(value) => update({ allowSoftwareInput: value })}
            />
          </SettingsRow>
        )}
        {(section === 'all' || section === 'shortcuts') &&
          (['toggleShortcut', 'clipboardShortcut'] as const).map((name) => (
            <SettingsRow
              key={name}
              id={`expansion-${name}`}
              label={t(`product.field.${name}`)}
              disabled={disabled}
            >
              <ShortcutInput
                id={`expansion-${name}`}
                disabled={disabled}
                key={`${query.data?.[name] ?? ''}:${save.isError}`}
                value={query.data?.[name] ?? ''}
                onCommit={(value) => update({ [name]: value })}
              />
            </SettingsRow>
          ))}
        {(section === 'all' || section === 'expansion') && (
          <SettingsRow
            id="expansion-delay"
            label={t('product.field.attachmentDelayMs')}
            disabled={disabled}
          >
            <Input
              id="expansion-delay"
              type="number"
              min={100}
              max={5000}
              step={100}
              disabled={disabled}
              key={query.data?.attachmentDelayMs}
              defaultValue={query.data?.attachmentDelayMs ?? 500}
              onBlur={(event) => {
                const value = event.currentTarget.valueAsNumber
                if (Number.isInteger(value) && value >= 100 && value <= 5000)
                  update({ attachmentDelayMs: value })
                else event.currentTarget.value = String(query.data?.attachmentDelayMs ?? 500)
              }}
            />
          </SettingsRow>
        )}
      </SettingsFields>
      {(section === 'all' || section === 'general') && (
        <FieldDescription>{t('product.desktopHelp')}</FieldDescription>
      )}
      {section !== 'general' &&
        (query.data?.insertion === 'type' || query.data?.clipboardMode === 'type') && (
          <Alert>
            <AlertDescription>{t('product.typeNotice')}</AlertDescription>
          </Alert>
        )}
    </FieldSet>
  )
}
