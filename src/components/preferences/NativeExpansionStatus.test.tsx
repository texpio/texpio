import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import { NativeExpansionStatus } from './NativeExpansionStatus'

const native = vi.hoisted(() => ({ read: vi.fn(), request: vi.fn() }))
vi.mock('@/hooks/use-platform', () => ({ usePlatform: () => 'macos' }))
vi.mock('@/services/product-platform', () => ({
  getProductCapabilities: native.read,
  requestProductPermissions: native.request,
}))
const capabilities = {
  backend: 'macos',
  readiness: 'permissionRequired',
  clipboardShortcut: false,
  autostart: true,
  richText: false,
  attachments: false,
}
beforeEach(() => {
  vi.clearAllMocks()
  native.read.mockResolvedValue(capabilities)
  native.request.mockResolvedValue({ ...capabilities, readiness: 'fieldEvidenceRequired' })
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

it('requests system permission only after the user presses the permission action', async () => {
  show()
  const button = await screen.findByRole('button', { name: 'Request macOS permissions' })
  expect(native.request).not.toHaveBeenCalled()
  await userEvent.click(button)
  await waitFor(() => expect(native.request).toHaveBeenCalledTimes(1))
  expect(
    await screen.findByText(/current field or input method cannot be verified/),
  ).toBeInTheDocument()
  expect(
    screen.queryByRole('button', { name: 'Request macOS permissions' }),
  ).not.toBeInTheDocument()
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
