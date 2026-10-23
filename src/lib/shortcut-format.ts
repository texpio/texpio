import type { AppPlatform } from '@/hooks/use-platform'
export function formatShortcut(shortcut: string, platform: AppPlatform) {
  const mac = platform === 'macos'
  return shortcut
    .split('+')
    .map((part) => {
      switch (part.toLowerCase()) {
        case 'commandorcontrol':
        case 'cmdorctrl':
          return mac ? '⌘' : 'Ctrl'
        case 'control':
        case 'ctrl':
          return mac ? '⌃' : 'Ctrl'
        case 'command':
        case 'cmd':
        case 'super':
        case 'meta':
          return mac ? '⌘' : 'Win'
        case 'shift':
          return mac ? '⇧' : 'Shift'
        case 'alt':
          return mac ? '⌥' : 'Alt'
        case 'period':
          return '.'
        case 'comma':
          return ','
        default:
          return part.replace(/^(Key|Digit)/, '')
      }
    })
    .join(mac ? '' : '+')
}
