import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { loadProductSettings, saveProductSettings } from '@/services/product'
import type { ProductSettings } from '@/lib/bindings'
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldSet,
  FieldLegend,
} from '@/components/ui/field'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { Alert, AlertTitle, AlertDescription } from '@/components/ui/alert'

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
      'enabled',
      'autostart',
      'closeToTray',
    ] as const
  ).filter(
    (name) =>
      section === 'all' ||
      (section === 'general'
        ? ['enabled', 'autostart', 'closeToTray'].includes(name)
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
      <FieldGroup>
        {shownChoices.map((field) => (
          <Field key={field.name}>
            <FieldLabel htmlFor={`expansion-${field.name}`}>
              {t(`product.field.${field.name}`)}
            </FieldLabel>
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
            {field.name === 'activation' && (
              <FieldDescription id="expansion-activation-help">
                {t('product.activationHelp')}
              </FieldDescription>
            )}
          </Field>
        ))}
        {shownSwitches.map((name) => (
          <Field
            key={name}
            orientation="horizontal"
            data-disabled={name === 'preserveTerminator' ? terminatorDisabled : disabled}
          >
            <FieldLabel htmlFor={`expansion-${name}`}>{t(`product.field.${name}`)}</FieldLabel>
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
          </Field>
        ))}
        {(section === 'all' || section === 'expansion') && (
          <Field orientation="horizontal" data-disabled={disabled}>
            <FieldContent>
              <FieldLabel htmlFor="expansion-allowSoftwareInput">
                {t('product.field.allowSoftwareInput')}
              </FieldLabel>
              <FieldDescription id="expansion-software-input-help">
                {t('product.softwareInputHelp')}
              </FieldDescription>
            </FieldContent>
            <Switch
              id="expansion-allowSoftwareInput"
              aria-describedby="expansion-software-input-help"
              checked={query.data?.allowSoftwareInput ?? false}
              disabled={disabled}
              onCheckedChange={(value) => update({ allowSoftwareInput: value })}
            />
          </Field>
        )}
        {(section === 'all' || section === 'shortcuts') &&
          (['toggleShortcut', 'clipboardShortcut'] as const).map((name) => (
            <Field key={name}>
              <FieldLabel htmlFor={`expansion-${name}`}>{t(`product.field.${name}`)}</FieldLabel>
              <Input
                id={`expansion-${name}`}
                disabled={disabled}
                key={query.data?.[name]}
                defaultValue={query.data?.[name] ?? ''}
                maxLength={128}
                onBlur={(event) => {
                  const value = event.currentTarget.value.trim()
                  if (value && value !== query.data?.[name]) update({ [name]: value })
                  else event.currentTarget.value = query.data?.[name] ?? ''
                }}
              />
            </Field>
          ))}
        {(section === 'all' || section === 'general') && (
          <p className="text-xs text-muted-foreground">{t('product.desktopHelp')}</p>
        )}
        {(section === 'all' || section === 'expansion') && (
          <Field>
            <FieldLabel htmlFor="expansion-delay">
              {t('product.field.attachmentDelayMs')}
            </FieldLabel>
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
          </Field>
        )}
      </FieldGroup>
      {section !== 'general' &&
        (query.data?.insertion === 'type' || query.data?.clipboardMode === 'type') && (
          <Alert>
            <AlertDescription>{t('product.typeNotice')}</AlertDescription>
          </Alert>
        )}
    </FieldSet>
  )
}
