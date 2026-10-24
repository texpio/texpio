# Library workspace implementation plan

**Goal:** Make Texpio a focused snippet library with a dedicated Preferences page.
**Spec:** [Workspace design](../specs/2026-10-25-library-workspace-design.md).
**Architecture:** One left navigation and a list/editor workspace. Rust-owned
preferences use existing services; UI store holds navigation/filter IDs and layout.
**Stack:** Pinned Bun 1.4.4, React/TypeScript, installed shadcn/Radix, Tauri/Rust.

## Constraints

No dependency upgrades or input-engine changes. Preserve the theme, three root
locales, explicit Save, draft/revision guards and listener cleanup. Current checkout
and eventual commit/push are authorized. No hooks/input in ordinary checks.

## Tasks

- [x] Read-only builder check of design; resolve implementation questions from
  existing context without an additional approval loop.
- [x] Shell and navigation: test removed right/Developer commands and old layout
  migration, then implement two-pane shell, filtered folder navigation and a real
  Preferences view. Remove obsolete dialog/entry-retention state. Keep the library
  mounted but hidden on Preferences and disable its command bindings there.
- [x] Preferences: organize existing controls into General/Expansion/Shortcuts/Data,
  retaining native settings ownership, failures, shortcuts and backup guards.
  Test page semantics, section navigation and saving.
- [x] Library: reorganize the existing save/import/draft logic into a compact
  list/editor layout; use stored filter IDs, explicit Save, focused New snippet,
  selected-state rows, content/preview/attachments choices and clear empty states.
  Retain and run existing behavioral tests; add navigation/draft regressions.
- [x] Render isolated desktop fixtures in light/dark and compact sizes, inspect
  screenshots, fix visible defects, run full check:all and frontend/native builds.
- [x] Independent whole-change review and bounded fix pass; reopen the new preview,
  update TASKS/acceptance evidence and commit/push the verified change.

## Review focus

Hidden editor losing a draft; duplicate keyboard/listener registration; filter IDs
stale after a data-location change; collapsed panes hiding navigation at startup;
preview/attachment choices accidentally bypassing validation or revision guards.
