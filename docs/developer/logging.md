# Privacy-preserving logging

Use `src/lib/logger.ts` with fixed DiagnosticEvent values and optional safe unsigned count. Unknown text maps to `unknown`; extra metadata is discarded. Do not log snippets, quick entries, clipboard, titles, paths, credentials, URLs or raw errors.

Rust validates a deny-unknown-fields DTO and writes only to target `texpio::diagnostic`. Native logging filters that exact target, uses stdout/app log directory, rotates at 1 MiB and keeps one previous file. Dependency updater logs and Webview log targets are excluded.

Browser development logging prints the same bounded record. Logging failures cannot recursively log. Recovery contains fixed event/version/time only. New diagnostics need an enum event and tests proving input cannot reach fields.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
