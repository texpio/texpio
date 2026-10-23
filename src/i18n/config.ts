import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import en from '../../locales/en.json'
import fr from '../../locales/fr.json'
import ar from '../../locales/ar.json'
import type { Language } from '@/lib/bindings'
export const availableLanguages = ['en', 'fr', 'ar'] as const
export function resolveLanguage(
  saved: unknown,
  system: readonly string[] = navigator.languages,
): Language {
  if (availableLanguages.includes(saved as Language)) return saved as Language
  for (const language of system) {
    const base = language.toLowerCase().split('-')[0]
    if (availableLanguages.includes(base as Language)) return base as Language
  }
  return 'en'
}
export const isRTL = (language: string) => language.split('-')[0] === 'ar'
function documentLanguage(language: string) {
  document.documentElement.dir = isRTL(language) ? 'rtl' : 'ltr'
  document.documentElement.lang = language
}
i18n.on('languageChanged', documentLanguage)
void i18n.use(initReactI18next).init({
  resources: { en: { translation: en }, fr: { translation: fr }, ar: { translation: ar } },
  lng: 'en',
  fallbackLng: 'en',
  showSupportNotice: false,
  keySeparator: false,
  initAsync: false,
  interpolation: { escapeValue: false },
})
export async function applyLanguage(language: Language) {
  await i18n.changeLanguage(language)
  documentLanguage(language)
}
export { i18n }
export default i18n
