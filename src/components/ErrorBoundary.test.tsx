import { render, screen, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { ErrorBoundary } from './ErrorBoundary'
import { saveCrashState } from '@/lib/recovery'
import { safeRelaunch } from '@/lib/updater'
vi.mock('@/lib/recovery', () => ({ saveCrashState: vi.fn() }))
vi.mock('@/lib/logger', () => ({ logger: { error: vi.fn() } }))
vi.mock('@/lib/updater', () => ({ safeRelaunch: vi.fn() }))
function Broken(): never {
  throw new Error('C:/private/secret captured text')
}
beforeEach(() => {
  vi.mocked(saveCrashState).mockResolvedValue(false)
  vi.mocked(safeRelaunch).mockResolvedValue({ success: false, code: 'unavailable' })
})
it('shows fixed feedback and never claims failed diagnostics were saved', async () => {
  render(
    <ErrorBoundary>
      <Broken />
    </ErrorBoundary>,
    { onCaughtError: () => {} },
  )
  expect(await screen.findByText('Diagnostics could not be saved.')).toBeInTheDocument()
  expect(screen.queryByText(/C:\/private/)).not.toBeInTheDocument()
  expect(screen.queryByText('Diagnostic metadata saved.')).not.toBeInTheDocument()
  await userEvent.click(screen.getByRole('button', { name: 'Restart Texpio' }))
  expect(safeRelaunch).toHaveBeenCalledOnce()
  expect(await screen.findByText('Restart unavailable. Try again.')).toBeInTheDocument()
})
it('reports saved diagnostics only after the write succeeds', async () => {
  let finish: (value: boolean) => void = () => {}
  vi.mocked(saveCrashState).mockReturnValue(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  render(
    <ErrorBoundary>
      <Broken />
    </ErrorBoundary>,
    { onCaughtError: () => {} },
  )
  expect(screen.queryByText('Diagnostic metadata saved.')).not.toBeInTheDocument()
  await act(async () => finish(true))
  expect(screen.getByText('Diagnostic metadata saved.')).toBeInTheDocument()
})
