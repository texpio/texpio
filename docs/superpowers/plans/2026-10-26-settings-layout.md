# Settings Layout Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans for the established native/current-checkout workflow.

**Goal:** Adapt Preferences to the user's Multica settings reference.
**Architecture:** A settings sidebar and independently scrolling content pane;
local search finds translated section/settings labels. Existing services and the
single mounted product-settings mutation controller retain persistence ownership.
**Tech Stack:** Pinned Bun 1.4.4, React/TypeScript, installed Radix/shadcn and Tauri.
**Spec:** [Settings layout](../specs/2026-10-26-settings-layout-design.md).

## Global constraints

Preserve theme, typography, current-checkout execution, all three locales and
native save/draft guards. No dependencies or Rust changes. No global input checks.

## Review focus

- Search/navigation must not remount the product mutation owner during a save.
- A hidden or failed settings save remains serialized and visible after searching.
- Narrow content, long translations and RTL must not overflow the application.
- Labels, descriptions and keyboard navigation remain associated with controls.
- Search indexes labels rather than preferences values or snippet content.

## Task 1: Settings layout and search

Files: PreferencesPage/AppearanceSettings/ExpansionSettings/QuickPaneShortcut/
LibraryLocation in `src/components/preferences/`; add `SettingsLayout.tsx` for
bordered groups and responsive rows. Translate new search, empty-state and section
descriptions in `locales/{en,fr,ar}.json`.

- [x] Add a behavioral PreferencesPage test: searching `clipboard` finds Expansion
  and Shortcuts and opens Expansion; clearing restores General; a no-match query
  shows feedback without saving. Run focused tests and observe the missing search.
- [x] Build the sidebar, search and responsive rows with installed primitives;
  retain component identity for the existing ExpansionSettings controller.
- [x] Pass PreferencesPage/ExpansionSettings/ShortcutPicker/MainWindow tests,
  including existing delayed-save and failed-save regressions.
- [x] Run `bun run check:all`, frontend build and rendered light/dark, compact and
  RTL fixtures. Exercise search and delayed saves; inspect actual screenshots.
- [x] Complete fresh read-only review and fix important findings with regressions.
- [x] After save/Quit acknowledgement, build/reopen the native preview, record
  acceptance evidence, update TASKS and commit/push the verified change.
