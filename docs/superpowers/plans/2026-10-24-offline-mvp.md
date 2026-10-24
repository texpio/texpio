# Windows Offline MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for the integrated Native execution. Parallel helpers may own independent storage/UI files; the primary agent owns integration and one final fresh review.

**Goal:** Deliver a runnable Windows offline snippet library with guarded native expansion and documented native evidence.

**Architecture:** Rust owns SQLite and validated immutable rule snapshots. React manages records through typed commands/TanStack Query. Existing strict target checks, tray lifecycle and Developer probe remain in place.

**Tech Stack:** Existing Tauri 2, React/TypeScript, shadcn, Bun 1.4.4, Rust/windows 0.62.2; bundled rusqlite for the library.

**Spec:** [Offline MVP design](../specs/2026-10-24-offline-mvp-design.md). The user authorized autonomous execution while offline; no further review waits are required.

## Global constraints

- Native/current-checkout execution; preserve pinned tooling and existing library data.
- Never arm global input in ordinary checks, log content or count synthetic input as physical trials.
- Immutable native rule snapshots; stop before library changes; no clipboard automatic fallback.
- Explicit unavailable/refused states for unsupported controls/platforms; retain separate P0 evidence.
- Computer Use sessions are brief, target-owned, bounded and ended after each check.

## Review focus

- Stale editor revisions and failed saves preserve the user's draft and acknowledged content.
- Imported conflicts/oversized data never leave a partly changed library.
- A deleted/disabled/changed snippet cannot expand from an obsolete active session.
- Pause/stop/focus/caret changes cancel queued native work; partial submission never retries.
- Native tests distinguish synthetic insertion, synthetic observer paths and physical trials.

### Task 1: Durable library

**Files:** Create `src-tauri/src/library/` modules and tests; modify Cargo dependencies/lock. Integration registration belongs to Task 3.

**Interfaces:** `LibraryStore` owns an app-data database path and exposes load/save/export/import-preview/import. DTOs: Library, Folder, Snippet, SaveLibrary, ImportPreview and LibraryError. Exact IPC contract is recorded in the execution workspace before dispatch.

- [ ] Write/run failing tests for restart, stale revisions, active trigger collisions, deleted records, import conflict rollback and round-trip.
- [ ] Implement bounded validation, schema migration, transaction/revision checks and backup-before-import.
- [ ] Run focused tests and inspect output; commit the verified library unit.

### Task 2: Library UI and picker

**Files:** Create `src/features/library/`, `src/services/library.ts`; adapt `src/components/layout/MainWindow.tsx`, quick pane and three locales. Do not edit generated bindings.

**Interfaces:** Consume typed library commands and engine library-start/status/stop commands via a service facade. Use Library/SaveLibrary DTOs from Task 1. Copy is an explicit clipboard action.

- [ ] Write/run meaningful UI tests for editing/save conflict, search, unsaved draft and import preview.
- [ ] Build folder/list/editor UI, trash/restore/duplicate, validation, explicit copy, JSON file workflows and searchable picker.
- [ ] Preserve shell navigation, preferences, command registry and themes; run focused Vitest/typecheck.
- [ ] Commit verified UI after generated bindings and integration are available.

### Task 3: Independent dynamic engine and typed integration

**Files:** Modify `src-tauri/src/engine/` rule/recognizer/observer/worker/context/profile modules, add library command facade, register state/bindings and update tray/quit integration as needed.

**Interfaces:** `RuleSet` is an immutable validated native snapshot, shared by observer and worker. Fixture construction uses the same shape without granting runtime content from arbitrary candidates. `library_start` loads enabled non-trash snippets and selects generic positively identified Notepad editors; `library_save` and `library_import` stop the active session before committing.

- [ ] Write/run failing tests for owned dynamic rules, invalid/colliding rules, disabled/trash exclusion and preserved strict fixture profiles.
- [ ] Implement rule snapshot threading through existing matcher/native pipeline, retaining all target/outcome checks.
- [ ] Add typed asynchronous library command facade, state initialization, start/stop/status controls and binding generation.
- [ ] Run Rust/frontend checks and no-bundle Windows build; commit verified integration.

### Task 4: Native acceptance, review and handoff

**Files:** Add `docs/research/offline-mvp-acceptance.md`; update TASKS, README, docs index and technology decisions.

- [ ] Exercise isolated app/library/persistence/copy/import/export and guarded native synthetic fixtures with bounded helpers/Computer Use; retain exact observations.
- [ ] Run `bun run check:all`, frontend build and Windows native build; inspect every result.
- [ ] Dispatch one fresh whole-change reviewer with spec/plan/evidence, fix important findings with regressions, rerun affected/full checks.
- [ ] Commit/push, verify remote SHA, stop all task automation/hooks/servers and launch a stopped runnable preview. Record unmeasured physical/release/platform gates honestly.
