import { beforeEach, expect, it, vi } from 'vitest'
import { logger } from './logger'
import { saveCrashState } from './recovery'
import { notify } from './notifications'
import { toast } from 'sonner'
const native = vi.hoisted(() => ({ record: vi.fn(), save: vi.fn(), notify: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@/lib/tauri-bindings', () => ({
  commands: {
    recordDiagnostic: native.record,
    saveEmergencyData: native.save,
    sendNativeNotification: native.notify,
  },
  requireDesktop: () => {},
  unwrapResult: (result: { status: string; data?: unknown; error?: unknown }) => {
    if (result.status === 'error') throw result.error
    return result.data
  },
}))
vi.mock('sonner', () => ({
  toast: { error: vi.fn(), info: vi.fn(), success: vi.fn(), warning: vi.fn() },
}))
const sensitive = {
  quickEntry: 'secret quick entry',
  capturedText: 'captured input',
  clipboard: 'clipboard payload',
  path: 'C:/private/secret',
  HWND: 12345,
  error: new Error('C:/private/failure'),
}
beforeEach(() => {
  native.record.mockResolvedValue(undefined)
  native.save.mockResolvedValue({ status: 'ok', data: null })
  native.notify.mockResolvedValue({ status: 'error', error: sensitive })
})
it('drops arbitrary metadata and unknown log messages', async () => {
  await logger.error('renderFailed', sensitive)
  await logger.error('C:/private/secret' as never, sensitive)
  const calls = JSON.stringify(native.record.mock.calls)
  for (const text of [
    'secret quick entry',
    'captured input',
    'clipboard payload',
    'C:/private',
    'HWND',
    '12345',
  ])
    expect(calls).not.toContain(text)
})
it('saves only allowed crash metadata and reports actual save success', async () => {
  await expect(saveCrashState(sensitive)).resolves.toBe(true)
  const calls = JSON.stringify(native.save.mock.calls)
  for (const text of [
    'secret quick entry',
    'captured input',
    'clipboard payload',
    'C:/private',
    'HWND',
    '12345',
  ])
    expect(calls).not.toContain(text)
  native.save.mockResolvedValue({ status: 'error', error: sensitive })
  await expect(saveCrashState(sensitive)).resolves.toBe(false)
})
it('uses a toast when a native notification fails without exposing its error', async () => {
  await expect(notify('Texpio', 'Test notification', { native: true })).resolves.toEqual({
    delivered: 'toast',
  })
  expect(toast.error).toHaveBeenCalledWith(
    'Texpio',
    expect.objectContaining({ description: 'Test notification' }),
  )
  expect(JSON.stringify(vi.mocked(toast.error).mock.calls)).not.toContain('C:/private')
})
