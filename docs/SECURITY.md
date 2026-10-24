# Security guidance

This is a pre-release foundation with an experimental guarded engine. No version is production-qualified. Report non-sensitive concerns through the repository's available reporting channels. A private security contact and response SLA have not been established; do not publish secrets or private input in public issues.

CSP lives in `src-tauri/tauri.conf.json`, limits connections to IPC and avoids remote scripts. Capabilities enumerate commands by window; quick-pane can explicitly write clipboard text for Copy, but has no clipboard-read, filesystem, recovery, updater or process-restart powers. Installing plugins does not grant blanket webview access.

Rust validates bounded DTOs, recovery paths and updater configuration. Preferences fail explicitly on corruption and write atomically. Diagnostics contain fixed enum events/counts only. No input, snippets, clipboard, titles, paths, credentials or exception stacks belong in logs/recovery.

The P0 engine requires explicit arming and independent native acceptance. Ordinary suites never install hooks/send input. Automatic clipboard insertion remains disabled; explicit Copy is a separate user action. Real signing/feed and platform certificate/notarization setup are required before distribution.

Adapted from [tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../THIRD_PARTY_NOTICES.md).
