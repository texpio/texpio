import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { loadProductSettings, saveProductSettings } from '@/services/product'
import type { ProductSettings } from '@/lib/bindings'
import { Field, FieldGroup, FieldLabel, FieldSet, FieldLegend } from '@/components/ui/field'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { Alert, AlertTitle, AlertDescription } from '@/components/ui/alert'

const key = ['expansion-settings'] as const
const optionLabels = {
  paste: 'product.option.paste',
  type: 'product.option.type',
  word: 'product.option.word',
  anywhere: 'product.option.anywhere',
  fast: 'product.option.fast',
  balanced: 'product.option.balanced',
  careful: 'product.option.careful',
} as const
export function ExpansionSettings() {
  const { t } = useTranslation()
  const client = useQueryClient()
  const query = useQuery({ queryKey: key, queryFn: loadProductSettings, retry: false })
  const save = useMutation({
    mutationFn: (settings: ProductSettings) => saveProductSettings(settings),
    onSuccess: (settings) => client.setQueryData(key, settings),
  })
  const disabled = !query.data || query.isError || save.isPending
  function update(patch: Partial<ProductSettings>) {
    if (query.data) save.mutate({ ...query.data, ...patch })
  }
  const choices = [
    { name: 'insertion', values: ['paste', 'type'] },
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
  return (
    <FieldSet>
      <FieldLegend>{t('product.settings')}</FieldLegend>
      {(query.isError || save.isError) && (
        <Alert variant="destructive">
          <AlertTitle>{t('product.settingsError')}</AlertTitle>
        </Alert>
      )}
      <FieldGroup>
        {choices.map((field) => (
          <Field key={field.name}>
            <FieldLabel htmlFor={`expansion-${field.name}`}>
              {t(`product.field.${field.name}`)}
            </FieldLabel>
            <NativeSelect
              id={`expansion-${field.name}`}
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
          </Field>
        ))}
        {(
          [
            'preserveTerminator',
            'restoreClipboard',
            'clipboardEnabled',
            'enabled',
            'autostart',
            'closeToTray',
          ] as const
        ).map((name) => (
          <Field key={name} orientation="horizontal">
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
              disabled={disabled}
              onCheckedChange={(value) => update({ [name]: value })}
            />
          </Field>
        ))}
        {(['toggleShortcut', 'clipboardShortcut'] as const).map((name) => (
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
        <p className="text-xs text-muted-foreground">{t('product.desktopHelp')}</p>
        <Field>
          <FieldLabel htmlFor="expansion-delay">{t('product.field.attachmentDelayMs')}</FieldLabel>
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
      </FieldGroup>
      {(query.data?.insertion === 'type' || query.data?.clipboardMode === 'type') && (
        <Alert>
          <AlertDescription>{t('product.typeNotice')}</AlertDescription>
        </Alert>
      )}
    </FieldSet>
  )
}
