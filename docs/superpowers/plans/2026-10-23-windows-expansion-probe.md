# Windows Expansion Probe Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a stopped-by-default Windows native expansion probe and produce evidence for a defensible engine/strategy decision.

**Architecture:** Rust owns a pure recognizer, a dedicated observation thread, and a separate context/insertion worker. A narrow command boundary exposes session controls and content-free status to the existing React/shadcn screen. Native mutation is restricted to explicit reference profiles and verified controls; direct Unicode, clipboard, and picker results remain separate.

**Tech Stack:** Existing Tauri 2, React/TypeScript, shadcn, Node `24.18.0`, pnpm `12.15.0`, Rust `1.99.1`; Microsoft `windows` bindings using the existing locked `0.62.2` version; official Tauri global-shortcut plugin only for the picker task.

**Spec:** [Approved Windows expansion probe design](../specs/2026-10-23-windows-expansion-probe-design.md).

## Global Constraints

- Windows first; initial environment Windows 11 x64 and US English layout `00000409`. Exact host/app/control builds are measured during execution.
- Start **Stopped** on every launch. Only explicitly selected reference app/control profiles can expand; unknown contexts are refused.
- Literal, case-sensitive `;` plus lowercase ASCII letters, at most 32 characters. **Space activates a complete trigger and one trailing space remains.** Require start-of-field or preceding whitespace.
- Cap transient matching state at 33 ASCII characters including delimiter. No raw input stream, matching suffix, target text, clipboard content, or full paths enters React/diagnostics.
- Publish an automatic request after the matching Space release; intervening input, repeat, ambiguity, or held trigger keys cancel it.
- Direct Unicode is first. Clipboard is an explicit separate experiment. Picker inserts only a body at a captured collapsed caret, deletes nothing, and adds no automatic space.
- Refuse uncertain focus/caret, unsupported password semantics, elevated/unknown integrity, modifiers, composition, unsupported layout, selection, or unrecognized controls before mutation.
- Partial/uncertain insertion stops automatic expansion. No compensating backspaces or blind replay.
- At least 300 eligible **physical-input automatic end-to-end** attempts, at least 100 per reference app; synthetic trials do not contribute. Zero observed wrong-target insertion or unrelated deletion in positive and negative campaigns.
- Retain pinned tools, frozen/locked installs, current checkout, original Texpio UI, and narrow capabilities. No backend, database, production editor, rich text, scripts, autostart, packaging release, or macOS adapter.
- Preserve the user's earlier **Native execution in the current checkout** choice. The user approved this plan for implementation on 23 October 2026; do not ask for the execution method again.

## Review Focus

- Key rollover, auto-repeat, and Space release must not cause duplicate dispatch or a false eligible candidate; pinned in Task 1 and native Task 7.
- A slow/stale UI Automation provider must not unblock into a different target or keep mutation enabled after Stop; pinned in Tasks 2–3.
- UI Automation's unsupported-property defaults must never be mistaken for an explicitly safe editable control; pinned in Task 2.
- A clipboard copy between publication, paste, verification, and restoration must win over restoration and must not cause blind repair; pinned in Task 5.
- Picker focus transitions, repeated Enter, and cancel-after-target-change must not reuse a consumed/stale target token; pinned in Task 6.

## File and interface map

| Area | Files | Responsibility |
| --- | --- | --- |
| Pure engine | `src-tauri/src/engine/{mod,types,fixtures,recognizer}.rs` | Shared contracts, synthetic fixtures, bounded matching and reset logic |
| Runtime | `src-tauri/src/engine/{runtime,control}.rs` | Worker ownership, cancellation/dispatch gate, status/counters, session transitions |
| Windows adapter | `src-tauri/src/engine/windows/{mod,context,observer,unicode,clipboard}.rs` | UIA target evidence, hooks, direct input, conditional clipboard strategy |
| Picker adapter | `src-tauri/src/probe_picker.rs` | Native shortcut, separate captured-target token, chooser lifecycle |
| IPC | `src-tauri/src/probe_commands.rs`, existing `lib.rs` | Validated commands and managed runtime; existing metadata stays separate |
| React | `src/lib/probe.ts`, `src/features/probe/{ProbePanel,FixturePicker}.tsx` and colocated tests | Typed validated bridge, engineering controls, fixture chooser |
| Existing integration | `src/features/foundation/FoundationScreen.tsx`, `src-tauri/src/desktop.rs`, `src/App.tsx` | Show probe UI, tray control parity, chooser route |
| Browser fixture | `tests/fixtures/windows-probe/{index.html,fixture.js}`, `scripts/serve-probe-fixture.mjs` | Offline test fields, synthetic sentinels, loopback-only static server |
| Evidence | `docs/research/windows-expansion-probe.md`, `docs/development.md`, `TASKS.md` | Environment, actual outcomes, failures, support decision and phase status |

Shared public contracts are declared in Task 1; private Windows/COM types stay inside their worker. Tests live beside the Rust modules in `#[cfg(test)]` modules. No input hook runs in ordinary unit tests or CI.

## Preflight — establish the actual baseline

- [ ] Read this plan and its approved spec; inspect `git status`, latest commits, and toolchain pins. Preserve unrelated user edits. Record the execution base SHA.
- [ ] Verify the foundation against the existing binary: metadata/themes, close-to-tray, Open restoring hidden/minimized windows, second process launch restoring the original with one surviving process, Quit/relaunch. A launcher that merely activates an existing process does not prove the single-instance callback.
- [ ] Run the existing check sequence once: frozen install; `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm test`, `pnpm build`; locked Cargo fmt/Clippy/tests and native build as listed in Task 7. Inspect remote CI for the exact commit or record the access/run limitation.
- [ ] Record verified foundation results in `docs/development.md`; leave any unperformed or failed item open. Resolve a foundation runtime defect before arming the new probe. A remote-CI access limitation is reported separately and does not fabricate a pass.

### Task 1: Fixed fixtures and pure recognition contract

**Files:** Create `engine/mod.rs`, `engine/types.rs`, `engine/fixtures.rs`, `engine/recognizer.rs`; modify `src-tauri/src/lib.rs` only to declare the pure module. Tests: inline `recognizer::tests` and `types::tests`.

**Interfaces:**

- Produce `ReferenceProfile::{Notepad,VsCode,ChromeFixture}`, `InsertionStrategy::{Unicode,Clipboard}`, `ProbePhase::{Stopped,Running,Paused,Fault}`, and Serde camelCase `ProbeStatus { phase, profile: Option<ReferenceProfile>, strategy, counters: ProbeCounters, refusal_reason: Option<RefusalReason>, last_activity_age_ms: Option<u64> }`. `ProbeCounters` fields are `submitted`, `observed`, `refused`, `uncertain` (`u64`); no additional captured data.
- Produce `Fixture { id: &'static str, title: &'static str, trigger: &'static str, body: String }` and `fixtures() -> Vec<Fixture>`.
- Produce `PhysicalKey::{Ascii(u8),Space,Other}`, `EngineInput::{Key { key: PhysicalKey, down: bool, repeat: bool }, Reset(ResetReason)}`, `Candidate { fixture_id: String, trigger: String }`, and `Recognizer::new(fixtures: &[Fixture]) -> Recognizer`, `Recognizer::push(&mut self, input: EngineInput) -> Option<Candidate>`, `Recognizer::clear(&mut self)`.
- `ResetReason` is a closed internal enum for focus, pointer/navigation/selection, layout/composition/modifier, injected input, pause/stop/fault, overflow and uncertainty. `RefusalReason` is a serialized closed enum with corresponding public safe categories plus unsupported profile/password semantics/integrity, stale context, provider timeout, clipboard state and uncertain outcome. Define variants as used; serialize no free-form OS messages.

- [ ] **Write failing tests:** `space_release_emits_once` asserts `;hello` plus Space down gives `None`, Space up gives fixture `hello`, repeated up gives `None`; `case_and_prefix_are_literal` rejects `;Hello` and `hello`; `rollover_repeat_and_reset_cancel` covers repeated Space, intervening key-down, unmatched release and reset before release; `bound_is_33` exercises a 32-character trigger and rejects overflow without retaining a typed history. In `types::tests`, assert `serde_json::to_value(ProbePhase::Stopped).unwrap() == "stopped"`; in fixture tests assert `fixtures().iter().find(|f| f.id == "long").unwrap().body.len() == 2_000`. Test helpers create only synthetic `EngineInput` values. Boundary confirmation belongs to Task 2, not remembered key events.
- [ ] Run `cargo test --locked --manifest-path src-tauri/Cargo.toml engine::recognizer`; confirm failure for missing recognizer behavior.
- [ ] Implement the contracts and recognizer. Fixed fixture IDs/triggers: `hello/;hello`, `address/;addr`, `accent/;accent`, `devanagari/;hindi`, `emoji/;emoji`, `multiline/;lines`, `long/;long`. Bodies: `Hello from Texpio!`, `17 Example Road, Sample City`, `Café déjà vu`, `नमस्ते दुनिया`, `Hello 👋 e\u{301}` (combining acute), `Line one\nLine two`, and exactly 2,000 ASCII `x` characters. Keep fixture bodies in Rust-owned definitions, not duplicate frontend constants.
- [ ] Add serialization assertions for stopped status and fixture DTOs; rerun the focused tests and Cargo fmt. Commit only this task as `feat: add bounded expansion probe recognizer`.

### Task 2: Read-only Windows context guard and test fixtures

**Files:** Create `engine/windows/mod.rs`, `engine/windows/context.rs`, the browser fixture/server files, and the initial `docs/research/windows-expansion-probe.md`; modify `src-tauri/Cargo.toml`, lockfile, and `package.json` for `probe:fixture`. Tests: `context::tests`, `scripts/serve-probe-fixture.test.mjs` using built-in `node:test`.

**Consumes:** Task 1 fixtures, profiles, refusal/reset reasons. **Produces:** worker-owned `WindowsContext`, opaque `TargetToken` and `PreparedEdit`; native-only `TargetStamp { foreground_window: isize, focused_window: isize, process_id: u32, thread_id: u32 }` and `current_stamp() -> Result<TargetStamp,RefusalReason>`; `WindowsContext::new() -> Result<Self,RefusalReason>`, `capture(&mut self, profile: ReferenceProfile, origin: &TargetStamp) -> Result<TargetToken,RefusalReason>`, `prepare_automatic(&mut self, target: &TargetToken, candidate: &Candidate, body: &str) -> Result<PreparedEdit,RefusalReason>`, `revalidate(&mut self, edit: &PreparedEdit) -> Result<(),RefusalReason>`, `verify(&mut self, edit: &PreparedEdit) -> Result<bool,RefusalReason>`. Each edit carries an internal deadline and expected output; no COM object crosses threads. `current_stamp` records only actual current native handles/identity and performs no UIA work; stamps/tokens never serialize to React.

- [ ] Write failing guard tests using a fake property/range provider: explicit editable/non-password plus collapsed caret and exact `;hello ` suffix/boundary may prepare; unsupported password property, false supplied as a default, stale focused control, selection, changed caret/suffix/boundary, unknown/elevated integrity and provider timeout all return a refusal with **zero mutation calls**. Include same-process two-field focus changes.
- [ ] Run the focused tests; confirm the expected missing guard behavior. Add `windows = "=0.62.2"` only under `cfg(target_os = "windows")` with narrow Foundation, WindowsAndMessaging, keyboard/mouse, Accessibility, COM/Ole/Variant, process/token/security, globalization/IME and memory features actually required. Add clipboard features in Task 5 if not yet needed. Update Cargo.lock normally once, then return to `--locked` commands.
- [ ] Implement the UIA worker-owned guard using explicit non-default password evidence, bounded adjacent ranges, stable target/control/caret identity, editability/integrity checks and monotonic context tokens. A prepared request must match its original stamp/token; do not capture an arbitrary current target after a queued candidate arrives. Configure finite UIA connection/transaction timeouts; choose an initial 250 ms overall preparation budget and 2 s completion-observation budget. Check cancellation/deadline after every blocking call and immediately before returning an edit. No fallback to guessed backspaces or whole-document logging.
- [ ] Implement `probe:fixture` as `node scripts/serve-probe-fixture.mjs`, default `127.0.0.1:4175`, serving only the fixture allowlist with no request bodies/uploads/directory traversal. The page has labeled positive textarea/input, password, readonly/disabled fields, a second focus target, contenteditable exploratory area and synthetic sentinels. Tests assert loopback binding, allowed GET assets, path rejection and method rejection. Do not open or read password values.
- [ ] Perform read-only discovery in disposable Notepad, VS Code plain-text and the local Chrome fixture. Record exact builds, UIA capabilities/control identities and refusal cases in the report. Limit profiles to the observed compatible controls, not executable name alone. Use synthetic documents; retain no real text or account screenshots. If sufficient caret/range evidence is absent, record the gate as unmet and investigate without relaxing it. Run focused tests, fmt/Clippy and commit `feat: add guarded Windows probe context`.

### Task 3: Dedicated observation and guarded direct insertion

**Files:** Create `engine/control.rs`, `engine/runtime.rs`, `engine/windows/observer.rs`, `engine/windows/unicode.rs`; update `engine/mod.rs`. Tests: `control::tests`, `runtime::tests`, `unicode::tests` with fake observer/guard/inserter.

**Consumes:** Task 1 recognizer and Task 2 guard. **Produces:** `StartOptions { profile: ReferenceProfile, strategy: InsertionStrategy }`; `ProbeHandle::new() -> ProbeHandle`, `start(&self, options: StartOptions) -> Result<ProbeStatus,RefusalReason>`, `stop(&self) -> ProbeStatus`, `set_paused(&self, paused: bool) -> Result<ProbeStatus,RefusalReason>`, `exclude(&self, profile: ReferenceProfile) -> ProbeStatus`, `status(&self) -> ProbeStatus`. Produce internal `ObservedCandidate { candidate: Candidate, origin: TargetStamp, context_generation: u64, input_generation: u64, published_at: Instant }`, `InjectionPlan` and `unicode_plan(edit: &PreparedEdit) -> InjectionPlan`; only the worker may call `dispatch(plan: &InjectionPlan) -> DispatchResult`, with exact submitted/expected event counts. Retain the original worker-owned target token associated with each observed candidate.

- [ ] Write failing tests: stopped default emits nothing; a verified candidate dispatches once; stale generations and input after candidate refuse; pause/stop during a blocked prepare prevents later dispatch; queue overflow invalidates pending edits; an expected own generated event neither rematches nor cancels its own operation; any unexpected injection resets context. Assert exactly zero dispatch calls in every pre-dispatch negative case.
- [ ] Add UTF-16 tests: greeting includes only verified trigger/delimiter deletion plus body/space; Devanagari, surrogate pairs and combining marks survive encoding; multiline line-ending mapping is explicit. A `DispatchResult` below the requested count produces `Fault`/uncertain once with no repair or retry. Test an accepted dispatch whose observed text differs also faults.
- [ ] Run the focused tests and observe the expected failures. Implement a dedicated hook/message-loop thread, bounded nonblocking queue (initial capacity 128), separate COM/insertion worker, context/input generations, own-event marker, and control/dispatch gate. Carry the original native stamp and the preexisting validated context token/generation with the candidate; refresh focus/control eligibility off the hook thread and refuse while refresh is pending. Use explicit US-layout mapping and recorded modifier transitions; never call UIA or wait in hook callbacks. Pointer/navigation/focus/layout/lock/suspend changes invalidate context; lock/resume/fault require rearming. Hooks are never installed by construction or unit tests.
- [ ] Implement one bounded Unicode `SendInput` batch after final guard/generation checks. Record dispatch and observed completion separately, with no target-text logging. An interrupted/uncertain in-flight edit closes the gate; requests acknowledge any already-dispatched uncertainty. Stop closes the gate before draining/unhooking, and workers must honor deadlines during shutdown. If bounded provider shutdown cannot be established, leave this task's native gate failed rather than blocking Quit indefinitely.
- [ ] Run fake-backend race/lifecycle tests, fmt/Clippy and locked Cargo tests. Perform only explicitly armed synthetic-document native trials; ordinary CI remains non-injecting. Confirm hooks are gone after Stop/Quit and no queued work fires on restart. Commit `feat: probe guarded Windows text replacement`.

### Task 4: Native commands, maintainer controls and tray state

**Files:** Create `src-tauri/src/probe_commands.rs`, `src/lib/probe.ts`, `src/lib/probe.test.ts`, `src/features/probe/ProbePanel.tsx`, `ProbePanel.test.tsx`; modify `lib.rs`, `desktop.rs`, `FoundationScreen.tsx` and its existing tests.

**Consumes:** Task 3 `ProbeHandle`, Task 1 status/fixtures. **Produces:** Tauri commands `probe_status`, `probe_fixtures`, `probe_start(options: StartOptions)`, `probe_stop`, `probe_pause(paused: bool)`, `probe_exclude(profile: ReferenceProfile)`; each control returns `Result<ProbeStatus,ProbeError>` where `ProbeError { code: RefusalReason }` exposes no OS details. `probe_status` and `probe_fixtures` are read-only. Frontend wrappers `getProbeStatus()`, `getProbeFixtures()`, `startProbe(options)`, `stopProbe()`, `pauseProbe(paused)`, `excludeProbe(profile)` return validated typed promises.

- [ ] Write failing bridge tests for malformed data, unknown enum values, negative/nonfinite/unsafe-integer counters, rejected commands and browser mode; ignore extra fields rather than forwarding them, and reject non-native use with `Windows expansion probe is unavailable.`. For the mocked browser test, assert `await expect(getProbeStatus()).rejects.toThrow('Windows expansion probe is unavailable.')` and no IPC call. Write panel tests that Start requires a profile, defaults to Unicode, shows Stopped initially, serializes pending mutations, exposes Pause/Stop, displays fixed refusal labels, and never reports queued events as successful expansions. A stale async response after Stop/unmount must not restore Running in the UI.
- [ ] Run the focused Vitest tests and confirm failure. Implement wrappers like the existing `getAppInfo` boundary; add `ProbePanel` below metadata. Poll content-free status every 500 ms while mounted, clean up on unmount, and refresh after commands. Engine workers continue independently while the window is hidden. Render Rust fixture titles/triggers/bodies as known synthetic data; no live captured text is displayed.
- [ ] Manage one `ProbeHandle` in Rust, keep Windows cfg boundaries, and return a fixed unsupported-platform error elsewhere. Register no broad filesystem/shell/network capability. Keep official single-instance initialization first. Add tray Pause/Resume and Stop with state reflecting the same handle; Quit tears down the probe before exit. A hidden/minimized window does not pause implicitly.
- [ ] Add Rust command serialization/transition tests and panel tests for fault, retry, exclusions and tray-equivalent status transitions. Run frontend checks and locked Rust tests; build and manually verify default Stopped and window/tray control parity. Commit `feat: add Windows expansion probe controls`.

### Task 5: Explicit clipboard strategy comparison

**Files:** Create `engine/windows/clipboard.rs`; update `runtime.rs`, `Cargo.toml`/lockfile only if narrow API features need adding, probe controls/tests, and report. Tests: `clipboard::tests` against an in-memory clipboard/target backend; no real user clipboard in tests.

**Consumes:** Tasks 2–4 guard, runtime, strategy selector. **Produces:** worker-owned `ClipboardLease`, `prepare_clipboard(body: &str) -> Result<ClipboardLease,RefusalReason>`, `paste_verified(edit: &PreparedEdit, lease: &mut ClipboardLease) -> Result<(),RefusalReason>`, `restore_if_owned(lease: ClipboardLease) -> Result<(),RefusalReason>`. The implementation owns the complete publication/restore transaction, including Stop/Drop cleanup; no lease travels over IPC.

- [ ] Write failing tests for empty/plain-text snapshot roundtrip; lock contention; unknown/private/image/file/delayed formats refused **before either clipboard or target mutation**; changed sequence/owner before paste; concurrent user copy during observation/restoration; and Stop while a restore is pending. Assert a newer user's copy is never overwritten and no blind deletion/replay follows uncertain paste.
- [ ] Run focused tests and confirm failure. Implement explicit supported-format enumeration, deep byte snapshots for supported eager plain-text formats, correct handle ownership, owner/sequence checks, and observation-based consumption. Any non-snapshottable format refuses. Use a private owner window on the native worker as required; do not clear an unknown clipboard to make a test pass. Document the exact supported format set and delayed-rendering limitation.
- [ ] Expose Clipboard only through explicit experimental selection with a plain description of clipboard history/manager exposure. Keep Unicode the default and do not use clipboard as an automatic fallback. Use only test-owned clipboard fixtures for automated native trials; perform real UI trials with user authorization for the affected clipboard state.
- [ ] Run race tests and native synthetic clipboard cases, record timing/failures separately, and commit `feat: add explicit clipboard insertion experiment`. If preservation/consumption cannot be qualified, keep the mode disabled with a precise refusal/report rather than claiming strategy success.

### Task 6: Captured-target picker experiment

**Files:** Create `src-tauri/src/probe_picker.rs`, `src/features/probe/FixturePicker.tsx`, `FixturePicker.test.tsx`; modify `engine/windows/context.rs`, `probe_commands.rs`, `src/lib/probe.ts`/tests, `src/App.tsx`, `lib.rs`, `Cargo.toml`/lockfiles and Tauri configuration only as needed for the small chooser.

**Consumes:** Task 1 fixtures, Task 2 target guard, Task 3 gate and Task 4 bridge. **Produces:** separate opaque single-use `PickerToken`; `capture_picker(&mut self, profile: ReferenceProfile) -> Result<PickerToken,RefusalReason>`, `prepare_picker(&mut self, token: PickerToken, body: &str) -> Result<PreparedEdit,RefusalReason>`. Commands `probe_picker_insert(fixture_id: String)` and `probe_picker_cancel()` operate on the internally held token and never accept a target HWND/PID/text from React.

- [ ] Write failing tests that picker insertion deletes zero characters and adds no trailing space; cancel dispatches nothing; stale/changed caret/anchor, edited/destroyed original target and unexpected focus refuse; duplicate Enter/replayed command cannot consume a token twice. Distinguish expected chooser focus transitions from the automatic recognizer generation and invalidate normal matching when opening it.
- [ ] Run those tests; add a stable Tauri-2-compatible official global-shortcut plugin and lock its resolved version. Register Ctrl+Shift+Space only while armed, surface a fixed conflict reason, and unregister on Stop/Quit. Capture the actual original target before showing/focusing a dedicated small `probe-picker` window; if restoration cannot retain evidence, refuse.
- [ ] Add the chooser route and body-only selection/Enter/Escape UI. Keep it hidden/nonexistent outside a picker session. Restore the captured control/caret through the supported adapter and revalidate before insertion; no insertion into an arbitrary current foreground window. No search database or full editor. Add malformed command/body identity checks; fixtures are resolved by Rust IDs.
- [ ] Run frontend and Rust tests, then native restore/cancel/window-switch cases for all reference profiles. Record unsupported paths, shortcut conflicts and actual results separately from automatic expansion. Commit `feat: add captured-target probe picker`.

### Task 7: Native qualification, report and phase decision

**Files:** Update `docs/research/windows-expansion-probe.md`, `docs/development.md`, `TASKS.md`, `README.md`, and existing `.github/workflows/checks.yml` only if checks need explicit new commands. Fix implementation defects in their owning files with regression tests before repeating affected campaigns.

**Consumes:** Tasks 1–6 and the approved evidence gate. **Produces:** an actual measured report and one explicit advance/continue/revise decision; never an invented completion claim.

- [ ] Freeze the commit/build, Windows/app/control versions, keyboard layout, fixture IDs, profile rules, strategy and evidence classes. Record how trials are generated. Treat injected UI-automation keys as synthetic; do not disable normal injected filtering just to make automated counts look physical. Any harness-only tagged-input acceptance is test-only and separately reported.
- [ ] Execute at least 100 physical-input automatic direct-mode trials per reference app (300 total), including all fixture output classes/speeds. Verify exact output, original target and unrelated sentinels; count eligible refusals as unsuccessful completions. If physical input assistance is unavailable, leave these counts/gates pending and provide the short reproducible human test procedure. Synthetic tests alone never complete P0.
- [ ] Run separate negative campaigns for all conditions in the spec, especially focus changes around dispatch, same-app field changes, pending Stop/Pause, held modifiers, composition/layout, unknown/password/elevated controls, provider timeouts and overflow. Any wrong-target insertion/unrelated deletion stops the campaign and blocks advancement until fixed and affected campaigns rerun.
- [ ] Run and report clipboard and picker experiments separately, including refusal matrices. Record recognition-to-dispatch and observed completion p50/p95/max, delimiter-down timing, sample counts, the long fixture, settled idle CPU and combined resident memory on this host. No P1 reference-hardware performance claim follows from this host's measurements.
- [ ] Run the full locked checks below, a final Windows build, and foundation/probe native regression checks on that binary. Verify remote CI against the pushed implementation SHA or record why unavailable. Keep CI non-injecting; add the built-in fixture-server test command if introduced. No release/signing workflow.

```powershell
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm format:check
pnpm test
node --test scripts/serve-probe-fixture.test.mjs
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
pnpm tauri build --no-bundle -- --locked
```

- [ ] Write the observed engine/strategy comparison and license/dependency record. No Espanso source reuse or bundled runtime is included. Name accepted control profiles and unsupported cases, and choose advance, continue, or revise. An unmet three-app/physical-trial/safety gate remains unmet even if the application builds.
- [ ] Obtain the Native execution method's independent final implementation review, address supported findings and rerun affected checks. Review the complete execution diff against the recorded base SHA, not an empty comparison against the current `main` HEAD.
- [ ] Commit the evidence/status and verified fixes, push authorized project changes, verify remote SHA and CI state, and leave a working stopped-by-default preview if requested. Only mark P0 complete if the report actually satisfies its evidence gate; P1 is a separate reviewed scope.

## Coverage and self-review

The approved spec's architecture, bounded matching and physical release rules map to Tasks 1/3; context, password/integrity and caret evidence to Task 2; control/lifecycle/IPC to Tasks 3/4; insertion comparison to Tasks 3/5; picker restoration to Task 6; foundation regression, physical/synthetic separation, campaigns, resources and phase decision to Preflight/Task 7. Screenshot-derived management features stay in the later-phase backlog.

Self-review completed: file responsibilities and shared interfaces are explicit; fixed values and fixture bodies match the spec; each Review Focus item has an owning test step; all design requirements have a task; no product code or dependency installation is part of writing this plan. The physical campaign requires human typing or equivalent genuine hardware events; this prerequisite is visible rather than hidden in an automated test claim.

## Execution handoff

The user approved the written design and requested this implementation plan. **Plan review is pending.** Preserve Native execution in this checkout after review; do not silently start implementing or switch to delegated implementation. Read-only support research/review may still be delegated.
