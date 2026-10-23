import { renderHook } from '@testing-library/react'
import { StrictMode, type PropsWithChildren } from 'react'
import { expect, it, vi } from 'vitest'
import { useKeyboardShortcuts } from './use-keyboard-shortcuts'
import type { CommandContext } from '@/lib/commands'
it('cleans up the one keyboard listener through StrictMode and unmount', () => {
  const context: CommandContext = {
    platform: 'windows',
    openPreferences: vi.fn(),
    showToast: vi.fn(),
    windowAction: vi.fn().mockResolvedValue({ success: true }),
  }
  const wrapper = ({ children }: PropsWithChildren) => <StrictMode>{children}</StrictMode>
  const hook = renderHook(() => useKeyboardShortcuts(context), { wrapper })
  document.dispatchEvent(new KeyboardEvent('keydown', { key: ',', ctrlKey: true }))
  expect(context.openPreferences).toHaveBeenCalledOnce()
  hook.unmount()
  document.dispatchEvent(new KeyboardEvent('keydown', { key: ',', ctrlKey: true }))
  expect(context.openPreferences).toHaveBeenCalledOnce()
})
