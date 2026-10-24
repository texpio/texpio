import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, expect, it, vi } from 'vitest'
import { EngineControls } from './EngineControls'

const native = vi.hoisted(() => ({ status: vi.fn(), stop: vi.fn(), start: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@/lib/bindings', () => ({
  commands: { probeStatus: native.status, probeStop: native.stop, libraryStart: native.start },
}))
const stopped = {
  phase: 'stopped',
  profile: null,
  strategy: 'unicode',
  counters: { submitted: 0, observed: 0, refused: 0, uncertain: 0 },
  refusalReason: null,
  lastActivityAgeMs: null,
}
beforeEach(() => {
  native.status.mockResolvedValue({ status: 'ok', data: stopped })
  native.stop.mockImplementation(async () => {
    native.status.mockResolvedValue({ status: 'ok', data: stopped })
    return { status: 'ok', data: stopped }
  })
})
function show(eligibleCount = 1) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <EngineControls dirty={false} eligibleCount={eligibleCount} />
    </QueryClientProvider>,
  )
}
it('allows Stop to retry fault cleanup and only offers Start after the engine is stopped', async () => {
  native.status.mockResolvedValue({
    status: 'ok',
    data: { ...stopped, phase: 'fault', refusalReason: 'busy' },
  })
  show()
  await screen.findByText('Fault')
  const stop = screen.getByRole('button', { name: 'Stop expansion' })
  expect(stop).toBeEnabled()
  expect(screen.getByRole('button', { name: 'Start expansion' })).toBeDisabled()
  await userEvent.click(stop)
  await screen.findByText('Stopped')
  await waitFor(() => expect(screen.getByRole('button', { name: 'Start expansion' })).toBeEnabled())
  expect(native.stop).toHaveBeenCalledOnce()
  expect(native.start).not.toHaveBeenCalled()
})
it('does not offer Start when the saved library has no enabled non-trash snippets', async () => {
  show(0)
  await screen.findByText('Stopped')
  const start = screen.getByRole('button', { name: 'Start expansion' })
  expect(start).toBeDisabled()
  await userEvent.click(start)
  expect(native.start).not.toHaveBeenCalled()
})
