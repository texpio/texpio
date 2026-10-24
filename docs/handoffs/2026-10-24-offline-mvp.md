# Texpio offline MVP handoff

**25 October layout update:** the app now uses a folder/list/editor workspace and
a dedicated, categorized Preferences page. The right sidebar/toggle and Developer
route are removed. Read the [workspace acceptance record](../research/library-workspace-acceptance.md)
before following older navigation instructions. Full checks now pass 162 frontend
and 188 Rust tests; the native expansion engine is unchanged by this UI work.

**25 October behavior update:** immediate expansion and Backspace correction are
implemented and passed the user-typed owned fixture. Read the
[immediate expansion acceptance record](../research/immediate-expansion-acceptance.md)
before following the older delimiter-only instructions below. The user confirmed
Backspace at the end as their correction method. Full checks now pass 154 frontend
and 188 Rust tests; cursor movement remains a history reset.

24 October 2026 · Checkpoint for continuation on another PC

**Continuation update:** the final section records work after `d2f6d09`: toolchain setup, whole-change review/fixes, actual-app checks, new native isolation evidence and Espanso/Snipt/Ampello source research. Read that section before treating the older checkpoint's pending items as current.

**Windows v1 continuation:** the user subsequently prioritized Ampello's Windows feature families in Texpio's existing UI. Read the [Windows v1 acceptance record](../research/windows-v1-acceptance.md), [review/fix record](../research/windows-v1-review.md) and [parity tracker](../research/windows-v1-parity.md). Attachments, backup formats, library routing, favorites/editor tools and desktop settings are implemented. Manual trigger expansion now passes on this PC with explicit software-input compatibility enabled: the observed cause was Windows classifying the user's events as injected. Do not repeat completed feature work or report the older insertion failure as the current blocker. The user's availability rule is in root `AGENTS.md`: silence means offline; wait for their reply before a check needing participation.

The current integrated check passed 152 frontend and 174 Rust tests plus tooling/static gates. Optimized executable/MSI/NSIS builds passed. The normal preview was configured through native commands with `allowSoftwareInput: true` and `enabled: false`; its library was not edited. It is open from `src-tauri/target/release/texpio.exe`, without remote-debugging arguments or a listening test port. Verify the live process identity before any future action; ask the user to save drafts and use tray → Quit before replacing its locked executable. Compatibility remains off by default on another installation and must be selected for software keyboard input. Strict P0 remains 0/300. Native adjunct checks still listed in the acceptance record remain unqualified; a working prototype fixture does not complete them.

## Objective and authorization

Continue building a working Windows offline MVP of Texpio, an open-source local-first text expander. The user chose an **independent Rust engine modeled on Espanso's architecture**, retaining the existing Tauri/React/shadcn shell. They requested autonomous implementation while offline, authorized app testing, and subsequently explicitly requested committing/pushing all current changes and a handoff for another PC. Native execution in the current checkout and commit/push are authorized. Do not repeat completed naming, starter, architecture-choice or plan-approval conversations. Work remains incomplete; this is a checkpoint rather than release acceptance.

Repository: [texpio/texpio](https://github.com/texpio/texpio), branch `main`. MVP execution base: `ee0ee96028d9d791b96c18df55262dac361d231b`. Prior checkpoints: `8096a7d` UIA composition fix; `51028c0` template migration evidence; template runtime `1aae8e1`. Locate this handoff's exact checkpoint with `git log -1 --format=fuller -- docs/handoffs/2026-10-24-offline-mvp.md` after pulling. The current source is the authority; don't reset to those older checkpoints.

## Read first

1. Root [AGENTS.md](../../AGENTS.md), [.agents/README.md](../../.agents/README.md), [architecture](../developer/architecture-guide.md), [testing](../developer/testing.md), [security](../SECURITY.md).
2. [MVP design](../superpowers/specs/2026-10-24-offline-mvp-design.md) and [implementation plan](../superpowers/plans/2026-10-24-offline-mvp.md). The offline/autonomous request removed further approval waits for this work; preserve actual safety and evidence boundaries.
3. [MVP acceptance](../research/offline-mvp-acceptance.md), including the failed native trial and competing-expander uncertainty.
4. [Espanso audit](../research/espanso-source-audit.md), original [P0 design](../superpowers/specs/2026-10-23-windows-expansion-probe-design.md), [P0 plan](../superpowers/plans/2026-10-23-windows-expansion-probe.md), [P0 evidence](../research/windows-expansion-probe.md), and [TASKS.md](../../TASKS.md).

## Implemented in this checkpoint

- SQLite library in `src-tauri/src/library/{mod,store,validation,tests}.rs`: transactions, schema version 1, expected revisions, canonical IDs, active-trigger uniqueness, bounded content, initial synthetic samples and backups before changed saves/imports.
- `src-tauri/src/library_commands.rs`: typed asynchronous load/save/export/import-preview/import/start; save/import serialize against library start and invalidate observation before content changes. Registration, generated bindings and main/picker capabilities are included.
- Library Home UI in `src/features/library/`: folders, title/trigger/body search, create/edit/duplicate, enabled state, trash/restore, preview, explicit saves, unsaved-draft guards, stale revisions/errors, import preview and export. Data hooks/facades are in `src/services/library.ts`; strings are in en/fr/ar.
- Searchable quick pane with explicit Copy; native window is now 600×420. No captured-target picker insertion is implemented.
- `src-tauri/src/engine/rules.rs`: validated owned immutable RuleSet, shared by observer and native worker. The old fixture-only couplings were replaced; Developer creates a separate fixed snapshot.
- A separate internal `TargetPolicy::NotepadEditor` accepts ordinary Notepad editor controls without changing the original P0 exact fixture profiles. All suffix/caret/password/composition/integrity/context checks remain.
- Library Start/Pause/Stop; library sessions report `profile: null`, while P0 fixtures retain their named profile. Library epochs prevent old paused rules from being resumed after a save/import. Fault Stop remains enabled; Start requires Stopped and at least one enabled, non-trash saved snippet.
- Unicode input plans support at most 32,769 UTF-16 output units and reject control packets except TAB/LF/CR; saved bodies are capped at 16,384 Unicode scalars. **This construction limit is not native delivery qualification.**

The management code is implemented and automated checks pass as recorded below. The main application UI, clipboard/file dialogs and native engine still need actual acceptance. No release/public readiness claim is justified.

## Critical unresolved native failure

Use the [explicit synthetic helper](../../src-tauri/examples/mvp_native_trial.rs), not a normal unit test. It requires the exact disposable Notepad caption `texpio-mvp-test.txt - Notepad` (optional leading `*`), installs no hooks, recognizes a synthetic normalized trigger, then runs the real prepare/revalidate/dispatch/verify pipeline. It waits five seconds before capturing the foreground target. It refuses unrelated documents. It must remain separate from physical observer evidence.

On the original Windows PC:

- Notepad fixture was observed containing exactly `;hello `, caret after the trailing space (line 1, column 8).
- Earlier attempts refused UnsupportedProfile before mutation, with no exact stage initially captured.
- After Full access was active, the helper logged:

```text
Stage: foreground stamp captured.
Stage: disposable-document caption accepted.
Stage: context initialized.
Stage: native control accepted.
Synthetic result: Uncertain { submitted: true }. No repair or retry attempted.
```

- The subsequent screenshot showed only `;` (one character), with no intended `Hello from Texpio! ` output. The native effect failed. The exact `SendInput` expected/submitted counts were not yet logged, so partial submission and downstream delivery/consumption are not distinguished.
- The helper now prints `Native submission: expected=..., submitted=...` and a verification refusal reason. A subsequent setup attempt using Computer Use `type_text(';hello ')` did not replace the selected semicolon. The next helper refused Selection before mutation, so the new count diagnostic has not been reached.
- TextExpander was running concurrently. Its File menu showed Disable Expansion. One click on Disable Expansion was issued to prepare a comparison, then the user pressed Escape before state/comparison/restore were verified. **Do not claim TextExpander caused the failure.** Its state on the original PC may now be paused; the user should check File → Enable Expansion if desired. No snippet/group content was edited.
- Computer Use was stopped by the user's physical Escape key. No subsequent UI automation was performed. On a new PC and explicit continuation, start fresh selection/state; never reuse this PC's window handles or UI indexes.

**Next priority:** reproduce on a clean disposable fixture and isolate the failing layer using the new counts, a bounded output observation, and a controlled comparison with competing expanders paused/absent. Keep original state and restore any temporarily changed app setting. Don't change batching, add sleeps, relax context checks, or blame a competitor without evidence. No blind backspace repair or replay after partial/unobserved insertion.

Current native code: `engine/windows/unicode.rs` builds a single tagged batch of verified trigger+delimiter Backspaces followed by Unicode key-down/up pairs. `SendInput` return count is checked by `runtime.rs`; unobserved/partial results close the gate. `context.rs` uses UIA TextEdit's explicit idle-composition result; do not reintroduce foreign-thread IMM guesses. The Notepad policy requires notepad.exe, Win32, RichEditD2DPT, Text editor, Document plus the other native checks. Generic policy removes the fixture filename requirement only.

P0 physical automatic trials remain **0/300**. Pure tests, synthetic native trials and Computer Use input do not count as physical input; the ordinary observer intentionally rejects unexpected injected events. No native hook/input runs in ordinary tests or CI. Do not enable injected input in normal builds to manufacture passing results.

## Remaining work, in order

1. Set up the pinned toolchain on the new PC; read current git status and retain all existing work. Establish actual Windows/app/layout versions and native permissions.
2. Fix and qualify the native failure above with regression tests and bounded diagnostics. Keep wrong-target/unrelated-deletion stop conditions. The old P0 guard/matcher and Notepad read-only success don't prove insertion.
3. Exercise the actual Tauri library app: create/edit/save/restart, folders, duplicate/trash/restore, conflicts, draft guards, selected-file import/export, explicit Copy and picker focus/Escape. Use synthetic data only. Verify capability scopes rather than widening filesystem access preemptively.
4. Test Start/Pause/Stop/Fault recovery, save/import while active/paused, tray close/Open/Quit/single instance, repeated sessions, layout/composition, caret/selection/tab/focus changes and unsupported/elevated/password controls.
5. Complete native campaigns and support decisions. Keep macOS, X11 and Wayland as independently qualified adapters. The user chose Windows first, macOS next.
6. Run one fresh whole-change independent review against the MVP execution base and design. Only storage received a focused source review; **the MVP whole-change review has not happened**. Review concurrency, stale drafts, transaction/import rollback, old snapshot rearming, native cancellation/outcomes, clipboard/file scopes and multi-window lifecycle. Fix important findings with regressions, rerun full checks/builds, then commit/push.
7. Build and verify the intended Windows executable/installer. Signing, remote CI, packaged installation and detailed accessibility/performance remain open. The old release exe is from the template checkpoint; rebuild it before calling it the MVP.

No backend, accounts, billing or sync service should be built here; the user has an existing Laravel/PostgreSQL project for future service infrastructure. Rich text/images/scripts, interpreted date/time/clipboard/cursor tokens, automatic clipboard insertion, captured-target picker insertion and corrupt-database restore UI remain deferred. Bodies are literal plain text. Preserve the design's current limited support promise rather than silently replacing it with history-only deletion.

## Toolchain and reproduction

Use Windows x64/MSVC, Node **24.18.0**, Bun **1.4.4**, Rust **1.99.1** plus rustfmt/Clippy, C++ desktop Build Tools/Windows SDK and WebView2. See [setup](../development.md). Bun/Rust/lockfiles are pinned; install those versions rather than silently upgrading. On the original PC `.tools/bun/1.4.4/package/bin/bun.exe` is an ignored local install; that path will not exist after a fresh clone. `.agents` is committed, but user-global dependency skills may need installing from the provenance in AGENTS.md.

```powershell
bun install --frozen-lockfile
bun run check:all
bun run build
bun run tauri build --debug --no-bundle -- --locked
```

Use `bun run test`, not Bun's separate `bun test` runner. Regenerate changed IPC with `bun run rust:bindings`. Keep native calls off the webview, use TanStack Query for native data and Zustand for layout only.

For the native helper, build it explicitly with `cargo build --locked --manifest-path src-tauri/Cargo.toml --example mvp_native_trial`. Set up the exact owned fixture, observe the correct focused editable surface/caret, then run only the selected fixed case. Launch background helpers with `Start-Process -WindowStyle Hidden` so a console cannot steal focus; redirect output and enforce a ~12-second parent deadline, terminating only that exact owned helper if needed. `hello`, `address`, `accent`, `devanagari`, `emoji`, `multiline`, `long` and `custom` are fixed synthetic cases. Never invoke against private documents or expand the helper caption allowlist casually.

The original host's sandbox denied atomic file replacement/loopback/GUI calls, causing prompts. After restart the runtime became `danger-full-access` / approval `never`; normal execution worked. Use the new host's actual permission profile. Do not request `require_escalated` under a `never` profile, and do not promise that a chat message alone changes tool permissions. Computer Use can have its own app-access controls.

## Data, evidence and portability

SQLite lives under app-data as `library.sqlite3`, with `backups/library-r<revision>.json`. JSON exports/backups contain unencrypted snippet bodies. Imports are additive: identical IDs skip, differing existing IDs/active-trigger conflicts refuse atomically. Imports are not destructive rollback of current records. Backups can restore into a deliberately empty valid library, but there is no corrupt-database recovery UI. Do not move a live DB into a cloud sync folder.

The original `.superpowers/sdd/2026-10-24-offline-mvp/` workspace holds detailed RED/GREEN logs, helper outputs and its progress ledger. The earlier P0 workspace remains separately under `.superpowers/sdd/2026-10-23-windows-expansion-probe/`. Both are ignored and **will not transfer through Git**. This handoff and acceptance doc preserve the meaningful portable evidence; rebuild synthetic fixtures/logs on the new PC. `.tools`, node_modules, target/dist, binaries, local user data and screenshots also do not transfer. No credentials are needed from the old PC to continue development.

Espanso reference checkout is ignored too. If needed, inspect the pinned audited commit `a87d1723f47a59e38d370e2f0c636d66d568b8f6` (v2.4.1 tag) via the audit's source links. Independent architecture was selected; don't port GPL runtime code or select Texpio's product license incidentally.

## Calls made during execution

| Call | Reason | Cost if wrong |
| --- | --- | --- |
| Keep strict verified-target insertion and initial Notepad policy | Preserve the existing target guarantee while adding a useful library | Automatic app compatibility is narrow until additional adapters are qualified |
| Parallel helpers owned storage/UI; primary owned engine/integration | Make autonomous progress with explicit interfaces | Coordination risk requires integrated tests and the still-pending whole-change review |

Literal-only tokens, additive imports and explicit-Copy picker were explicit MVP scope decisions, not completed future workflow features. Native body size bounds are tested in construction, not qualified at maximum length. No final-review minors were deferred because that review has not occurred.

## Checkpoint verification

The original full check run passed 133 frontend tests, 109 Rust tests, 11 tooling tests, one loopback fixture test and six ast-grep groups. Two later EngineControls regressions passed in a 20-test focused library run. Windows debug no-bundle build passed (11.04-second frontend; 33.92-second Rust build). The MVP binary was not launched/accepted, and native insertion failed as above.

A fresh full checkpoint `bun run check:all` passed before commit/push: **135 frontend tests across 27 files; 109 Rust tests; 11 tooling tests; one loopback fixture test; six ast-grep groups**. TypeScript, ESLint, AST lint, Prettier on its configured files, rustfmt and strict all-targets Clippy passed. The diagnostic helper is compiled by those checks; its synthetic native trials remain the failures described above. `git diff --check` and 107 local links across nine changed documents passed; the new handoff index link was checked afterward.

The local debug executable is 30,489,600 bytes, SHA256 `F3B3987072A64488601D088D9A10F42D1EA2E215739EBBA0055496F6D6742F89`. It is ignored and not pushed. New checkouts must rebuild it. This digest does not turn a compiled app into native acceptance. Remote CI and whole-change review remain unverified/pending.

## Continuation after d2f6d09

The new PC checkout and remote matched `d2f6d097b779a4f665f5f04c551a84013b42b81c` before changes. Windows 11 Home Single Language build 26300, x64; Notepad 11.2607.14.0; WebView2 155.0.4283.62. Node 24.18.0, Bun 1.4.4 and Rust 1.99.1/rustfmt/Clippy are installed under ignored `.tools`; Visual Studio Build Tools 17.14.37806.7 and SDK 10.0.26100.0 installed successfully. The local `.tools/texpio-env.ps1` sets the process-local PATH and local Rust homes. Dot-source it before commands on this PC; recreate equivalent configuration on another PC. Frozen installation passed without changing dependency versions or lockfiles. Rust mirror archives matched official SHA-256 sidecars.

### Review and application work already completed

An independent read-only whole-change review covered `ee0ee96..d2f6d09`. Important findings were fixed with demonstrated failing-then-passing regressions:

- A cancellation generation and short publication lock prevent a Resume admitted before Stop/library invalidation from arming or publishing a stale session. Provider calls, worker initialization/shutdown and native dispatch remain outside the short lock.
- Native tray Quit stops observation and requests renderer acknowledgement; the existing draft dialog gates a main-only typed Quit command. Update installation and explicit restart await the same transient draft guard. Missing renderer acknowledgement leaves the app alive.

The proposed Important conflict-recovery finding was withdrawn after Duplicate was verified to preserve the draft with a new ID/current revision. One minor wording follow-up remains for its discard confirmation. The security guide's stale clipboard capability description was corrected. Do not repeat the completed whole-change review solely because the older section says it is pending.

A debug acceptance build used identifier `io.github.texpio.texpio.acceptance`, isolating its real SQLite data from the normal product library. Bounded loopback-only WebView2 checks, real native IPC and read-only native visibility checks passed Unicode save/restart; disabled duplicate; trash/restore; Quit cancel preserving a draft; confirmed discard/process exit; main and picker clipboard Copy; picker search focus/Copy/Escape hiding; main close-to-hide; and second-launch restoration. All owned test connections/processes ended and the engine was never armed. Incorrect initial selectors/visibility predicates were corrected in the harness, not counted as product defects. Physical tray gestures and selected-file dialog/scope checks remain open.

### Current native blocker and strongest evidence

The native helper now supports `--inspect-only` and `--prepare-fixture`. Preparation requires the exact caption and entire fixed synthetic input, selects one visible strict-profile editor, focuses it and collapses its caret; it changes no text. The real pipeline then retains every target/caret/suffix/composition/password/integrity/layout check. All helper runs have bounded parent deadlines, no hooks and no repair/replay.

Fresh Notepad trials fully submitted input but failed exact output: greeting 52/52 events, accent 42/42. The first greeting had 19 UTF-16 units, first difference at offset 11; the exact owned-window capture showed `Tex` changed to `xxx`. After the user said they shut TextExpander down, zero TextExpander processes were verified and a **fresh** greeting still failed after 52/52 submission, first difference at offset 0. TextExpander was not restarted. **TextExpander alone does not explain this failure.** Other running software was not changed or blamed. Preserve failed fixtures; do not replay them.

The new explicit `unicode_receiver_trial --owned-receiver` creates only its own standard Win32 EDIT and sends the same 52-event combined deletion/output sequence. Its final run passed: 19 exact WM_CHAR units and 19 exact final control units, no cancellation; its window closed and process exited. An initial fixture caret error was fixed to 7/7 and explicitly checked. This isolates successful basic Windows delivery but does not establish Notepad's root cause or universal app support. Product `windows/unicode.rs` was not changed during this diagnosis. Notepad automatic expansion remains failing/unqualified, and physical trials remain **0/300**.

### Requested source research

Read the committed [Espanso/Snipt comparison](../research/cross-platform-expansion-source-comparison.md) and [Ampello audit](../research/ampello-source-audit.md). Ignored source references are pinned: Espanso v2.4.1 `a87d1723f47a59e38d370e2f0c636d66d568b8f6`; development HEAD `fb90347ec0bdfceb0ea5cf1a1b36280fa2961463`; Snipt `dc5476049d3d0470c2c3685da3db6e62a71524bf`; Ampello `c6ba8ea4a227b063df945597d1223f43f6e2f074`. Snipt's Enigo 0.3.0/rdev 0.5.3 published source checksums matched its lockfile. No upstream app/build/test was run, no runtime source was copied and no product dependencies changed.

Ampello is the closest Tauri/React/Rust/SQLite reference. Windows defaults to paste; Type uses separate deletion and paced scalar-sized SendInput calls, marked-event metering and cancellation. Its target checks/output guarantees are weaker than ours, and Linux explicitly refuses Wayland. These mechanisms provide controlled experimental variables, not a proven native fix. Preserve exact output verification, no blind fallback/repair and separate OS capability matrices. The independent-engine and Windows-first decisions remain in force.

The next native investigation should vary one factor on a new disposable fixture and compare Notepad/control behavior: combined versus separated deletion, paced delivery, and receiver message flow. Do not silently enable a multi-batch or clipboard product strategy from upstream source alone. Keep the computer available and close only task-owned helpers/connections.

### Verified continuation builds

The final full `bun run check:all` passed **138 frontend/111 Rust tests**, plus 11 tooling tests, one loopback fixture, six AST groups, TypeScript/ESLint/Prettier/rustfmt and all-targets Clippy. The final owned receiver run passed separately; ordinary checks never sent native input. Whitespace checks and 88 local document links passed.

Debug acceptance and optimized normal-identifier builds passed. The release command `bun run tauri build -- --locked` exited successfully and produced both MSI and NSIS installers. Exact artifact sizes/hashes are in the [acceptance record](../research/offline-mvp-acceptance.md). NSIS's stalled download was resolved with checksum-verified pinned cache files, without changing dependencies. Installers have not been installed/tested or signed. The normal release executable is now the continuation MVP, superseding the old template executable described earlier. Native expansion, physical campaigns, file-dialog scopes, physical tray gestures, signing and remote CI remain open.
