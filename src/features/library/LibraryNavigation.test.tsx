import { render, screen, within } from '@testing-library/react'
import { expect, it, vi } from 'vitest'
import type { AppPlatform } from '@/hooks/use-platform'
import { LibraryNavigation } from './LibraryNavigation'

const context = vi.hoisted(() => ({ platform: 'windows' as AppPlatform }))
vi.mock('@/hooks/use-command-context', () => ({
  useCommandContext: () => ({
    platform: context.platform,
    openPreferences: vi.fn(),
    showToast: vi.fn(),
    windowAction: vi.fn(),
  }),
}))
vi.mock('@/services/library', () => ({ useLibrary: () => ({ data: undefined }) }))
vi.mock('@/services/library-metadata', () => ({ useLibraryMetadata: () => ({ data: undefined }) }))

it.each([
  ['windows', 'Control+K', 'Control+,', ['Ctrl', 'K'], ['Ctrl', ',']],
  ['macos', 'Meta+K', 'Meta+,', ['⌘', 'K'], ['⌘', ',']],
] as const)(
  'shows the registered shortcuts on %s without changing button names',
  (platform, searchShortcut, settingsShortcut, searchKeys, settingsKeys) => {
    context.platform = platform
    render(<LibraryNavigation />)
    const search = screen.getByRole('button', { name: 'Search' })
    const settings = screen.getByRole('button', { name: 'Settings' })
    expect(search).toHaveAttribute('aria-keyshortcuts', searchShortcut)
    expect(settings).toHaveAttribute('aria-keyshortcuts', settingsShortcut)
    const keys = (button: HTMLElement) =>
      [...button.querySelectorAll('[data-slot="kbd"]')].map((key) => key.textContent)
    expect(keys(search)).toEqual(searchKeys)
    expect(keys(settings)).toEqual(settingsKeys)
    expect(within(search).getByText('Search')).toBeInTheDocument()
    expect(within(settings).getByText('Settings')).toBeInTheDocument()
  },
)
