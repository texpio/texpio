import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { useUIStore } from '@/store/ui-store'
import { NativePermissionPrompt } from './NativePermissionPrompt'

const native = vi.hoisted(() => ({ read: vi.fn(), request: vi.fn(), open: vi.fn() }))
const currentPlatform = vi.hoisted(() => ({ value: 'macos' }))
vi.mock('@/hooks/use-platform', () => ({ usePlatform: () => currentPlatform.value }))
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
  currentPlatform.value = 'macos'
  useUIStore.setState({ permissionPromptDismissed: false })
  native.read.mockResolvedValue(capabilities)
  native.request.mockResolvedValue(capabilities)
  native.open.mockResolvedValue(undefined)
})
function show() {
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <NativePermissionPrompt />
    </QueryClientProvider>,
  )
}

it('shows the single required grant inside the app and lets the user defer it', async () => {
  show()
  expect(await screen.findByRole('dialog')).toBeInTheDocument()
  expect(screen.getByText('Accessibility')).toBeInTheDocument()
  expect(screen.queryByText('Input Monitoring')).not.toBeInTheDocument()
  expect(screen.getAllByRole('button', { name: 'Open System Settings' })).toHaveLength(1)
  await userEvent.click(screen.getByRole('button', { name: 'Not now' }))
  await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument())
  expect(useUIStore.getState().permissionPromptDismissed).toBe(true)
})
it('disappears once macOS reports the grant', async () => {
  native.request.mockResolvedValue({
    ...capabilities,
    readiness: 'ready',
    permissions: { accessibility: true, listen: true, post: true },
  })
  show()
  await userEvent.click(await screen.findByRole('button', { name: 'Request permissions' }))
  await waitFor(() => expect(native.request).toHaveBeenCalledTimes(1))
  await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument())
})
it('stays out of the way when grants are already allowed', async () => {
  native.read.mockResolvedValue({
    ...capabilities,
    readiness: 'ready',
    permissions: { accessibility: true, listen: true, post: true },
  })
  show()
  await waitFor(() => expect(native.read).toHaveBeenCalled())
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
})
it('never prompts outside macOS', async () => {
  currentPlatform.value = 'windows'
  show()
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  expect(native.read).not.toHaveBeenCalled()
})
