# Keyboard shortcuts

Application shortcuts have one DOM owner in `src/hooks/use-keyboard-shortcuts.ts` and dispatch shared command IDs. Native Edit roles keep OS accelerators; custom app menu actions do not duplicate those DOM bindings.

Match exact platform modifiers, ignore handled/repeated/composing events and extra modifiers, and leave ordinary editable keys alone. Prevent default only for a recognized available action. These chords require focused webview.

The separate global quick-pane chord defaults to CommandOrControl+Shift+Period. Preferences capture locally with Escape/blur cancellation and reset. Rust canonicalizes, serializes registration and preserves the old binding on conflict/save failure.

Tests use fake registration; native OS/layout/conflict acceptance is separate. P0 abbreviation recognition is another subsystem and is never enabled by shortcut tests.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
