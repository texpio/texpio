# Settings and expansion controls acceptance — 26 October 2026

Base: `5cceef08c45cfa6365efa374989331b8bb220605`. The user requested moving
the library's always-visible expansion bar into Settings, renaming Preferences
to Settings, and showing a warning with an enable action while paused/stopped.

## Result

- Settings → Expansion contains runtime status and Start/Pause/Resume/Stop controls.
  A healthy running engine occupies no space in the library workspace.
- A persistent alert below the title bar follows the user across pages while
  expansion is paused or stopped. Turn on expansion resumes/starts through the
  same registered actions as the Settings controls. Pending actions disable both
  surfaces. Faults require explicit Reset cleanup before Start is available.
- Status failures show an unavailable message and Retry. Cached paused/fault
  data does not assert the current engine state after a failed status poll.
  Native command errors and incomplete clipboard-restoration warnings remain
  visible even while expansion is running or Settings is hidden.
- One main-window query/mutation owner retains pending state and errors across
  navigation. Starting requires a saved enabled, non-trash snippet and a clean,
  idle library editor. Only a transient readiness boolean enters UI state; no
  snippet content or runtime status is persisted there. Unmount fails closed.
- English, French and Arabic visible Preferences labels are now Settings.
  Persisted navigation IDs, command IDs, shortcuts and native preferences remain
  compatible. Existing primitives/theme tokens are preserved; no new dependency
  or Rust/native behavior changes were introduced.

## Verification

`bun run check:all` passed: **171 frontend tests, 197 Rust tests, 12 tooling tests,
one fixture-server test**, typechecking, lint, formatting, AST gates and Clippy.
`bun run build` passed. Tests use synthetic native replies and install no hooks.

New behavior tests cover shared pause/resume, fault cleanup, empty/disabled/trash
libraries, dirty drafts, pending saves, unmount cleanup, pending actions/errors
across Settings unmount, unavailable-status Retry, cached-phase poll failures and
clipboard restoration warnings. The existing draft/settings retention tests pass.

Independent read-only review found a stale-status message after a failed poll
and a French agreement typo. Both are fixed; the stale-status regression and full
suite pass. This records the review/fix pass, not a second review approval.

An isolated production-frontend headless Chrome fixture passed healthy-library
bar removal, Settings Pause, persistent library warning, one-click enable and
dirty-draft gating. It also passed settings search, pending preference saves and
draft retention across sections/pages, light/dark, 800×560 compact layouts, French
and Arabic RTL. Rendered images were inspected. Browser and loopback server ended.
Ignored evidence is under `.superpowers/sdd/2026-10-26-expansion-controls/`.
These are UI checks with synthetic replies, not native input qualification.

## Native preview

The previous normal preview (observed PID 10328) was preserved while coding and
testing. The user saved/Quit and acknowledged closure, and process absence was
verified before replacing the executable. The locked optimized no-bundle build
passed, including a fresh frontend build and native compilation (4m 33s).

The normal preview reopened from `src-tauri/target/release/texpio.exe` without
debugging arguments, observed PID 25500. A bounded read-only native inspector
confirmed the visible Settings button, absence of the old Preferences label and
the library's Automatic expansion region, and centered screen-proportional bounds:
client 1948×1096 on a 2560×1440 screen at 120 DPI, inside the usable work area and
not maximized. The inspector ended; the normal app remains open. It performed
no clicks, typing, clipboard operations or library/settings writes. Actual native
Pause/Turn on interaction was not exercised; state transitions were verified with
the synthetic production-frontend fixture and unit tests above.

Executable SHA-256:
`D9A4AC7EEEE6F0E1660664BEEE04A6F24DF70262098AEE2BA51A7537AD7B6DBB`.

Fresh native launches still start expansion automatically under the previously
approved startup policy. Strict P0 and other physical input qualification remain
separate; this UI refactor does not claim new native input acceptance.
