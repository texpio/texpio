# Immediate expansion acceptance — 25 October 2026

Base: `199ec0f693983f3405da45507621139b8005777c`. The user confirmed that the
reported correction used Backspace at the end, then closed Texpio for rebuilding.

## Behavior

- Immediate activation is the default, including existing settings files which
  have no activation field. `;hello` matches on the final `o`.
- `;helllo` → Backspace twice → `o` matches. The same correction after a failed
  trailing Space works after deleting Space, `o` and the extra `l`.
- Space/punctuation activation remains selectable. The terminating-character
  preference is retained but disabled while immediate mode is selected.
- The first complete abbreviation wins in immediate mode. Delimiter mode remains
  available for overlapping abbreviations such as `;he` and `;hello`.
- Corrections retain at most 512 Unicode scalars with explicit truncated-left
  context. Shortcuts, navigation, pointer activity and uncertain translation
  invalidate history. Unmatched controls such as Tab and Return also clear it.
- No cursor-position guessing or editor-text reads were introduced. Input origin,
  target, generation, cancellation, clipboard ownership and strict P0 policies remain.

## Verification

The original immediate-match and typo-after-Space tests both failed before the
matcher change and passed after it. A Ctrl/Alt/Win Backspace ordering test failed
before shortcut rejection was moved before character-edit processing. Preferences
tests failed with the activation selector missing, then passed with both options.

An independent read-only review found one Important issue: unmatched Tab could
leave the previous browser field's abbreviation in history. A reproducer failed
before the fix, then the full suite passed after unmatched controls were reset.
The reviewer reported no Critical or Minor findings; the final fix was verified
by tests, not a second review approval.

`bun run check:all` passed with **154 frontend tests, 188 Rust library tests,
12 tooling tests and one fixture-server test**, including TypeScript, ESLint,
AST checks, Prettier, rustfmt and all-target Clippy. Bindings were regenerated.
Ordinary checks did not install hooks or send input.

An earlier interrupted concurrent frontend run reported the pre-existing
ProbePanel “starts stopped” assertion before cancellation. Both later full serial
runs passed; no timeout or product behavior was changed to hide it.

## Native check

The explicit `product_workflow_trial --immediate-correction` helper asks the user
to type `prefix ;helllo`, press Backspace twice, then type `o` with no final Space.
It requires the exact full result `prefix Texpio Windows paste works.`, keeping
the preceding text to detect over-deletion. It allows three minutes and closes
on settled exact output, Escape, or timeout. The helper generates no typing.

The user replied “Ready now” after the isolated debug build completed. The visible
fixture then reported **exact=true**, receiver exit 0, one submitted job, zero
cancellations, no insertion error, and one usage record. The full output retained
`prefix ` and had no trailing Space. Capture counters reported 52 callbacks,
14 own output events ignored, 38 accepted software events, zero lower-integrity
events, one match and one accepted job; maximum callback time was 111 microseconds.
This is a successful user-typed compatibility check, not strict physical-input
qualification. The strict P0 campaign remains unchanged.

The harness's overall process returned 1 **after** the successful assertions:
restoring the disposable profile's original clipboard-hotkey preference returned
`shortcutUnavailable`. Its library was restored first; cleanup then stopped the
engine, disconnected CDP and ended both owned processes. A subsequent read-only
check confirmed zero Texpio/helper processes and zero port-9225 listeners. The
isolated profile remains stopped with clipboard hotkey disabled. This cleanup
conflict was not counted as an insertion failure or hidden as a fully clean harness
exit. The normal user profile/library was not changed by this trial.

## Build and preview

The locked isolated debug build and `bun run tauri build -- --locked` both passed.
The latter produced the optimized executable, MSI and NSIS installer. No installer
was run and no signed release was published.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `src-tauri/target/release/texpio.exe` | 12045312 | `8BBA23230E1AD929EE1D435A58BA2439FA5B4FF5EF184EFFE37D6287BD3DFF6F` |
| `src-tauri/target/release/bundle/msi/Texpio_0.1.0_x64_en-US.msi` | 5820416 | `0B3CC491C208531202E93CDEE7290FF43363226CC143F6A1B6A9230EA042B742` |
| `src-tauri/target/release/bundle/nsis/Texpio_0.1.0_x64-setup.exe` | 4243757 | `1801F8DA6A6C2804FDA96E12D27C6EF0BC5D561B7184B997EF77D616008EA2CA` |

The normal optimized preview was reopened from the release path (observed PID
22396, window title Texpio). Its existing software-input compatibility setting
remains enabled and enable-on-launch remains false. The absent activation field
loads as immediate through the tested settings migration. No normal-library or
preference write was needed to make this change. No remote-debugging arguments
were supplied to the preview; always reobserve live process identity later.

Ignored local logs are under `.superpowers/sdd/2026-10-25-immediate-expansion/`.
See the [design](../superpowers/specs/2026-10-25-immediate-expansion-design.md) and
[plan](../superpowers/plans/2026-10-25-immediate-expansion.md).
