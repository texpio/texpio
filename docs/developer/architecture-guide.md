# Architecture

Components compose shadcn primitives; hooks own subscriptions/effects; services use TanStack Query for native data; pure libraries contain reusable non-React logic. Rust owns preferences, recovery, shortcuts, tray lifecycle and the guarded engine. UI state belongs in `src/store/ui-store.ts`.

Use local React state for an interaction, Zustand for shared transient/layout state, Query for native persisted data. Subscribe with narrow selectors; pure command code reads `getState()` without subscribing. Do not persist quick-pane/probe text.

Both root HTML entries have independent React roots. Shared providers maintain appearance, localization and error handling. Payload-free preference events trigger native rereads. Cleanup must survive StrictMode and delayed native registration. Capabilities differ by window.

The main window has library and Preferences views. Both remain mounted while
hidden so draft guards and pending settings mutations survive page navigation.
Library filters are transient UI-store IDs; sidebar/list widths are persisted
through layout normalization. A single product-settings mutation controller
serves every Preferences section. The Developer route and global right sidebar
are no longer product surfaces; native probe qualification remains separate.

Register actions once in `src/lib/commands/`. Typed IPC facades validate closed errors; generated bindings alone are not a trust boundary. Native context tokens stay on the worker. See [state](state-management.md), [commands](command-system.md), [security](../SECURITY.md) and [testing](testing.md).

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
