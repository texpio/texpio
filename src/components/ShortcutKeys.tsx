import { Kbd, KbdGroup } from '@/components/ui/kbd'
import type { AppPlatform } from '@/hooks/use-platform'
import { formatShortcut } from '@/lib/shortcut-format'

/** Display a shortcut as keys while retaining a readable accessible label. */
export function ShortcutKeys({ shortcut, platform }: { shortcut: string; platform: AppPlatform }) {
  const parts = shortcut.split('+').filter(Boolean)
  return (
    <span className="inline-flex min-w-0 items-center" dir="ltr">
      <span className="sr-only">{formatShortcut(shortcut, platform)}</span>
      <KbdGroup aria-hidden="true" className="flex-wrap">
        {parts.map((part, index) => {
          const occurrence = parts.slice(0, index).filter((previous) => previous === part).length
          return (
            <span className="inline-flex items-center gap-1" key={`${part}-${occurrence}`}>
              {index > 0 && platform !== 'macos' && <span>+</span>}
              <Kbd>{formatShortcut(part, platform)}</Kbd>
            </span>
          )
        })}
      </KbdGroup>
    </span>
  )
}
