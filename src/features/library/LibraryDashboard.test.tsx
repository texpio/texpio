import { render, screen, within } from '@testing-library/react'
import { expect, it } from 'vitest'
import { LibraryDashboard } from './LibraryDashboard'
it('orders recent and most used independently and excludes trash', () => {
  const snippets = ['First', 'Second', 'Trashed'].map((title, index) => ({
    id: String(index),
    title,
    trigger: ':' + index,
    body: '',
    folderId: null,
    enabled: true,
    deleted: index === 2,
  }))
  const metadata = [
    { snippetId: '0', favorite: false, usageCount: 10, createdAt: 1, updatedAt: 1, lastUsedAt: 2 },
    { snippetId: '1', favorite: true, usageCount: 2, createdAt: 1, updatedAt: 1, lastUsedAt: 4 },
    {
      snippetId: '2',
      favorite: false,
      usageCount: 999,
      createdAt: 1,
      updatedAt: 1,
      lastUsedAt: 99,
    },
  ]
  render(<LibraryDashboard snippets={snippets} metadata={metadata} />)
  expect(screen.queryByText(/Trashed/)).not.toBeInTheDocument()
  const recent = screen.getByRole('heading', { name: 'Recently used' }).parentElement!
  const frequent = screen.getByRole('heading', { name: 'Most used' }).parentElement!
  expect(within(recent).getAllByRole('listitem')[0]).toHaveTextContent('Second')
  expect(within(frequent).getAllByRole('listitem')[0]).toHaveTextContent('First')
})
