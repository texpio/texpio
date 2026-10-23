# Cross-platform development

Windows is first, macOS next; Linux has a shell/build path without a supported-release claim. Platform adapters choose native behavior; a browser preview reports unknown and must not pretend to be macOS.

Windows/macOS custom titlebars and Linux native decorations use separate Tauri platform configurations. Main close hides to tray; second launch restores/unminimizes/focuses it. macOS Dock reopen also restores main. Window-state excludes visibility and quick-pane state.

Use logical CSS for Arabic and native path APIs in Rust. Never accept arbitrary frontend filesystem paths. Windows/Linux quick panes are floating windows; macOS uses nonactivating NSPanel. Monitor/DPI behavior, menus and shortcut conflicts need native acceptance per target. A Windows build does not establish macOS/Linux runtime correctness.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
