import { commands, requireDesktop, unwrapResult } from '@/lib/tauri-bindings'
import type { ProductCapabilities } from '@/lib/bindings'

export function parseProductCapabilities(value: unknown): ProductCapabilities {
  if (!value || typeof value !== 'object')
    throw new Error('Native capability reply is unavailable.')
  const reply = value as ProductCapabilities
  if (
    !['windows', 'macos', 'linuxX11', 'linuxWayland'].includes(reply.backend) ||
    ![
      'ready',
      'permissionRequired',
      'unsupportedSession',
      'helperRequired',
      'bridgeRequired',
      'fieldEvidenceRequired',
      'unavailable',
    ].includes(reply.readiness) ||
    !['clipboardShortcut', 'autostart', 'richText', 'attachments'].every(
      (key) => typeof (value as Record<string, unknown>)[key] === 'boolean',
    )
  )
    throw new Error('Native capability reply is unavailable.')
  return reply
}
export async function getProductCapabilities() {
  requireDesktop()
  return parseProductCapabilities(unwrapResult(await commands.productCapabilities()))
}
export async function requestProductPermissions() {
  requireDesktop()
  return parseProductCapabilities(unwrapResult(await commands.productRequestPermissions()))
}
