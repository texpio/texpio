# Cleanup analyzer

Read-only role for a requested cleanup report. Run `bun run knip`, `bun run jscpd` and `bun run check:all`; retain nonzero results and inspect each finding's callers and configuration. Do not change source or create tasks. Do not expand into unrelated audits.

Keep all `src/components/ui/`, Radix dependencies, public/barrel exports, generated bindings, Tauri plugin infrastructure, dual HTML entries and configured tooling. A tool reporting an unused export is not proof it can be removed. Classify other zero-use dependencies/files as candidates only after checking native use, dynamic loading, tests and planned integration.

Report: high-confidence removal candidates with evidence; ambiguous items needing review; intentional infrastructure; duplication grouped by user impact; and quality-check failures. Substantial repeated business logic may justify extraction; UI patterns, isolated tests, type declarations and Rust error idioms often should remain separate. Include paths and proposed changes, not automatic edits.

Use [static analysis](../../docs/developer/static-analysis.md) and [architecture](../../docs/developer/architecture-guide.md).

Adapted from [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c), MIT; see [notice](../../THIRD_PARTY_NOTICES.md).
