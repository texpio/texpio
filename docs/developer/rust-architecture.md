# Rust architecture

`src-tauri/src/lib.rs` composes plugins, state, commands and lifecycle. `commands/` orchestrates IPC; preferences/recovery/notifications modules own data. `shortcuts.rs` serializes registration/persistence. `desktop.rs` owns tray/windows. `engine/` isolates the cfg-gated native engine.

Use serializable DTOs and closed Specta errors. Keep tokens/COM/context decisions on native workers. Bound queues/blocking calls; never hold an app mutex across main-thread plugin APIs.

New commands require Specta registration, AppManifest entry, scoped capability, binding export and validating frontend adapter. Export does not run the app. Tests isolate OS-independent logic from native APIs.

Order plugins deliberately: single-instance first, filesystem before persisted-scope, managed state before setup consumers. Run locked checks and native linkage after dependency/capability changes.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
