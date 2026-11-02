import { describe, expect, it } from 'vitest'
import { parseProductCapabilities } from './product-platform'

describe('native platform capability boundary', () => {
  const valid = {
    backend: 'linuxWayland',
    readiness: 'helperRequired',
    permissions: { accessibility: true, listen: true, post: true },
    clipboardShortcut: false,
    autostart: true,
    richText: false,
    attachments: false,
  }
  it('retains an actionable helper requirement without pretending expansion is running', () => {
    expect(parseProductCapabilities(valid)).toEqual(valid)
  })
  it('rejects unknown backends, readiness values and malformed flags', () => {
    for (const patch of [
      { backend: 'browser' },
      { readiness: 'probablyReady' },
      { autostart: 'yes' },
      { clipboardShortcut: null },
      { permissions: null },
      { permissions: { accessibility: true, listen: 'yes', post: true } },
      { permissions: { accessibility: true, listen: true } },
    ]) {
      expect(() => parseProductCapabilities({ ...valid, ...patch })).toThrow()
    }
    expect(() => parseProductCapabilities(null)).toThrow()
  })
})
