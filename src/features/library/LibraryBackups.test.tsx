import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, expect, it, vi } from 'vitest'
import { LibraryBackups } from './LibraryBackups'
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
const native = vi.hoisted(() => ({
  open: vi.fn(),
  save: vi.fn(),
  preview: vi.fn(),
  apply: vi.fn(),
  discard: vi.fn(),
  recovery: vi.fn(),
  recover: vi.fn(),
  approve: vi.fn(),
}))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: native.open, save: native.save }))
vi.mock('@/lib/exit-guard', () => ({ approveExit: native.approve }))
vi.mock('@/lib/bindings', () => ({
  commands: {
    backupPreview: native.preview,
    backupImport: native.apply,
    backupDiscard: native.discard,
    libraryRecoveryStatus: native.recovery,
    libraryRecover: native.recover,
  },
}))
beforeEach(() => {
  vi.clearAllMocks()
  native.open.mockResolvedValue(null)
  native.approve.mockResolvedValue(true)
  native.discard.mockResolvedValue({ status: 'ok', data: null })
  native.recovery.mockResolvedValue({ status: 'ok', data: { canRecover: false } })
  native.preview.mockResolvedValue({
    status: 'ok',
    data: {
      token: 'owned-token',
      revision: 4,
      folders: 1,
      snippets: 2,
      skipped: 0,
      files: 1,
      replace: true,
    },
  })
  native.apply.mockResolvedValue({
    status: 'ok',
    data: { schemaVersion: 1, revision: 5, folders: [], snippets: [] },
  })
})
function show() {
  render(
    <QueryClientProvider
      client={
        new QueryClient({
          defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
        })
      }
    >
      <LibraryBackups />
    </QueryClientProvider>,
  )
}
it('cancels file selection and never enables recovery for a healthy library', async () => {
  show()
  await userEvent.click(screen.getByRole('button', { name: 'Backups and recovery' }))
  await screen.findByText('JSON, YAML or ZIP')
  await userEvent.click(screen.getByRole('button', { name: 'Choose backup' }))
  expect(native.preview).not.toHaveBeenCalled()
  expect(native.apply).not.toHaveBeenCalled()
  expect(screen.getByRole('button', { name: 'Recover corrupt library' })).toBeDisabled()
})
it('previews explicit replacement and respects the draft guard before applying its token', async () => {
  native.open.mockResolvedValue('C:\\fixture\\backup.zip')
  show()
  await userEvent.click(screen.getByRole('button', { name: 'Backups and recovery' }))
  await userEvent.selectOptions(screen.getByLabelText('Import behavior'), 'replace')
  await userEvent.click(screen.getByRole('button', { name: 'Choose backup' }))
  await waitFor(() =>
    expect(native.preview).toHaveBeenCalledWith('C:\\fixture\\backup.zip', 'archive', 'replace'),
  )
  expect(native.apply).not.toHaveBeenCalled()
  native.approve.mockResolvedValue(false)
  await userEvent.click(await screen.findByRole('button', { name: 'Replace library' }))
  expect(native.apply).not.toHaveBeenCalled()
  native.approve.mockResolvedValue(true)
  await userEvent.click(screen.getByRole('button', { name: 'Replace library' }))
  await waitFor(() => expect(native.apply).toHaveBeenCalledWith('owned-token'))
})
