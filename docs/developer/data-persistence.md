# Data persistence

Use local state for drafts, persisted Zustand for non-sensitive layout, native preferences for appearance/shortcut configuration. `texpio.layout` excludes dialogs, active view and quick entries. First-paint appearance caches are not authoritative.

Rust serializes preferences and writes synced temporary files with atomic replacement. Appearance writes preserve the worker-managed shortcut. Missing state may initialize defaults; corrupt state returns an error and Retry instead of overwriting data.

Recovery accepts bounded filenames/bodies, rejects traversal/reserved names/symlinks and cleans files older than seven days. Crash recovery currently stores only fixed event/version/timestamp, not snippets or stacks.

New persistent data requires explicit schema, migration, integrity and recovery contracts. SQLite and encrypted sync are future options, not installed product services. Keep secrets and input out of logs/layout storage.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).

On Windows, eligible access/sharing/lock errors during atomic replacement retry at most five times with 10ms pauses against an existing non-readonly destination. The staged file is retained; the destination is never deleted first. Persistent/read-only failures preserve previous bytes and return the same closed I/O error.
