# Testing

Use `bun run test` for one Vitest run, `test:watch`, optional `test:ui`, and `test:coverage`. `bun test` is a different runner. DOM tests use Testing Library/jsdom and narrow native mocks. No ordinary suite installs hooks/sends input.

Locked `rust:test` uses disposable persistence fixtures and fake shortcut backends. `test:probe-fixture` verifies the browser target server; `test:tooling` covers safe helpers/skill mapping; `ast:test` checks rule fixtures. `test:all` combines them.

Test visible results, malformed replies, cancellation, StrictMode replay and rollback. Avoid implementation-mirroring tests. Regenerate bindings and build both entries.

Native visual acceptance, notification delivery, OS shortcut conflicts and physical expansion trials remain separate evidence. Run `check:all`, frontend build and applicable native no-bundle build before claiming completion.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
