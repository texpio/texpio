# Immediate expansion implementation plan

**Goal:** Expand on completion and after Backspace correction on Windows.
**Spec:** [Design](../specs/2026-10-25-immediate-expansion-design.md).
**Architecture:** Pure bounded matcher produces explicit deletion plans; existing
hook and effect worker retain target/cancellation protection. Rust owns settings.

## Constraints and review focus

Pinned Bun 1.4.4 / Node 24.18.0; no dependency upgrades. Current checkout execution
and commit/push are already authorized. Ordinary checks never arm global input.
Review suppressed final keys, Backspace deletion counts, prefix collisions,
truncated word context, and shortcut resets. Do not guess cursor positions.

## Tasks

- [x] Reproduce immediate-match and typo-history failures in matcher tests.
- [x] Add `Activation::{Immediate,Terminator}` (default Immediate), bounded history,
  Backspace rechecking, `Expansion.erase_count` and optional terminator. Cover
  Unicode, overlap, truncation and reset cases with pure tests.
- [x] Integrate one-character translation and plain Backspace match submission in
  the hook; reset shortcut edits. Test accepted/refused queue and deletion plans.
- [x] Add Rust-owned `activation` preference, regenerate bindings; use the existing
  Preferences select and localized labels/help. Test selection persistence.
- [x] Run targeted and full checks, independent review, then native build.
- [x] Offer a fresh user typing check after the build is available; record exact
  native evidence separately from unit results.
- [x] Prepare the reviewed and verified work for the authorized commit/push after packaging.

## Evidence

Base: `199ec0f693983f3405da45507621139b8005777c`. Investigation confirms delimiter-only
matching and capacity of longest-trigger + 1. Hook previously handled Backspace
before shortcut rejection. The two matcher reproductions failed before the fix;
the full final run passed 154 frontend and 188 Rust tests. The user-typed native
correction produced exact output. Details and cleanup limitations are in the
[acceptance record](../../research/immediate-expansion-acceptance.md).
