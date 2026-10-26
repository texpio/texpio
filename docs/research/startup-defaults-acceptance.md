# Startup defaults acceptance — 26 October 2026

Base: `14badab33edf0d5a5824b436d7b3bbaef581ce6f`. The user requested screen-
proportional startup bounds, Uncategorized/Search labels, and expansion enabled
on every fresh launch/restart despite stopping or pausing the preceding session.
They explicitly chose following the screen's aspect ratio rather than fixed 16:10.

## Behavior

- The main window starts centered using approximately 80% of its monitor's usable
  area, preserving the monitor resolution's aspect ratio. Rust reserves native
  frame space, accounts for DPI and grows to 800×560 logical pixels when possible.
  Smaller work areas clamp the minimum instead of placing the window offscreen.
- The window-state plugin skips initial restoration for main, preserving the
  saved file and plugin infrastructure. A running instance retains manual
  resizing/maximization; restoring it from the tray or second launch does not
  reset its geometry or its current expansion state.
- Every fresh Windows process attempts product startup once, including restart
  and hidden sign-in launch. Pause/Stop applies to the current session. The legacy
  `enabled` settings field is still accepted/round-tripped for compatibility,
  defaults to true and no longer gates startup. No settings rewrite is required.
  The conflicting launch preference switch is removed from Preferences.
- The startup attempt shares the existing library coordinator and admission
  guard. A newer Stop/Pause or already running session wins. Corrupt data, invalid
  native shortcuts, cancellation and actual startup failures still refuse; errors
  are reported. Shortcut registration conflict alone does not prevent expansion.
- Unfiled is now Uncategorized throughout navigation/editor/folder help; the
  command-search entry is Search. French and Arabic equivalents and startup help
  are updated. Internal folder IDs, palette commands and input guards are intact.

## Verification and review

Pure geometry tests cover landscape/portrait, negative origins, DPI/minimums,
small work areas and invalid inputs. Fake desktop backends verify initial startup
despite legacy enabled=false, a second fresh session, no settings rewrite, actual
registration conflict and invalid settings. Library tests verify skipping startup
after running/Pause/Stop and refusing corrupt settings without touching input.

A fresh read-only review found one P2: native shortcut canonicalization could
return InvalidSettings yet still reach startup. A regression failed with
`invalid native shortcut must not start expansion`; a guard now refuses that
case while preserving startup after actual registration conflicts. Final full
checks passed after the fix; this is not a second review approval.

`bun run check:all` passed **163 frontend tests, 197 Rust library tests, 12 tooling
tests, one fixture-server test and all static gates**. Bindings were regenerated
and remained unchanged. `bun run build` passed. Ordinary checks kept constructors
stopped and used fake backends; none installed hooks or sent native input.

An isolated headless Chrome production-frontend fixture passed renamed labels,
the removed switch, new startup explanation and a synthetic Running status. It
also retained pending settings saves/drafts across search/Data/navigation and
passed English/French/Arabic, light/dark and compact layout checks. The browser
and server ended. This is UI evidence, not actual engine startup or screen sizing.
Ignored local artifacts live under `.superpowers/sdd/2026-10-26-startup-defaults/`.

## Native build and preview

The user saved/Quit and acknowledged closure before the first native build.
The locked optimized no-bundle build passed (5m 07s native compilation), and the
normal preview reopened at the expected path (observed PID 10804). A bounded,
read-only native inspector verified **Running** in the Automatic expansion region,
without any simulated input or preference/library change.

That same inspector correctly failed the sizing assertion: on a 2560×1440 display
at 120 DPI, the client was 1896×1103. It was centered, not maximized and inside the
work area, but its aspect ratio was wrong. Installed Tao source shows that showing
the frameless window recalculates its native frame; measurements before show can
be stale. Sizing now runs after the first successful show with a process-owned
one-time guard. Tests verify show-before-fit, preserving later manual bounds and
retrying a failed show without consuming the first fit. The full suite passed
again. The first preview was preserved until the user saved/Quit and acknowledged
closure again; it was not described as size-qualified.

The corrected locked optimized no-bundle build passed (4m 40s native compilation).
The normal preview reopened without remote-debugging arguments, observed PID 10328.
The same read-only inspector then passed: visible/not maximized, client 1948×1096
on the 2560×1440 screen at 120 DPI, screen ratio matched within pixel rounding,
centered, entirely inside the work area, and Automatic expansion **Running**.
No input, clipboard operation or library/settings mutation was performed by the
verification. The inspector process ended; the normal app remains open and running.
Executable SHA-256:
`46E5CE9DD2FD771205C7521D8D0EA15F04D1B21F8850A224BDF9C1F83C7FC651`.
Always reobserve process identity and preserve drafts before replacing it.

The normal preview starts expansion automatically, as explicitly requested.
Use headless fixtures for ordinary UI checks; old native acceptance helpers must
not assume that a legacy enabled=false file suppresses startup anymore. Strict
probe/input-origin policies and the separate P0 campaign remain unchanged.

See the [design](../superpowers/specs/2026-10-26-startup-defaults-design.md) and
[plan](../superpowers/plans/2026-10-26-startup-defaults.md).
