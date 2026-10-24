# Windows offline MVP acceptance record

24 October 2026 · Development build; interactive qualification pending

## Scope and provenance

The user selected an independent Rust engine modeled on [Espanso's architecture](espanso-source-audit.md), then requested autonomous construction of a working MVP while offline. This work follows the [MVP design](../superpowers/specs/2026-10-24-offline-mvp-design.md) and [plan](../superpowers/plans/2026-10-24-offline-mvp.md). Execution started from `ee0ee96028d9d791b96c18df55262dac361d231b` in the approved current checkout. No Espanso runtime code was copied and no license was selected.

## Implemented behavior

| Area | Behavior | Implementation evidence |
| --- | --- | --- |
| Library | Rust-owned bundled SQLite, schema version 1, transactions, revision conflicts, first-run synthetic samples | [Library store](../../src-tauri/src/library/store.rs) |
| Validation | Canonical IDs, folder references, active-trigger uniqueness, count/body/JSON bounds, control-character refusal | [Validation](../../src-tauri/src/library/validation.rs) |
| Recovery | Immutable JSON snapshots before changed saves/imports; backup failure refuses the change; trash/restore in the UI | [Store](../../src-tauri/src/library/store.rs), [library UI](../../src/features/library/LibraryView.tsx) |
| Editor | Create/edit/duplicate, enabled state, folders, title/trigger/body search, preview, explicit save, conflict/error states, unsaved-draft guards | [Library UI](../../src/features/library/LibraryView.tsx) |
| Interchange | Versioned JSON export, selected-file import preview, bounded additive import, identical-record skipping, all-or-nothing conflict refusal | [Native facade](../../src-tauri/src/library_commands.rs), [frontend service](../../src/services/library.ts) |
| Shared engine | Immutable native-owned rules shared by recognition and effect workers; candidate resolves only against that snapshot | [Rules](../../src-tauri/src/engine/rules.rs), [worker](../../src-tauri/src/engine/windows/worker.rs) |
| Product target policy | Ordinary Notepad editor class; original fixture filename restrictions remain in Developer/P0 | [Profiles](../../src-tauri/src/engine/windows/profiles.rs) |
| Native controls | Explicit start/pause/stop; save/import invalidates active and paused snapshots before committing; no startup hooks | [Handle](../../src-tauri/src/engine/handle.rs), [facade](../../src-tauri/src/library_commands.rs) |
| Picker | Search snippets and explicitly Copy; opening/closing does not send text to another app | [Picker](../../src/components/quick-pane/QuickPaneApp.tsx) |
| Privacy | Content remains local; no raw keyboard or target tokens in IPC; no automatic clipboard fallback; errors redact content/paths | Native facade, typed bindings and capability files |

Local data is unencrypted under the application's app-data directory: `library.sqlite3` and `backups/library-r<revision>.json`, separate from preferences. Do not place a live database in a cloud-sync directory. JSON exports/backups contain snippet bodies and should be stored accordingly. Import preserves existing content rather than overwriting differing records; it is not a destructive restore button. A corrupt-database replacement/recovery UI is not implemented.

Bodies are literal plain text, up to 16,384 Unicode scalars. Triggers are a semicolon plus 1–31 lowercase ASCII letters; Space activates expansion and one trailing space is retained. Date/time, clipboard and cursor tokens are not interpreted in this milestone. Copy is an explicit request that replaces clipboard text. Automatic clipboard insertion remains disabled.

## Automated evidence

- Storage: 19 real temporary-database tests passed, including reopen/restart, concurrent store handles, an actual SQLite abort during import, stale revisions, encoded-size limits, backup failure and export/import round trips. These are not empirical sudden-power-loss tests.
- Dynamic core: tests passed for independently owned rule IDs/bodies, invalid and duplicate rules, strict fixture separation, native product-profile predicates and old library-session invalidation.
- Unicode regression: the old fixed-fixture batch limit rejected a maximum valid saved body. The new bound test failed first, then passed with a 32,769 UTF-16-unit output bound and forbidden control-packet validation. This is construction evidence, not observed insertion of a maximum-sized body into an app.
- Integrated native unit suite: 108 tests passed before the final Unicode-limit regression was added; the four focused Unicode tests then passed. Final complete-suite/build results are recorded below after execution.
- UI helper's complete frontend run: 133 tests across 26 files passed, with TypeScript, scoped ESLint/Prettier and production frontend build also passing. The primary agent reruns the integrated project gates before handoff.

Native atomic-file tests need execution outside the restricted sandbox on this host: sandboxed `MoveFileEx` calls return access denied, while unchanged escalated tests pass. This is an execution-environment boundary, not evidence that failed writes are accepted.

## Interactive/native evidence and remaining gates

The initial Computer Use request timed out at app approval, before input. The user subsequently restarted ChatGPT; the active runtime then reported Full access with approval policy `never`. Native fixture checks resumed with Computer Use. The later user Escape interruption ended that check; no further UI automation was attempted. The user requested a commit/push checkpoint and handoff rather than continued interactive testing.

A synthetic document, `.superpowers/sdd/2026-10-24-offline-mvp/texpio-mvp-test.txt`, was opened through the CLI. The explicit [native trial helper](../../src-tauri/examples/mvp_native_trial.rs) builds, installs no hooks, accepts only that exact disposable Notepad caption, and uses the real target gate/direct insertion/verification pipeline. It is not an ordinary test/CI entry point and does not enable injected input in the normal observer.

Observed synthetic results, in order:

1. Two earlier attempts refused with `UnsupportedProfile` before mutation; that stage was not initially isolated.
2. After restart, the observed disposable editor held `;hello ` with a collapsed caret after its seven characters. The instrumented helper accepted the foreground caption, context and native control, then returned **`Uncertain { submitted: true }`**. The later screenshot showed only `;` (one character), without the intended `Hello from Texpio! ` output. The engine did not repair or replay. The exact submission count was not recorded in that build, so partial submission versus subsequent delivery/consumption failure remains unresolved.
3. Submission-count and verification-reason diagnostics were added to the helper. During the next setup, Computer Use text entry did not replace the selected synthetic semicolon; the helper refused **Selection** before mutation. That attempt did not reach the new submission-count diagnostic.

TextExpander was concurrently running on this PC. Its File menu showed **Disable Expansion**. One click on that control was issued for a prospective comparison, then the user stopped Computer Use with Escape before the new state or comparison could be verified. **Interference is a hypothesis, not established causation. TextExpander's expansion state was not restored or verified after interruption; the original PC user should check File → Enable Expansion if desired.** No user snippet/group content was edited. The failed trial must be diagnosed before claiming working native expansion or starting a larger campaign. See the [portable handoff](../handoffs/2026-10-24-offline-mvp.md).

The [earlier P0 Notepad read-only result](windows-expansion-probe.md) remains valid only for its exact recorded fixture/build. This MVP does not convert it into insertion, ordinary-document, physical-observer, picker-target or lifecycle acceptance. **Physical automatic trials remain 0/300.**

Pending interactive checks: editor create/save/restart, file dialogs and selected-file scopes, explicit clipboard Copy, picker focus/Escape, tray close/Open/Pause/Stop/Quit, ordinary Notepad exact output and sentinel preservation, same-window tabs/caret changes, active composition and unknown controls, layout changes, large/multiline Unicode output, repeated arm/stop and fault recovery. No wrong-target or unrelated-deletion rate is claimed when native trials have not run.

Broader Windows apps, macOS, Linux/X11 and Linux/Wayland need their own positive capabilities and native campaigns. The original larger P1 correctness/resource/accessibility/installer/signing gates remain open. This build is a development MVP, not a verified public release.

## Final integration results

The earlier complete check run passed 133 frontend tests, 109 Rust tests, 11 tooling tests, one loopback fixture test and six ast-grep groups, plus typecheck/lint/format/Clippy. Two subsequent EngineControls regressions passed in a 20-test focused library run: retrying Stop in Fault and preventing Start with no eligible snippets.

The fresh handoff checkpoint run of `bun run check:all` passed **135 frontend tests across 27 files, 109 Rust tests, 11 tooling tests, one loopback fixture test and six ast-grep groups**, with TypeScript/ESLint/AST lint/Prettier/rustfmt/strict all-targets Clippy also passing. This includes the final diagnostic helper source. No ordinary check armed native input.

The Windows debug no-bundle build passed after those UI changes: frontend build 11.04 seconds; Rust development build 33.92 seconds. It produced `src-tauri/target/debug/texpio.exe`. No MVP release/installer build, signed distribution, actual new-app launch acceptance or whole-change independent review was completed. The older release executable belongs to the template checkpoint and must not be presented as the MVP. Final checkpoint checks and commit identity are recorded in the handoff after execution.
