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

## Continuation verification on the current PC

The continuation checkout and remote `main` both matched `d2f6d097b779a4f665f5f04c551a84013b42b81c`, with a clean working tree before this work. This PC runs Windows 11 Home Single Language, build 26300, x64, and WebView2 155.0.4283.62. Local Node 24.18.0, Bun 1.4.4 and Rust 1.99.1/rustfmt/Clippy were installed under ignored `.tools`; Visual Studio Build Tools 17.14.37806.7 and Windows SDK 10.0.26100.0 were installed successfully. The frozen Bun install retained the committed lockfile. Rust archives downloaded through a mirror were verified against the official distribution's SHA-256 checksums. No global Node/Bun/Rust version was changed.

A fresh, independent, read-only review covered `ee0ee96..d2f6d09`. It confirmed a Resume publication race and the absence of native Quit draft protection. The race regression first failed because an activation admitted before Stop could still arm afterward. A short publication lock and cancellation generation now reject that activation before worker initialization, check session publication, and prevent an overtaken Pause from restoring its session. Save/import invalidate through the same Stop boundary. These ordinary regressions activate only the in-memory gate; they install no hooks and send no input.

Tray Quit now stops observation, restores the main window and requests its acknowledgement. The main window uses the existing discard/keep-editing dialog before calling a main-only typed Quit command. Update installation and explicit restart also await the draft guard. Missing/late renderer listeners leave the application open. The new frontend Quit tests failed first for the absent listener; the restart cancellation test failed first because restart proceeded. All 20 affected library/updater tests then passed. A full frontend run during prerequisite installation had one loading-query timeout (137 passed, one failed); the final complete check result is recorded separately below.

Actual native-app checks used a separately built application identifier, `io.github.texpio.texpio.acceptance`, so test content did not enter the normal product library. A bounded, loopback-only WebView2 test connection exercised the real React application, typed IPC and SQLite storage. No native services were mocked and the engine was never armed. The following passed:

- Create/save a literal body containing an accent, emoji, Devanagari and a newline.
- Duplicate into a disabled snippet; trash and restore it with its body intact.
- Deliver a Quit request through the real native event plugin, cancel the dialog and retain the unsaved draft.
- Confirm discard, observe process exit, relaunch, and load the acknowledged body from SQLite without the discarded title edit.

The first harness attempt stopped on an incorrect test selector (`Active` instead of the visible `Library` tab); the corrected run passed. Both attempts disconnected their WebView2 connection and stopped only their owned acceptance process. A screenshot was inspected locally. This verifies the native event/acknowledgement path, not a physical tray-menu gesture.

The review's proposed Important draft-recovery finding was withdrawn after checking that Duplicate preserves the edited content with a new ID and current revision. The stale security guide was corrected to describe the picker's intentional explicit text-write capability and absence of clipboard-read permission. A minor wording follow-up remains for the Duplicate recovery path's discard confirmation.

Further actual-app checks passed using the same isolated identifier and real native services:

- Main-window Copy and picker Copy placed the exact synthetic body on the system clipboard; comparisons did not log clipboard text.
- Picker search received focus; Copy hid its native window; reopening and Escape hid it again.
- Closing the main window hid it while the process remained alive. A second launch exited and restored the original window.

The first picker harness used DOM focus as a proxy for visibility and timed out. The corrected check used read-only Win32 visibility for the exact owned process/window; it passed. Each connection and owned acceptance process ended. No input observer was armed. Selected-file dialogs, physical tray gestures and the remaining native acceptance matrix are still unverified.

### Native insertion isolation

The explicit helper now has caption-restricted read-only inspection and optional preparation of the exact synthetic fixture's focus/collapsed end caret. Preparation does not change text and does not bypass the real pipeline's suffix/caret/password/composition/integrity/layout checks. Unknown modes refuse. All native trials used bounded parent deadlines and stopped without repair or replay.

| Fresh disposable trial | Submission | Observation |
| --- | --- | --- |
| Greeting, TextExpander enabled | 52/52 | Uncertain; 19 UTF-16 units, first differing unit at offset 11, trailing space retained. A read-only capture of the exact owned fixture showed the `Tex` portion replaced by `xxx` |
| Accent, TextExpander enabled | 42/42 | Uncertain; 13 UTF-16 units, first differing unit at offset 0, trailing space retained |
| Greeting, after user shut TextExpander down | 52/52 | Uncertain; 19 UTF-16 units, first differing unit at offset 0 |

Zero TextExpander processes were verified before the last comparison. It was not restarted. **TextExpander alone does not explain the failure.** Other running input software was left unchanged; no cause is assigned to it. Notepad version was 11.2607.14.0. Failed synthetic documents were preserved rather than reset or replayed. Ordinary global input remained stopped.

A separate explicit [owned receiver](../../src-tauri/examples/unicode_receiver_trial.rs) created its own Win32 EDIT control with the same fixed trigger and submitted the same combined deletion/output sequence. The final run passed: **52/52 submitted, 19 VK_PACKET key-down messages, 19 exact Unicode WM_CHAR units, and 19 exact final control units**. Its window was destroyed and helper exited. An initial harness caret error retained the original seven units despite exact character delivery; correcting the owned caret to 7/7 produced the full replacement result. The final helper also checks that selection before submission and bounds captured message units. It installs no global hooks and touches no foreign document.

This narrows the failure toward Notepad/control-specific processing. It does not prove a precise root cause, validate all Win32 controls or complete physical input qualification. Product Unicode dispatch remains unchanged. **Automatic Notepad expansion is still failing qualification; physical automatic trials remain 0/300.**

The [Espanso/Snipt comparison](cross-platform-expansion-source-comparison.md) and [Ampello audit](ampello-source-audit.md) record pinned source findings and specific next experimental variables. Paced/separated insertion and clipboard strategies remain alternatives requiring evidence, not fixes silently enabled by this research.

### Final continuation checks and bundles

The fresh final `bun run check:all` passed: **138 frontend tests across 27 files, 111 Rust tests, 11 tooling tests, one loopback fixture test and six AST groups**; TypeScript, ESLint, Prettier, rustfmt and strict all-targets Clippy passed. This compiles the explicit native diagnostics without running them. `git diff --check` and 88 local links across seven changed documents also passed.

The isolated debug no-bundle build and normal-identifier optimized Windows build passed. `bun run tauri build -- --locked` finished both MSI and NSIS bundles. A packaging download stall was resolved by caching the exact NSIS 3.13 archive and Tauri NSIS utility plugin 0.5.3 after checking the hashes from the installed Tauri CLI's pinned source. No installer was executed and no signing, remote CI or distribution qualification is claimed.

| Local artifact under `src-tauri/target/release/` | Bytes | SHA-256 |
| --- | --- | --- |
| `texpio.exe` | 10,897,408 | `F8E2296D2D4B8E35110D07441F44E9393A12C43D30EE68CC7F2D8BA819671064` |
| `bundle/msi/Texpio_0.1.0_x64_en-US.msi` | 5,320,704 | `1608E59544D578F4BD408EBB36DAFFDC2500EE85418C2100B50CFCAA6D385696` |
| `bundle/nsis/Texpio_0.1.0_x64-setup.exe` | 3,903,408 | `6C3CA3633BBC2023B4949141CE71A20886041F26462482C784A179954F9CFE0B` |

These ignored binaries stay on this PC. Rebuild on another checkout; compilation and packaging do not establish successful automatic text expansion.
