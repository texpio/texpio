# Notifications

`src/lib/notifications.ts` uses native delivery where available, with Sonner toast fallback. It returns `native`, `toast` or `unavailable`; API success does not prove visible delivery. Native commands bound content and never log titles/bodies.

Use notifications for meaningful user outcomes, inline state for persistent errors. Developer includes a test notification action. OS permission, Focus Assist and packaged identity affect delivery.

Convenience helpers preserve typed severity/content and failure handling. Main owns Sonner. There is no background cloud notification service. Test native failure/fallback/unavailable paths and verify packaged delivery separately.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
