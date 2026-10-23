# Typed Tauri commands

Rust DTOs/commands feed Specta and tauri-specta. `bun run rust:bindings` exports `src/lib/bindings.ts` deterministically through a standalone example, without starting app/observer/hooks. Do not hand-edit generated types.

`src/lib/tauri-bindings.ts` and the probe facade validate unknown replies, classify errors and redact exceptions. Browser tests mock narrow adapters. Types describe contracts but do not validate runtime data.

For a command: bound DTO/errors, register builder/AppManifest, scope capability, export bindings, wrap/validate and test. Main and quick-pane permissions intentionally differ.

Families include metadata, appearance, recovery, notification, pane/shortcut, update readiness, safe relaunch, redacted diagnostics and experimental probe controls. Inspect generated declarations for exact signatures.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
