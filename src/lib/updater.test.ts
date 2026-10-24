import { beforeEach, expect, it, vi } from 'vitest'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { waitFor } from '@testing-library/react'
import { registerExitGuard } from './exit-guard'
import {
  checkForUpdates,
  getUpdateConfiguration,
  cancelUpdate,
  installUpdate,
  safeRelaunch,
} from './updater'
const native = vi.hoisted(() => ({ configuration: vi.fn(), stop: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@/lib/tauri-bindings', () => ({
  commands: { getUpdateConfiguration: native.configuration, prepareForRelaunch: native.stop },
  requireDesktop: () => {},
  unwrapResult: (result: { status: string; data: unknown; error?: unknown }) => {
    if (result.status === 'error') throw result.error
    return result.data
  },
}))
vi.mock('@tauri-apps/plugin-updater', () => ({ check: vi.fn() }))
vi.mock('@tauri-apps/plugin-process', () => ({ relaunch: vi.fn() }))
const update = {
  version: '0.2.0',
  currentVersion: '0.1.0',
  download: vi.fn(),
  install: vi.fn(),
  close: vi.fn(),
}
beforeEach(async () => {
  await cancelUpdate()
  native.configuration.mockResolvedValue({ configured: false })
  native.stop.mockResolvedValue({ status: 'ok', data: null })
  vi.mocked(check).mockResolvedValue(update as never)
  update.download.mockResolvedValue(undefined)
  update.install.mockResolvedValue(undefined)
  update.close.mockResolvedValue(undefined)
})
it('unconfigured updates never call plugin check', async () => {
  await expect(checkForUpdates()).resolves.toEqual({ status: 'unconfigured' })
  expect(getUpdateConfiguration()).toEqual({ configured: false })
  expect(check).not.toHaveBeenCalled()
})
it('returns version metadata only and cancellation performs no mutation', async () => {
  native.configuration.mockResolvedValue({ configured: true })
  await expect(checkForUpdates()).resolves.toEqual({
    status: 'available',
    version: '0.2.0',
    currentVersion: '0.1.0',
  })
  await cancelUpdate()
  expect(update.close).toHaveBeenCalledOnce()
  expect(update.download).not.toHaveBeenCalled()
  expect(update.install).not.toHaveBeenCalled()
  expect(relaunch).not.toHaveBeenCalled()
})
it('downloads then waits for confirmed Stop before installation', async () => {
  native.configuration.mockResolvedValue({ configured: true })
  await checkForUpdates()
  const order: string[] = []
  update.download.mockImplementation(async () => {
    order.push('download')
  })
  native.stop.mockImplementation(async () => {
    order.push('stop')
    return { status: 'ok', data: null }
  })
  update.install.mockImplementation(async () => {
    order.push('install')
  })
  await expect(installUpdate()).resolves.toEqual({ success: true })
  expect(order).toEqual(['download', 'stop', 'install'])
  expect(relaunch).not.toHaveBeenCalled()
})
it.each(['outcomeUncertain', 'providerTimeout'])(
  'blocks install and restart after unconfirmed shutdown: %s',
  async (error) => {
    native.configuration.mockResolvedValue({ configured: true })
    await checkForUpdates()
    native.stop.mockResolvedValue({ status: 'error', error })
    await expect(installUpdate()).resolves.toEqual({ success: false, code: 'unavailable' })
    await expect(safeRelaunch()).resolves.toEqual({ success: false, code: 'unavailable' })
    expect(update.install).not.toHaveBeenCalled()
    expect(relaunch).not.toHaveBeenCalled()
  },
)
it('awaits Stop before explicit relaunch', async () => {
  let release: (value: unknown) => void = () => {}
  native.stop.mockReturnValue(
    new Promise((resolve) => {
      release = resolve
    }),
  )
  const pending = safeRelaunch()
  expect(relaunch).not.toHaveBeenCalled()
  await waitFor(() => expect(native.stop).toHaveBeenCalled())
  release({ status: 'ok', data: null })
  await pending
  expect(relaunch).toHaveBeenCalledOnce()
})
it('cancelling the draft guard prevents both restart and update installation', async () => {
  const unregister = registerExitGuard(async () => false)
  try {
    native.configuration.mockResolvedValue({ configured: true })
    await checkForUpdates()
    await expect(safeRelaunch()).resolves.toEqual({ success: false, code: 'unavailable' })
    await expect(installUpdate()).resolves.toEqual({ success: false, code: 'unavailable' })
    expect(native.stop).not.toHaveBeenCalled()
    expect(relaunch).not.toHaveBeenCalled()
    expect(update.install).not.toHaveBeenCalled()
  } finally {
    unregister()
  }
})
it('keeps raw updater failures out of public results', async () => {
  native.configuration.mockResolvedValue({ configured: true })
  vi.mocked(check).mockRejectedValue({
    url: 'https://user:password@example.test',
    path: 'C:/private/secret',
    text: 'captured text',
  })
  await expect(checkForUpdates()).resolves.toEqual({ status: 'error', code: 'unavailable' })
})
it('closes update resources when received metadata is malformed', async () => {
  native.configuration.mockResolvedValue({ configured: true })
  vi.mocked(check).mockResolvedValue({ ...update, version: null } as never)
  await expect(checkForUpdates()).resolves.toEqual({ status: 'error', code: 'unavailable' })
  expect(update.close).toHaveBeenCalledOnce()
})
