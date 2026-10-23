# Error handling

Rust services return closed enums. TypeScript facades validate replies and redact native failures. Do not display exception strings, paths, URLs or arbitrary objects. Generated types cannot validate unknown runtime data.

Preferences distinguish missing from corrupt data; native failures show a fixed localized error and Retry, without caching success defaults. Retries must match operation semantics; never blindly retry input or shortcut mutations.

`ErrorBoundary` renders localized fallback, saves redacted diagnostics and reports actual save success/failure. Restart requires confirmed probe shutdown. React root callbacks avoid raw caught-error logging.

Shortcut replacement registers inactive ownership before saving and rolls back persistence/registrations on failure. Inactive cleanup handles are retained for retry. Test cancellation, late resolution and partial failure as well as happy paths.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
