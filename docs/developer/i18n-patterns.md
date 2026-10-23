# Localization and RTL

Root `locales/en.json`, `fr.json` and `ar.json` contain visible strings. i18next/react-i18next initialization lives in `src/i18n/index.ts`; Rust preferences own the explicit language choice. Use interpolation/plurals instead of concatenating fragments.

Providers apply language/direction in each window and refresh after native preference events. Arabic exercises RTL; use logical spacing/borders. Exact probe fixture bodies remain test data, unchanged.

Native menus update translated labels in place; command keywords are translated too. Outside React use the initialized instance, not hooks. Native failures map to fixed translated messages.

Adding a language requires complete keys, selector/validator registration and direction tests. System language resolves the first supported browser/OS language, falling back to English; an explicit stored choice wins. Test cross-window changes, error fallback and late listener cleanup.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
