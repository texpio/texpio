import { act, render, waitFor } from '@testing-library/react'
import { expect, it, vi } from 'vitest'
import { toast } from 'sonner'
import { Toaster } from './sonner'

vi.mock('@/hooks/use-theme', () => ({ useTheme: () => ({ theme: 'light' }) }))

it('positions notifications at the bottom center', async () => {
  const { container } = render(<Toaster />)
  act(() => {
    toast.success('Saved')
  })
  await waitFor(() => expect(container.querySelector('[data-sonner-toaster]')).toBeInTheDocument())
  expect(container.querySelector('[data-sonner-toaster]')).toHaveAttribute(
    'data-y-position',
    'bottom',
  )
  expect(container.querySelector('[data-sonner-toaster]')).toHaveAttribute(
    'data-x-position',
    'center',
  )
})
