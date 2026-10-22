import { invoke, isTauri } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getAppInfo } from './desktop'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))

beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(true)
})

describe('native metadata boundary', () => {
  it('reads version and platform from the registered native command', async () => {
    vi.mocked(invoke).mockResolvedValue({ version: '0.1.0', platform: 'windows' })
    await expect(getAppInfo()).resolves.toEqual({ version: '0.1.0', platform: 'windows' })
    expect(invoke).toHaveBeenCalledWith('get_app_info')
    expect(invoke).toHaveBeenCalledTimes(1)
  })

  it.each([
    null,
    [],
    {},
    { version: 3, platform: 'windows' },
    { version: '', platform: 'windows' },
    { version: '0.1.0', platform: false },
    { version: '0.1.0', platform: '  ' },
  ])('refuses malformed metadata %j', async (value) => {
    vi.mocked(invoke).mockResolvedValue(value)
    await expect(getAppInfo()).rejects.toThrow('Native desktop information is unavailable.')
  })

  it('keeps internal native errors out of the UI contract', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Internal error at C:/private/example'))
    await expect(getAppInfo()).rejects.toThrow('Native desktop information is unavailable.')
  })

  it('reports a browser preview without attempting native IPC', async () => {
    vi.mocked(isTauri).mockReturnValue(false)
    await expect(getAppInfo()).rejects.toThrow('Native desktop information is unavailable.')
    expect(invoke).not.toHaveBeenCalled()
  })
})
