import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { NativeExpansionStatus } from './NativeExpansionStatus'

const native = vi.hoisted(() => ({ read: vi.fn(), request: vi.fn(), open: vi.fn() }))
vi.mock('@/hooks/use-platform', () => ({ usePlatform: () => 'macos' }))
vi.mock('@/services/product-platform', () => ({
  getProductCapabilities: native.read,
  requestProductPermissions: native.request,
  openPermissionSettings: native.open,
}))
const capabilities = {
  backend: 'macos',
  readiness: 'permissionRequired',
  permissions: { accessibility: false, listen: false, post: false },
  clipboardShortcut: false,
  autostart: true,
  richText: false,
  attachments: false,
}
beforeEach(() => {
  vi.clearAllMocks()
  native.read.mockResolvedValue(capabilities)
  native.request.mockResolvedValue({ ...capabilities, readiness: 'fieldEvidenceRequired' })
  native.open.mockResolvedValue(undefined)
})
function show() {
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <NativeExpansionStatus />
    </QueryClientProvider>,
  )
}

it('asks for Accessibility alone and requests it only on the user action', async () => {
  show()
  expect(await screen.findByText('Accessibility')).toBeInTheDocument()
  expect(screen.queryByText('Input Monitoring')).not.toBeInTheDocument()
  expect(screen.getAllByText('Needs access')).toHaveLength(1)
  const button = screen.getByRole('button', { name: 'Request permissions' })
  expect(native.request).not.toHaveBeenCalled()
  await userEvent.click(button)
  await waitFor(() => expect(native.request).toHaveBeenCalledTimes(1))
  expect(
    await screen.findByText(/current field or input method cannot be verified/),
  ).toBeInTheDocument()
  expect(screen.queryByRole('button', { name: 'Request permissions' })).not.toBeInTheDocument()
})
it('opens the privacy pane that owns the missing grant', async () => {
  show()
  const accessibility = await screen.findByRole('button', { name: 'Open System Settings' })
  await userEvent.click(accessibility)
  await waitFor(() => expect(native.open.mock.calls[0]?.[0]).toBe('accessibility'))
})
it('marks the grant independently once macOS reports it', async () => {
  native.read.mockResolvedValue({
    ...capabilities,
    permissions: { accessibility: true, listen: true, post: true },
  })
  show()
  expect(await screen.findByText('Allowed')).toBeInTheDocument()
})
it('falls back to Input Monitoring only when listening is withheld', async () => {
  native.read.mockResolvedValue({
    ...capabilities,
    permissions: { accessibility: true, listen: false, post: true },
  })
  show()
  expect(await screen.findByText('Allowed')).toBeInTheDocument()
  expect(screen.getByText('Input Monitoring')).toBeInTheDocument()
  expect(screen.getByText('Needs access')).toBeInTheDocument()
})
it('treats Accessibility as the only required grant once listening follows it', async () => {
  native.read.mockResolvedValue({
    ...capabilities,
    permissions: { accessibility: false, listen: true, post: true },
  })
  show()
  expect(await screen.findByText('Accessibility')).toBeInTheDocument()
  expect(screen.queryByText('Input Monitoring')).not.toBeInTheDocument()
  expect(screen.getAllByText('Needs access')).toHaveLength(1)
})
it('explains missing Wayland setup without requesting permissions or starting expansion', async () => {
  native.read.mockResolvedValue({
    ...capabilities,
    backend: 'linuxWayland',
    readiness: 'helperRequired',
  })
  show()
  expect(await screen.findByText(/Install the separate Wayland input helper/)).toBeInTheDocument()
  expect(native.request).not.toHaveBeenCalled()
  expect(
    screen.queryByRole('button', { name: 'Request macOS permissions' }),
  ).not.toBeInTheDocument()
})
