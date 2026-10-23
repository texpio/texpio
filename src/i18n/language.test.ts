import { describe, expect, it } from 'vitest'
import i18n, { resolveLanguage, applyLanguage } from './config'

describe('shell localization', () => {
  it('uses saved language, then system language with English fallback', () => {
    expect(resolveLanguage('fr', ['ar-IN'])).toBe('fr')
    expect(resolveLanguage(null, ['ar-IN'])).toBe('ar')
    expect(resolveLanguage('broken', ['zz-ZZ'])).toBe('en')
    expect(resolveLanguage(null, ['de', 'fr-FR'])).toBe('fr')
  })
  it('updates Arabic RTL and restores LTR with English fallback text', async () => {
    await applyLanguage('ar')
    expect(document.documentElement.dir).toBe('rtl')
    expect(document.documentElement.lang).toBe('ar')
    expect(i18n.t('shell.home')).not.toBe('shell.home')
    await applyLanguage('fr')
    expect(document.documentElement.dir).toBe('ltr')
    expect(i18n.t('shell.developer')).not.toBe('shell.developer')
  })
})
