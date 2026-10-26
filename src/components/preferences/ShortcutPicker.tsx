import { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { usePlatform } from '@/hooks/use-platform'
import { formatShortcut } from '@/lib/shortcut-format'
export function ShortcutPicker({
  value,
  defaultValue,
  onChange,
  disabled = false,
}: {
  value: string
  defaultValue: string
  onChange: (value: string) => void
  disabled?: boolean
}) {
  const { t } = useTranslation()
  const platform = usePlatform()
  const [capturing, setCapturing] = useState(false)
  const pending = useRef<string | null>(null)
  const cancel = () => {
    pending.current = null
    setCapturing(false)
  }
  return (
    <div className="flex flex-wrap items-center gap-2">
      <Button
        type="button"
        variant="outline"
        className="h-auto min-h-8 max-w-full whitespace-normal py-1.5 text-start"
        disabled={disabled}
        aria-pressed={capturing}
        aria-label={`${t('preferences.general.quickPaneShortcut')}: ${formatShortcut(value.replace('CommandOrControl', 'CmdOrCtrl'), platform)}`}
        onClick={() => {
          pending.current = null
          setCapturing(true)
        }}
        onBlur={cancel}
        onKeyDown={(event) => {
          if (!capturing) return
          event.preventDefault()
          event.stopPropagation()
          if (event.key === 'Escape') {
            cancel()
            return
          }
          if (
            event.repeat ||
            event.nativeEvent.isComposing ||
            ['Control', 'Shift', 'Meta', 'Alt'].includes(event.key)
          )
            return
          const parts: string[] = []
          if (platform === 'macos') {
            if (event.metaKey) parts.push('CommandOrControl')
            if (event.ctrlKey) parts.push('Control')
          } else {
            if (event.ctrlKey) parts.push('CommandOrControl')
            if (event.metaKey) parts.push('Super')
          }
          if (event.altKey) parts.push('Alt')
          if (!parts.length || !event.code) return
          if (event.shiftKey) parts.push('Shift')
          const key = event.code.replace(/^(Key|Digit)/, '')
          pending.current = [...parts, key].join('+')
        }}
        onKeyUp={(event) => {
          if (!capturing) return
          event.preventDefault()
          event.stopPropagation()
          if (pending.current) {
            const chord = pending.current
            cancel()
            onChange(chord)
          }
        }}
      >
        {capturing
          ? t('shortcut.capture')
          : formatShortcut(value.replace('CommandOrControl', 'CmdOrCtrl'), platform)}
      </Button>
      <Button
        type="button"
        variant="ghost"
        disabled={disabled || value === defaultValue}
        onClick={() => onChange(defaultValue)}
      >
        {t('shortcut.reset')}
      </Button>
    </div>
  )
}
