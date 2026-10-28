import { act, render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import * as probe from '@/lib/probe'
import { ProbePanel } from './ProbePanel'

vi.mock('@/lib/probe', async (original) => ({
  ...(await original<typeof import('@/lib/probe')>()),
  getProbeStatus: vi.fn(),
  getProbeFixtures: vi.fn(),
  startProbe: vi.fn(),
  stopProbe: vi.fn(),
  pauseProbe: vi.fn(),
  excludeProbe: vi.fn(),
}))
const stopped: probe.ProbeStatus = {
  phase: 'stopped',
  profile: null,
  strategy: 'unicode',
  counters: { submitted: 0, observed: 0, refused: 0, uncertain: 0 },
  refusalReason: null,
  lastActivityAgeMs: null,
}
const running: probe.ProbeStatus = { ...stopped, phase: 'running', profile: 'notepad' }
const fixture = { id: 'hello', title: 'Greeting', trigger: ';hello', body: 'Hello from Texpio!' }

beforeEach(() => {
  vi.mocked(probe.getProbeStatus).mockResolvedValue(stopped)
  vi.mocked(probe.getProbeFixtures).mockResolvedValue([fixture])
  vi.mocked(probe.startProbe).mockResolvedValue(running)
  vi.mocked(probe.stopProbe).mockResolvedValue(stopped)
  vi.mocked(probe.pauseProbe).mockResolvedValue({ ...running, phase: 'paused' })
  vi.mocked(probe.excludeProbe).mockResolvedValue({ ...stopped, refusalReason: 'excluded' })
})
async function ready() {
  render(<ProbePanel />)
  await screen.findByText('Greeting')
}
async function choose() {
  await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Reference app' }), 'notepad')
}

describe('probe panel', () => {
  it('keeps the unqualified clipboard experiment disabled and explains why', async () => {
    await ready()
    expect(screen.getByRole('option', { name: /Clipboard/ })).toBeDisabled()
    expect(
      screen.getByText('Clipboard preservation and paste consumption are not qualified.'),
    ).toBeInTheDocument()
  })
  it('starts stopped, requires a selected profile, and defaults to Unicode', async () => {
    await ready()
    expect(screen.getByRole('status', { name: 'Expansion status' })).toHaveTextContent('Stopped')
    expect(screen.getByRole('button', { name: 'Start probe' })).toBeDisabled()
    expect(screen.getByRole('combobox', { name: 'Insertion strategy' })).toHaveValue('unicode')
    await choose()
    await userEvent.click(screen.getByRole('button', { name: 'Start probe' }))
    expect(probe.startProbe).toHaveBeenCalledWith({ profile: 'notepad', strategy: 'unicode' })
    await waitFor(() =>
      expect(screen.getByRole('status', { name: 'Expansion status' })).toHaveTextContent('Running'),
    )
  })
  it('serializes ordinary controls while keeping Stop available', async () => {
    vi.mocked(probe.startProbe).mockReturnValue(new Promise(() => {}))
    await ready()
    await choose()
    await userEvent.click(screen.getByRole('button', { name: 'Start probe' }))
    expect(screen.getByRole('button', { name: 'Start probe' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Pause' })).toBeDisabled()
    await userEvent.click(screen.getByRole('button', { name: 'Stop' }))
    expect(probe.startProbe).toHaveBeenCalledTimes(1)
    expect(probe.stopProbe).toHaveBeenCalledTimes(1)
  })
  it('ignores a late Running response after Stop', async () => {
    let resolveStart!: (value: probe.ProbeStatus) => void
    vi.mocked(probe.startProbe).mockReturnValue(
      new Promise((resolve) => {
        resolveStart = resolve
      }),
    )
    await ready()
    await choose()
    await userEvent.click(screen.getByRole('button', { name: 'Start probe' }))
    await userEvent.click(screen.getByRole('button', { name: 'Stop' }))
    await act(async () => resolveStart(running))
    expect(screen.getByRole('status', { name: 'Expansion status' })).toHaveTextContent('Stopped')
  })
  it('uses native Pause and exclusion transitions and fixed refusal labels', async () => {
    vi.mocked(probe.getProbeStatus).mockResolvedValue(running)
    await ready()
    await userEvent.click(screen.getByRole('button', { name: 'Pause' }))
    expect(probe.pauseProbe).toHaveBeenCalledWith(true)
    expect(screen.getByRole('status', { name: 'Expansion status' })).toHaveTextContent('Paused')
    await userEvent.click(screen.getByRole('button', { name: 'Exclude this app' }))
    expect(probe.excludeProbe).toHaveBeenCalledWith('notepad')
    expect(screen.getByText(probe.refusalLabels.excluded)).toBeInTheDocument()
  })
  it('does not describe queued submissions as observed expansions', async () => {
    vi.mocked(probe.getProbeStatus).mockResolvedValue({
      ...running,
      counters: { submitted: 4, observed: 1, refused: 2, uncertain: 0 },
    })
    await ready()
    expect(within(screen.getByTestId('counter-submitted')).getByText('4')).toBeInTheDocument()
    expect(within(screen.getByTestId('counter-observed')).getByText('1')).toBeInTheDocument()
  })
  it('shows a content-free failure and retries without leaking native errors', async () => {
    vi.mocked(probe.getProbeStatus)
      .mockRejectedValueOnce(new Error('C:/private/example'))
      .mockResolvedValue(stopped)
    render(<ProbePanel />)
    expect(await screen.findByRole('alert')).toHaveTextContent(probe.unavailableMessage)
    expect(screen.queryByText(/private/)).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: 'Retry probe connection' }))
    expect(await screen.findByText('Greeting')).toBeInTheDocument()
    await waitFor(() =>
      expect(screen.queryByText(probe.unavailableMessage)).not.toBeInTheDocument(),
    )
  })
  it('polls tray-equivalent status and stops polling after unmount', async () => {
    const view = render(<ProbePanel />)
    await screen.findByText('Greeting')
    vi.mocked(probe.getProbeStatus).mockResolvedValue({
      ...running,
      phase: 'fault',
      refusalReason: 'outcomeUncertain',
    })
    await waitFor(
      () =>
        expect(screen.getByRole('status', { name: 'Expansion status' })).toHaveTextContent('Fault'),
      { timeout: 1800 },
    )
    view.unmount()
    const count = vi.mocked(probe.getProbeStatus).mock.calls.length
    await new Promise((resolve) => setTimeout(resolve, 600))
    expect(probe.getProbeStatus).toHaveBeenCalledTimes(count)
  })
})
