import { isTauri } from '@tauri-apps/api/core'

export { commands } from './bindings'
export type { AppPreferences, JsonValue, ServiceError, Theme } from './bindings'

export function unwrapResult<T, E>(
  result: { status: 'ok'; data: T } | { status: 'error'; error: E },
): T {
  if (result.status === 'ok') return result.data
  throw result.error
}

/** Never invoke native APIs from the browser preview. Callers redact errors for their UI. */
export function requireDesktop() {
  if (!isTauri()) throw new Error('Native desktop service is unavailable.')
}
