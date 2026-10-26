# Settings layout acceptance — 26 October 2026

Base: `17ec921ad8c38ba28dba137fdbdaa723d7e63b52`. The user supplied a Multica
settings screenshot and requested that layout for Texpio's Settings page.

## Result

- The main library sidebar remains. A separate settings sidebar contains the
  Preferences heading, search, icon-led General/Expansion/Shortcuts/Data navigation
  and the existing version/update entry. It scrolls separately from the content.
- Content has a section heading, description and bordered groups of setting rows.
  Labels/help and controls occupy separate columns; narrow rows stack. Existing
  theme, typography, controls, native services and all settings remain.
- Search uses translated section names and setting labels, selecting the first
  matching section when necessary. No matches has visible feedback; clearing
  search restores navigation without resetting the selection or saving settings.
  Search does not index snippets, paths or preferences values and is not persisted.
- One keyed ExpansionSettings instance remains mounted across search, sections
  and main-page navigation. Pending saves stay serialized and failure feedback is
  retained. Existing library draft and data-location/backup guards remain.
- New strings are in English, French and Arabic. No dependency, lockfile, Rust,
  generated binding or native permission change was needed.

## Verification and review

The new PreferencesPage search regression failed with a missing searchbox before
implementation and passed afterward. Existing tests cover theme persistence,
corrupt-preference recovery, delayed settings saves, acknowledged-value merging,
failed-save visibility and library draft retention. A test-only unsupported
Testing Library query option was corrected during typechecking.

Final `bun run check:all` passed **163 frontend tests, 188 Rust library tests,
12 tooling tests and one fixture-server test**, plus all static gates. The initial
post-review formatting check caught LibraryLocation formatting; it was formatted
before the complete successful rerun. The production frontend build passed.

A fresh read-only review found one P2 layout issue: long translated action buttons
could overflow the control column. A headless Chrome regression reproduced a
French Data row with scroll width 514 versus visible width 489 at a 1000 px window.
The control column now uses a constrained flex basis; action and shortcut-capture
buttons wrap inside their available width. The same assertion subsequently passed
at 800, 1000 and 1400 px, including active French shortcut capture. This records
the review and regression-tested fix, not a second review approval.

The isolated production-frontend fixture uses synthetic in-memory native replies.
It passed separate navigation/content geometry; English search and empty results;
delayed saves across search, Data, other sections and leaving/reopening Settings;
acknowledged preference state; unsaved snippet preservation; light/dark, every
section at 800×560, Arabic RTL and French layouts. Final screenshots were inspected.
No page errors, global-input starts or library saves occurred. Every browser and
fixture-server session ended. Logs/screenshots/script are ignored local artifacts
under `.superpowers/sdd/2026-10-26-settings-layout/`.

Native OS hotkeys, dialogs and expansion qualification do not inherit this UI
result. The normal profile was not used by these fixtures.

## Native build and preview

The user saved/Quit and acknowledged that the old preview was closed. A process
check confirmed it had exited before rebuilding. The locked optimized no-bundle
Tauri build passed; native compilation took 4m 58s. No installer or signed release
was produced.

The normal release executable was reopened without remote-debugging arguments.
A read-only check confirmed its expected path, title Texpio, live process and
visible window (PID 6948 at verification). Open Preferences in its sidebar to view
the new layout. Always reobserve process identity and preserve drafts before
future executable replacement. Executable SHA-256:
`A894C90D9905B64069538BE6B377B72E774E6C5BF28C1A966ADBD6422B6D4613`.

See the [design](../superpowers/specs/2026-10-26-settings-layout-design.md) and
[plan](../superpowers/plans/2026-10-26-settings-layout.md).
