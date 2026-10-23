import { commands, requireDesktop, unwrapResult, type JsonValue } from './tauri-bindings'
import { logger } from './logger'
export async function saveEmergencyData(filename: string, data: JsonValue) {
  try {
    requireDesktop()
    unwrapResult(await commands.saveEmergencyData(filename, data))
  } catch {
    throw new Error('Recovery data is unavailable.')
  }
}
export async function loadEmergencyData(filename: string): Promise<JsonValue | null> {
  try {
    requireDesktop()
    const result = await commands.loadEmergencyData(filename)
    if (result.status === 'error' && result.error === 'notFound') return null
    return unwrapResult(result)
  } catch {
    throw new Error('Recovery data is unavailable.')
  }
}
export async function cleanupOldRecoveryFiles() {
  try {
    requireDesktop()
    return unwrapResult(await commands.cleanupOldRecoveryFiles())
  } catch {
    throw new Error('Recovery data is unavailable.')
  }
}
/** Diagnostic metadata only. No input, state snapshot, error text, stack, path or URL. */
export async function saveCrashState(_error?: unknown): Promise<boolean> {
  void _error
  try {
    await saveEmergencyData('crash', {
      event: 'renderFailed',
      at: Date.now(),
      version: __APP_VERSION__,
    })
    await logger.info('recoverySaved')
    return true
  } catch {
    await logger.error('recoveryFailed')
    return false
  }
}
