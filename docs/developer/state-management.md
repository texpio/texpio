# State management

React state owns drafts/interactions, Zustand shared UI/layout, TanStack Query native persisted data. `src/store/ui-store.ts` persists only panel widths/visibility under `texpio.layout`; dialogs, view and last quick entry are transient.

Use narrow scalar selectors, multiple selectors or shallow-selected objects. Do not destructure a whole subscribed store. Pure libraries may use `getState()` but cannot subscribe; hooks belong in `src/hooks/` or `src/services/`.

Preferences commit natively, reread actual state, refresh Query/cache and notify other windows. Shortcut ownership uses a distinct worker/command. First-paint caches cannot override newer committed settings.

Identify lifetime and persistence before adding state. Never turn failed native mutation into cached success. Test hydration, corruption, concurrency and teardown.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
