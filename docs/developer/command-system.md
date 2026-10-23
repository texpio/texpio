# Command system

Commands in `src/lib/commands/` supply an ID, translated label/description/keywords, availability and an execution function. `executeCommand` returns closed unknown/unavailable/failed outcomes without raw exceptions.

Palette, native application/context menus, titlebar actions and DOM shortcuts share these IDs. Add registry behavior before referencing a surface. Palette search supports translation and diacritics; availability is checked again at execution. Preferences opening is idempotent.

Inject context for window/platform actions and toast/preferences callbacks. `src/lib/window-actions.ts` isolates native calls. Hooks own listeners and effects. Native Edit roles keep OS semantics. There are no fake Open/Save actions for an unimplemented document model. Test registry outcomes and surface dispatch.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
