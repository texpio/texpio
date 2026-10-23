# Tauri plugin infrastructure

Shell plugins include single-instance, OS, notification, log, process, global-shortcut and window-state. Updater initializes only with valid real configuration; macOS also uses revision-pinned NSPanel.

Filesystem, dialog, opener, clipboard-manager and persisted-scope infrastructure is retained for future authorized features. Filesystem registers before persisted-scope. Installed plugins do not grant blanket webview access. Capabilities list only window requirements. P0 clipboard experiments remain disabled without reads/writes.

Check exact installed APIs, OS support, initialization order, permissions and failures against official docs before using a plugin. Narrow Rust commands may contain privileges better than frontend defaults. Menu/window/event APIs are built-in.

HTTP, SQL, secrets, autostart and shell plugins are future options, not current features. Pinned installation is not cross-platform runtime proof.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
