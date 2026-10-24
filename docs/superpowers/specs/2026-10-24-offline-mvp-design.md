# Windows offline MVP design

24 October 2026 · Autonomous execution requested by the user while offline

## Intent and decisions

Build a useful local Windows text expander with an independent Rust engine modeled on the boundaries studied in [Espanso](../../research/espanso-source-audit.md). The user selected independent implementation and then explicitly requested a working MVP, authorizing Computer Use and app testing while unavailable. Proceed without further approval waits. Existing Native/current-checkout execution and commit/push authorization remain in force.

Keep Tauri 2, React/TypeScript, shadcn, Bun 1.4.4 and the adopted shell. Use Rust-owned SQLite for content, typed IPC for management and immutable native rule snapshots for expansion. No Espanso runtime code or new project license is introduced.

## Scope and support

Deliver create/edit/duplicate/trash/restore, folders, title/trigger/body search, explicit save with validation, Unicode plain-text bodies, local persistence, versioned JSON export/import preview, conflict refusal and recoverable backups. Add a searchable quick snippet picker with explicit Copy; captured-target insertion may be enabled only when its target/caret semantics are actually verified. Preserve Preferences, themes, palette, tray and Developer probe.

Automatic expansion starts only through an explicit user action and is stopped on exit. It uses literal `;`-prefixed lowercase ASCII triggers followed by Space, with one trailing space retained. Start with ordinary Windows Notepad editor controls on the qualified US/English India layouts; broaden application support only with positive native evidence. Unknown, password, selected-text, elevated, composition or changed-target contexts refuse before mutation. No claim of all-app or all-platform support follows from this milestone. macOS, X11 and Wayland remain separate adapters/qualification work.

The original strict P0 fixture profiles and evidence remain separate. Do not quietly introduce a keyboard-history-only deletion mode. The offline MVP can be demonstrated through synthetic native tests, but those never count as the original 300 physical trials. The old P0/P1 release campaigns are not waived or declared complete by this development build.

## Architecture

React manages library records through TanStack Query; content and editor drafts are not stored in layout localStorage. Rust validates and commits entire library changes transactionally with an expected revision, protecting multiple windows from lost updates. SQLite lives under the app-data directory, separate from preferences. Never log snippet/clipboard/input content. JSON import is bounded, versioned, validated and all-or-nothing; conflicting IDs or active triggers refuse rather than overwrite. Exact existing records may be skipped. Export preserves identities.

The shared engine receives an immutable `RuleSet` of validated owned `Rule { id, trigger, body }` values. The supervisor passes the same snapshot to matcher and insertion worker. A candidate resolves only within that snapshot; arbitrary webview bodies cannot reach a native effect. Library saves/imports stop observation before publishing changed content, and require explicit rearming. Developer fixtures construct their own snapshot. Retain bounded buffers/queues, context/input generations, render-before-mutation, full submission counts, observed output and no repair/replay on uncertainty.

Insertion strategies remain separate from policy. Direct Unicode is the automatic backend for this milestone. Explicit Copy may replace clipboard text at the user's request; it is not an invisible automatic fallback. Native date/time rendering may be added only with a fixed declarative grammar and validation; clipboard/cursor tokens, rich text, images, scripts and network expressions are deferred if they would weaken this contract.

## UX and errors

Library is the initial view, with folders, search/list and editor. Use existing semantic theme tokens and accessible shadcn components. Show empty/loading/error/saving states, disabled snippets, trash and a readable preview. Warn before discarding an unsaved draft. Display engine status and declared application support in plain language; fault/refusal must remain understandable. Browser preview marks native services unavailable. Add new visible strings to all three locales.

## Acceptance and limits

Run meaningful storage tests for restart durability, validation, duplicate triggers, stale revisions, import rollback and export round-trip; matcher/runtime tests for dynamic rules, state invalidation and no-dispatch refusals; UI tests for create/save/search/errors and unsaved drafts. Run the complete existing checks, frontend build and Windows native build. Review the integrated diff independently before handoff.

Use short Computer Use sessions on task-owned synthetic documents to verify the app. End each session after its check and on any stalled action; no terminal/system/security automation. Never enable injected-event acceptance in normal builds simply to make automation look like physical typing. If a harness-only path is needed, isolate it explicitly in a test helper and report the evidence class. Record actual native results, screenshots and remaining failures in a durable MVP report. Leave an updated runnable preview for the user with observation stopped.

## Alternatives considered

The user chose independent Rust implementation over selected GPL source reuse or an Espanso sidecar. This retains our typed Tauri integration and target/outcome guarantees, while leaving native compatibility implementation and testing with Texpio. A broad history-based compatibility mode is deferred rather than silently substituted for verified suffix/caret behavior. No sync/backend/accounts/billing, macOS delivery or production signing is part of this Windows build.
