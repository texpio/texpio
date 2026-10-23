import { render, screen, act, waitFor } from '@testing-library/react'
import { StrictMode } from 'react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { UpdateDialog } from './UpdateDialog'
import { useUIStore } from '@/store/ui-store'
import { checkForUpdates, cancelUpdate, installUpdate } from '@/lib/updater'
vi.mock('@/lib/updater', () => ({
  checkForUpdates: vi.fn(),
  cancelUpdate: vi.fn().mockResolvedValue(undefined),
  installUpdate: vi.fn(),
  safeRelaunch: vi.fn(),
}))
beforeEach(() => {
  useUIStore.setState({ updateDialogOpen: true })
  vi.mocked(checkForUpdates).mockResolvedValue({
    status: 'available',
    version: '0.2.0',
    currentVersion: '0.1.0',
  })
})
it('offers explicit installation and cancellation never installs', async () => {
  render(<UpdateDialog />)
  expect(await screen.findByRole('button', { name: 'Download and install' })).toBeInTheDocument()
  await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
  expect(cancelUpdate).toHaveBeenCalled()
  expect(installUpdate).not.toHaveBeenCalled()
})
it('closes a late available result after unmount', async () => {
  let finish: (value: Awaited<ReturnType<typeof checkForUpdates>>) => void = () => {}
  vi.mocked(checkForUpdates).mockReturnValue(
    new Promise((resolve) => {
      finish = resolve
    }),
  )
  const view = render(<UpdateDialog />)
  await waitFor(() => expect(checkForUpdates).toHaveBeenCalledOnce())
  view.unmount()
  await act(async () => finish({ status: 'available', version: '0.2.0', currentVersion: '0.1.0' }))
  expect(cancelUpdate).toHaveBeenCalledTimes(2)
})
it('starts one check through StrictMode effect replay', async () => {
  render(
    <StrictMode>
      <UpdateDialog />
    </StrictMode>,
  )
  await screen.findByRole('button', { name: 'Download and install' })
  expect(checkForUpdates).toHaveBeenCalledOnce()
})
