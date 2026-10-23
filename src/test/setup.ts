import '@testing-library/jest-dom/vitest'
import { cleanup } from '@testing-library/react'
import { afterEach, beforeEach, vi } from 'vitest'
import { applyLanguage } from '@/i18n/config'

beforeEach(async () => {
  await applyLanguage('en')
  localStorage.clear()
  document.documentElement.classList.remove('dark')
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})
