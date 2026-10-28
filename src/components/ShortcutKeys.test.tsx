import { render } from '@testing-library/react'
import { expect, it } from 'vitest'
import { ShortcutKeys } from './ShortcutKeys'

it.each([
  ['windows', 'CmdOrCtrl+Shift+Space', ['Ctrl', 'Shift', 'Space']],
  ['macos', 'CommandOrControl+Shift+Period', ['⌘', '⇧', '.']],
] as const)('renders whole keys for %s shortcuts', (platform, shortcut, expected) => {
  const { container } = render(<ShortcutKeys platform={platform} shortcut={shortcut} />)
  expect(
    [...container.querySelectorAll('[data-slot="kbd"]')].map((key) => key.textContent),
  ).toEqual(expected)
})
