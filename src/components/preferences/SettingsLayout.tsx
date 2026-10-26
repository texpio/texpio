import type { ReactNode } from 'react'
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from '@/components/ui/field'

export function SettingsFields({ children }: { children: ReactNode }) {
  return (
    <FieldGroup className="gap-0 overflow-hidden rounded-xl border bg-card">{children}</FieldGroup>
  )
}

export function SettingsRow({
  id,
  label,
  description,
  descriptionId,
  disabled,
  compact = false,
  children,
}: {
  id?: string
  label: ReactNode
  description?: ReactNode
  descriptionId?: string
  disabled?: boolean
  compact?: boolean
  children: ReactNode
}) {
  return (
    <Field
      orientation={compact ? 'horizontal' : 'responsive'}
      data-disabled={disabled}
      className="min-w-0 gap-4 border-b p-4 last:border-b-0"
    >
      <FieldContent className="min-w-0">
        <FieldLabel htmlFor={id}>{label}</FieldLabel>
        {description && <FieldDescription id={descriptionId}>{description}</FieldDescription>}
      </FieldContent>
      <div
        className={
          compact
            ? 'w-auto shrink-0'
            : 'w-full min-w-0 @md/field-group:basis-56 @md/field-group:shrink-0 [&>[data-slot=native-select-wrapper]]:w-full [&>button]:h-auto [&>button]:min-h-8 [&>button]:max-w-full [&>button]:whitespace-normal [&>button]:py-1.5'
        }
      >
        {children}
      </div>
    </Field>
  )
}
