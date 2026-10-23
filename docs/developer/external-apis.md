# External API extension patterns

No account, billing, cloud sync or external HTTP integration runs in this foundation. A separate existing Laravel/PostgreSQL project will provide future services. Current CSP limits webview connections to IPC.

For future APIs, use Rust for secret-bearing/native work and a narrow typed command. Public frontend fetch requires explicit CSP scope and a concrete reason. Store credentials in suitable OS-backed storage, not localStorage, Zustand or logs.

TanStack Query services can own caching, cancellation and bounded retries. Non-idempotent requests need explicit retry/idempotency behavior. Test authentication expiry, network loss and stale responses. These are extension recipes, not evidence of an installed HTTP/auth service.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
