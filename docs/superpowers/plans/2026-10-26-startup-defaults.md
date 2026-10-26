# Startup Defaults Implementation Plan

> **For agentic workers:** Use the established native/current-checkout execution workflow.

**Goal:** Follow screen proportions at launch, start expansion afresh, and shorten navigation labels.
**Architecture:** Rust computes physical window bounds and orchestrates one guarded startup attempt. Existing persisted settings remain readable; runtime pause/stop remains transient.
**Tech Stack:** Pinned Bun 1.4.4, Tauri/Rust, React/TypeScript and existing shadcn.
**Spec:** [Startup defaults](../specs/2026-10-26-startup-defaults-design.md).

## Constraints and review focus

No dependency changes or weakened input guards. Never arm global input in ordinary
checks. Preserve cancellation, corrupted-file refusal, input-origin rules, draft
guards and all locales. Cover negative monitor origins, portrait, small/high-DPI
work areas, stale startup after Stop/Pause and existing legacy enabled=false.

## Implementation

- [x] Add failing pure geometry tests and fake-backend startup orchestration tests.
- [x] Implement monitor-aware sizing, skip old main-window restore, and a guarded
  startup path that ignores the legacy enabled flag and reports genuine failures.
- [x] Remove the launch switch; translate labels, folder help and startup copy.
  Verify its removal and unchanged settings-save behavior in frontend tests.
- [x] Run focused/full checks, frontend build and headless UI fixtures. Request
  read-only review and regression-test important fixes.
- [x] After save/Quit acknowledgement, build/reopen the native preview (expansion
  starts automatically), record evidence, update TASKS and commit/push.

## Native acceptance correction

The first native build opened with expansion Running but failed the aspect-ratio
assertion because startup measured the frameless border before its first show.
Move fitting after the first successful show with a process-owned once guard;
cover failed show and repeated restore in tests. Full checks pass with 197 Rust
tests. After another save/Quit acknowledgement, the corrected native rebuild and
read-only startup inspection passed: screen ratio, bounds, centering and Running.
