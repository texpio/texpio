import { isTauri } from '@tauri-apps/api/core'
import { commands } from './tauri-bindings'
const events = [
  'appStarted',
  'renderFailed',
  'commandFailed',
  'notificationFailed',
  'recoverySaved',
  'recoveryFailed',
  'updateFailed',
  'unknown',
] as const
export type DiagnosticEvent = (typeof events)[number]
type Level = 'trace' | 'debug' | 'info' | 'warn' | 'error'
async function record(level: Level, event: DiagnosticEvent, metadata?: unknown) {
  try {
    const safeEvent = events.includes(event) ? event : 'unknown'
    const count =
      typeof metadata === 'object' && metadata !== null
        ? (metadata as { count?: unknown }).count
        : null
    const safeCount =
      typeof count === 'number' && Number.isInteger(count) && count >= 0 && count <= 0xffff_ffff
        ? count
        : null
    if (isTauri()) await commands.recordDiagnostic({ level, event: safeEvent, count: safeCount })
    else if (import.meta.env.DEV)
      console.info('[Texpio]', { level, event: safeEvent, count: safeCount })
  } catch {
    /* Diagnostics must never log the failure payload or recurse. */
  }
}
export const logger = {
  trace: (event: DiagnosticEvent, metadata?: unknown) => record('trace', event, metadata),
  debug: (event: DiagnosticEvent, metadata?: unknown) => record('debug', event, metadata),
  info: (event: DiagnosticEvent, metadata?: unknown) => record('info', event, metadata),
  warn: (event: DiagnosticEvent, metadata?: unknown) => record('warn', event, metadata),
  error: (event: DiagnosticEvent, metadata?: unknown) => record('error', event, metadata),
}
export const { trace, debug, info, warn, error } = logger
