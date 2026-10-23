# Quick pane

Root `quick-pane.html` uses `src/quick-pane-main.tsx` and a narrow capability. It shares appearance/i18n providers. Vite 8 uses `build.rolldownOptions.input` for both HTML entries.

Windows/Linux use a hidden undecorated always-on-top/no-taskbar window positioned at the cursor monitor with DPI handling. macOS uses a key-capable/non-main NSPanel with spaces/fullscreen support and resignation on hide. Native acceptance is target-specific.

Enter emits one bounded explicit entry to main; Escape or blur clears/cancels. Repeats/composition cannot submit; delayed listeners clean up. Main displays only the last transient entry. No external-app insertion or arbitrary typing capture occurs.

The default global chord is CommandOrControl+Shift+Period. Rust owns safe rebinding and rollback. Quick-pane window state is not persisted, and its capability has no filesystem, recovery, updater or process-restart powers.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
