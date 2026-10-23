import { invoke, isTauri } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  getProbeStatus,
  getProbeFixtures,
  startProbe,
  stopProbe,
  pauseProbe,
  excludeProbe,
  ProbeCommandError,
} from './probe'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))
const stopped = {
  phase: 'stopped',
  profile: null,
  strategy: 'unicode',
  counters: { submitted: 0, observed: 0, refused: 0, uncertain: 0 },
  refusalReason: null,
  lastActivityAgeMs: null,
}

beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(true)
})

describe('probe IPC boundary', () => {
  it('validates status and strips additional native fields', async () => {
    vi.mocked(invoke).mockResolvedValue({
      ...stopped,
      rawText: 'private',
      windowHandle: 42,
      counters: { ...stopped.counters, extra: 'private' },
    })
    await expect(getProbeStatus()).resolves.toEqual(stopped)
    expect(invoke).toHaveBeenCalledWith('probe_status')
  })
  it.each([
    null,
    [],
    {},
    { ...stopped, phase: 'armed' },
    { ...stopped, profile: 'arbitrary.exe' },
    { ...stopped, strategy: 'script' },
    { ...stopped, refusalReason: 'internal path' },
    { ...stopped, lastActivityAgeMs: -1 },
  ])('refuses malformed status %j', async (value) => {
    vi.mocked(invoke).mockResolvedValue(value)
    await expect(getProbeStatus()).rejects.toThrow('Windows expansion probe is unavailable.')
  })
  it.each([-1, 0.5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1, '1', null])(
    'refuses invalid counters %j',
    async (value) => {
      vi.mocked(invoke).mockResolvedValue({
        ...stopped,
        counters: { ...stopped.counters, submitted: value },
      })
      await expect(getProbeStatus()).rejects.toThrow('Windows expansion probe is unavailable.')
    },
  )
  it('does not make IPC calls in browser mode', async () => {
    vi.mocked(isTauri).mockReturnValue(false)
    await expect(getProbeStatus()).rejects.toThrow('Windows expansion probe is unavailable.')
    expect(invoke).not.toHaveBeenCalled()
  })
  it('maps only closed error codes and hides OS details', async () => {
    vi.mocked(invoke).mockRejectedValue({ code: 'passwordField', internal: 'C:/private/example' })
    await expect(startProbe({ profile: 'notepad', strategy: 'unicode' })).rejects.toBeInstanceOf(
      ProbeCommandError,
    )
    vi.mocked(invoke).mockRejectedValue(new Error('C:/private/example'))
    await expect(stopProbe()).rejects.toThrow('Windows expansion probe is unavailable.')
  })
  it('sends narrow command arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(stopped)
    await startProbe({ profile: 'notepad', strategy: 'unicode' })
    await pauseProbe(true)
    await excludeProbe('chromeFixture')
    await stopProbe()
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ['probe_start', { options: { profile: 'notepad', strategy: 'unicode' } }],
      ['probe_pause', { paused: true }],
      ['probe_exclude', { profile: 'chromeFixture' }],
      ['probe_stop'],
    ])
  })
  it('strips arbitrary fields from start options', async () => {
    vi.mocked(invoke).mockResolvedValue(stopped)
    await startProbe({ profile: 'notepad', strategy: 'unicode', windowHandle: 42 } as Parameters<
      typeof startProbe
    >[0])
    expect(invoke).toHaveBeenCalledWith('probe_start', {
      options: { profile: 'notepad', strategy: 'unicode' },
    })
  })
  it('validates fixed fixtures and strips extra fields', async () => {
    vi.mocked(invoke).mockResolvedValue([
      { id: 'hello', title: 'Greeting', trigger: ';hello', body: 'Hello!', rawText: 'private' },
    ])
    await expect(getProbeFixtures()).resolves.toEqual([
      { id: 'hello', title: 'Greeting', trigger: ';hello', body: 'Hello!' },
    ])
  })
  it.each([
    null,
    [{ id: 'unknown', title: 'X', trigger: ';x', body: 'Y' }],
    [{ id: 'hello', title: 'X', trigger: 'X', body: 'Y' }],
    [{ id: 'hello', title: 'X', trigger: ';x', body: 3 }],
  ])('refuses malformed fixtures %j', async (value) => {
    vi.mocked(invoke).mockResolvedValue(value)
    await expect(getProbeFixtures()).rejects.toThrow('Windows expansion probe is unavailable.')
  })
})
