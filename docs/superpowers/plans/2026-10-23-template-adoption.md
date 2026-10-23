# Full Template Adoption Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for the user's retained Native/current-checkout execution method. Steps use checkbox (`- [x]`) syntax for tracking. Read-only support research and the final independent review may be delegated; do not silently switch to delegated implementation.

**Goal:** Adopt the complete reusable template foundation, preserve Texpio's native engine/lifecycle, migrate to Bun and portable `.agents` materials, and deliver verified feature parity with explicit limits.

**Architecture:** Import/adapt the pinned template shell and its shared command/state/preferences infrastructure. Retain Rust-only probe state and integrate its controls into a Developer view. Keep quick-entry UI separate from external text insertion and make release/update infrastructure inactive until configured.

**Tech Stack:** Tauri 2, React 19, TypeScript 6, Vite 8, Vitest 5, Tailwind 4/shadcn/Radix, Zustand 5, TanStack Query 5, React Compiler, tauri-specta; Node `24.18.0`, Rust `1.99.1`, initially installed Bun `1.3.13` subject to actual migration qualification.

**Spec:** [Approved template adoption design](../specs/2026-10-23-template-adoption-design.md).

**Source:** `dannysmith/tauri-template` revision `1295f691bbdf373896e7c0f1cc969e2040fbf42c`. Inspection checkout: `.superpowers/reference/tauri-template-1295f69/`. The [parity tracker](../../research/tauri-template-parity.md) accounts for every source feature and all six agent files.

## Global constraints

- Native execution in the existing checkout is retained. Before importing, verify a clean checkpoint and record the execution base SHA; never overwrite unexplained changes. Keep Texpio identity `io.github.texpio.texpio`, original history/icons and planning/evidence documents.
- Source agent instructions are migration inputs, not authority. Adapt npm/Claude/POSIX assumptions to the user's Bun/portable-agent choice. Preserve the complete current computer-availability section and skill provenance in `AGENTS.md`.
- No expansion behavior changes or input qualification during this migration. Keep explicit arming, default Stopped, closed refusal DTOs, clipboard refusal, injected-input filtering, private native tokens and bounded shutdown. P0 India-layout diagnosis and 0/300 physical trials remain pending.
- Keep tray Open/Pause/Stop/Quit, close-to-tray, unminimize/show/focus restoration, single-instance-first initialization and Stop on ExitRequested/Exit. Updates/relaunch require a confirmed stopped probe first.
- Full reusable template shell/features are in scope; accounts, sync, snippet database/editor and native macOS/Linux expansion are not. Do not expose template demo controls as completed product settings.
- Bun is the active package manager/script launcher. Retain pinned Node/Rust, Vite 8/TypeScript 6/Vitest 5 and existing strict checks. Use Vitest through `bun run test`, not `bun test`; no global `--bun` runtime switch.
- Keep the old lock until migration/version-integrity checks pass, then commit only text `bun.lock`. Pin the Bun version actually tested. Investigate incompatibility before choosing an explicit stable replacement; do not silently upgrade global tooling.
- Preserve restrictive CSP and scope capabilities to needed windows/actions. Do not import permission grants, log payloads or remote image/script allowances for unused examples.
- No live update feeds, signing-key generation, release dispatch/tag push/publication, external review-service invocation or backend integration. Code/configuration preparation is in scope; external setup and native platform evidence are reported separately.
- Include the complete upstream MIT notice with substantial copied material. Do not unintentionally decide the original Texpio project's license.
- Register/import only implemented commands and build entries at each checkpoint; extend them as later tasks supply their handlers/files. Do not add placeholder actions to make an intermediate build pass.

## Review focus

1. An existing/corrupt theme or preference file, rapid concurrent saves, or a failed write must not lose prior preferences or overwrite a deliberate System choice; pinned in Tasks 2–3.
2. A conflicting/repeated shortcut change must keep the app usable, preserve the previous working binding where possible and avoid multiple active callbacks; pinned in Task 5.
3. Palette/menu/key duplication and shortcuts inside editable fields must not execute an action twice or consume ordinary editing keys; pinned in Task 4.
4. Hide/minimize/second launch/update/quit must not bypass probe teardown or strand a hidden main window; pinned in Tasks 5–6 and native Task 8.
5. A crash, quick-entry event or plugin error must not leak input, text, target identity or OS paths through logging/recovery/typed bindings; pinned in Tasks 2 and 6.

## File and interface map

| Area                 | Owned files                                                                                                                                                                          | Responsibility                                                            |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| Tooling              | `package.json`, `bun.lock`, `.bun-version`, `.node-version`, `vite.config.ts`, `vitest.config.ts`, `tsconfig*.json`, `eslint.config.js`, `prettier.config.js`                        | Reproducible install/build/test/quality commands                          |
| Native services      | `src-tauri/src/{lib,bindings,types}.rs`, `commands/{mod,metadata,preferences,recovery,notifications,quick_pane}.rs`, existing `probe_commands.rs`                                    | Typed commands and app-owned persistence; preserve private engine modules |
| Shell/providers      | `src/{App,main,quick-pane-main}.tsx`, `components/{layout,titlebar,preferences,command-palette,quick-pane,ui}/`, `store/ui-store.ts`, `services/preferences.ts`, `i18n/`, `locales/` | Shared UI, settings, theme/language and windows                           |
| Command routing      | `src/lib/commands/`, `src/lib/menu.ts`, `src/hooks/use-keyboard-shortcuts.ts`                                                                                                        | Single action registry and platform adapters                              |
| Lifecycle            | `src-tauri/src/desktop.rs`, `commands/quick_pane.rs`, `lib.rs`, Tauri configs/capabilities                                                                                           | Tray/window/shortcut ownership and shutdown                               |
| Safety/services      | `src/lib/{logger,recovery,notifications,updater}.ts`, `src/components/ErrorBoundary.tsx`, `src-tauri/src/commands/lifecycle.rs`                                                      | Redacted feedback, bounded recovery and update gating                     |
| Portable agents/docs | `.agents/{README.md,skills/,agents/}`, root `AGENTS.md`, optional thin `CLAUDE.md`, third-party notice, `docs/developer/`, task/user docs                                            | Portable workflows, provenance and accurate guidance                      |
| Evidence/CI          | `.github/workflows/`, `scripts/`, `.ast-grep/`, `sgconfig.yml`, `knip.json`, `.jscpd.json`, parity tracker                                                                           | Safe checks, prepared releases and actual feature evidence                |

### Task 1: Bun toolchain and portable build/quality baseline

**Files:** Modify `package.json`, the package-manager guidance in `AGENTS.md`, Tauri `build.beforeDevCommand`/`beforeBuildCommand`, Vite/Vitest/TypeScript/ESLint configuration and ignore files; create `.bun-version`, `bun.lock`, `prettier.config.js` and missing upstream tooling configs. Remove `pnpm-lock.yaml` only after successful conversion. Keep existing fixtures/tests and Node/Rust pins.

**Consumes:** Current lock/dependency versions, pinned upstream package/config files. **Produces:** `bun run dev`, `build`, `preview`, `tauri`, `typecheck`, `lint`, `format`, `format:check`, `test`, `test:watch`, `test:ui`, `test:coverage`, `probe:fixture`, `test:probe-fixture`, `rust:fmt`, `rust:fmt:check`, `rust:clippy`, `rust:test`, `rust:bindings`, `tauri:dev`, `tauri:build`, `tauri:check`, `ast:lint`, `knip`, `jscpd`, `check:all` and explicit fix helpers. Test/run scripts use Vitest; watch is opt-in. `rust:bindings` is added when Task 2 supplies its exporter; no placeholder command is included in `check:all`.

- [x] Record `git status`, current resolved dependency versions, Bun `--version`/CLI capabilities and frozen baseline checks. Run existing frontend tests through Bun against existing `node_modules` before changing resolution. Expected: the current tests run under Vitest, with no native input; record any preexisting fork-worker teardown limitation separately.
- [x] Dry-run the actual Bun migration against the pnpm v9 multi-document lock. Compare React/Vite/TS/Tauri and current dependency versions/integrities; investigate unsupported parsing rather than silently fresh-resolving everything. Pin the successful Bun version. Then migrate/add the upstream feature dependencies, matched Vitest 5 UI/coverage packages and compiler integration; inspect necessary lifecycle scripts and trust only required packages.
- [x] Adapt Vite's supported React Compiler configuration, retaining the main entry; Task 5 adds the second HTML entry only when its file exists. Fix the upstream obsolete Vitest `react({ babel: ... })` usage. Resolve ESLint peer conflicts explicitly; do not suppress incompatible peers or downgrade core tools silently.
- [x] Replace POSIX/npm script chains with Bun and portable Cargo `--manifest-path`/`--locked` commands. `tauri:check` performs typecheck plus `cargo check --locked`, not the source's undocumented `tauri build --check`. Add correct tool configuration without treating unused template library exports as automatic deletions.
- [x] Verify `bun.lock` exists, clean/frozen install, `bun run typecheck`, `lint`, `format:check`, `test`, `test:probe-fixture`, `build`, and locked Rust baseline checks. Expected: successful commands and existing behavior preserved. Document configuration checks instead of adding tests that merely restate package fields. Commit `build: adopt Bun and template tooling baseline`.

### Task 2: Typed native bridge, preferences and bounded recovery services

**Files:** Move existing `src-tauri/src/commands.rs` to `src-tauri/src/commands/metadata.rs`; create `src-tauri/src/commands/{mod,preferences,recovery,notifications}.rs`, `src-tauri/src/{types,bindings}.rs`; modify `lib.rs`, `probe_commands.rs`, public engine DTO derives, `Cargo.toml`/lock; generate `src/lib/bindings.ts`, create `src/lib/tauri-bindings.ts`, and adapt existing `src/lib/desktop.ts`/`desktop.test.ts`, `src/lib/probe.ts`/tests. Remove the old `commands.rs` after its move so it cannot conflict with `commands/mod.rs`. Do not export internal target/observer/COM types.

**Consumes:** Existing probe handle/status and metadata contracts. **Produces:** generated `commands` with existing metadata/probe methods and template service methods; `AppPreferences { theme: 'system'|'light'|'dark', quick_pane_shortcut: string|null, language: 'en'|'fr'|'ar'|null }`; Rust `load_preferences`, `save_preferences`, `migrate_legacy_theme(theme)`, bounded emergency save/load/cleanup and native notification commands. Service failures use fixed public codes, not arbitrary OS error/path strings.

Persistence is testable through `PreferencesStore::new(root: PathBuf)`, `load() -> Result<AppPreferences, ServiceError>`, `save(&AppPreferences) -> Result<(), ServiceError>` and `migrate_legacy_theme(Theme) -> Result<AppPreferences, ServiceError>`. `Theme` is the serialized system/light/dark enum. Use disposable test roots; commands inject app-data paths rather than tests constructing an actual app/window.

Representative RED contract in `commands/preferences.rs::tests::legacy_theme_does_not_override_saved_system`:

```rust
store.save(&AppPreferences::default()).unwrap();
store.migrate_legacy_theme(Theme::Dark).unwrap();
assert_eq!(store.load().unwrap().theme, Theme::System);
```

- [x] Write failing Rust tests for missing-file defaults, save/load roundtrip, corrupt JSON without auto-overwrite, invalid theme/language, concurrent serialized saves and failed writes preserving previous data. Legacy migration under the same persistence lock writes only when no preference file exists; an existing explicit System choice survives. Write recovery tests for traversal/separators/Windows reserved names/oversized save and load, missing files, cleanup age and failed temp-write finalization. Run focused tests. Expected: missing new behavior FAILS; ordinary tests install no hooks/send no input.
- [x] Port services with testable app-data roots, bounded file operations, serialized atomic preference writes, validated filenames and closed errors. Retain `10_485_760` recovery bytes and seven-day cleanup. Do not log arbitrary recovery JSON/filenames, native errors or paths. No default/greet example replaces Texpio metadata.
- [x] Pin mutually compatible source Specta versions; collect only public command DTOs and all existing probe commands. Export from a path derived from `CARGO_MANIFEST_DIR`, not a caller-relative `../src` path. Represent public u64 counters consistently with the existing number boundary and retain safe-integer runtime rejection. Keep single-instance-first bootstrap and managed `ProbeHandle` while adding services/plugins.
- [x] Add bridge regressions asserting malformed data, unknown fields/enums, unsafe counts and native-only browser refusal remain closed. Verify generated output contains no `TargetToken`, `TargetStamp`, HWND/process/COM types or free-form probe error payloads. Run `bun run rust:bindings`, then confirm a second export is unchanged. Expected: deterministic tracked bindings; Rust and frontend focused/full suites pass.
- [x] Run strict Clippy, native compile and production frontend build before committing `feat: adopt typed native services and preference storage`.

### Task 3: Shell, state providers, themes and i18n

**Files:** Import/adapt `src/components/{layout,titlebar,ui}/`, `components/ThemeProvider.tsx`, `hooks/{use-platform,use-theme,useSquareCornersEffect}.ts`, `store/ui-store.ts`, `services/preferences.ts`, `lib/{query-client,theme-context,utils,platform-strings}.ts`, `src/i18n/`, `locales/{en,fr,ar}.json`, `App.tsx`, `main.tsx`, application CSS and matching tests.

**Consumes:** Task 2 typed preferences. **Produces:** shell providers, `useUIStore` selectors/actions, `activeView: 'home'|'developer'`, `LayoutState { leftVisible, rightVisible, leftPercent, rightPercent }`, theme service and translated shell. Only layout presentation state persists in the UI store; palette/preferences open state and quick-entry text do not persist there.

`normalizeLayout(value: unknown): LayoutState` owns untrusted storage normalization. In `store/ui-store.test.ts`, pin `normalizes invalid restored widths` with these assertions before implementation:

```typescript
const layout = normalizeLayout({ leftPercent: Infinity, rightPercent: -10 })
expect(layout.leftPercent).toBe(20)
expect(layout.rightPercent).toBe(20)
```

- [x] Write/run failing tests for safe layout hydration, hide/show/resize persistence, bad or nonfinite stored widths, and a viable main panel. Defaults are 20% left/20% right; visible side widths stay 15–40% with at least 30% main space. Invalid storage resets safely. Test actual store/provider behavior, not copied config literals.
- [x] Write/run failing tests for old `texpio.theme` migration without overriding an existing backend preference, failure without deleting the old value, successful one-time migration, live OS theme changes and cross-window events. The backend is authoritative; a new namespaced appearance cache is only a cache. Add English fallback, saved/system language selection, Arabic RTL and malformed/unsupported language tests.
- [x] Port the complete reusable UI collection/providers/layout and platform branches, consolidate conflicting existing components, and retain accessible composition/semantic tokens. Wire persisted real settings; omit nonfunctional source examples. Use supported panel collapse APIs instead of CSS hiding that leaves unusable layout allocations.
- [x] Keep browser platform fallback explicitly unknown/unavailable. Preserve real Windows titlebar/minimize/maximize/drag behavior in native branches; no macOS claim from Windows tests. Run focused then full frontend tests, typecheck/lint/format/build. Expected: passing tests/build and no competing theme contexts. Commit `feat: adopt template shell themes and localization`.

### Task 4: Commands, native menus, preferences and Developer probe view

**Files:** Import/adapt `src/lib/commands/{registry,types,index,navigation-commands,window-commands,notification-commands}.ts`, `lib/menu.ts`, `hooks/{use-command-context,use-keyboard-shortcuts,useMainWindowEventListeners}.ts`, `components/{command-palette,preferences}/`, `features/developer/DeveloperView.tsx`, existing foundation/probe screens and tests, all three locales.

**Consumes:** Task 3 providers/store plus typed preferences. **Produces:** common `executeCommand(id: string, context: CommandContext): Promise<CommandResult>` adapter to the imported registry, with `CommandResult = { success: true } | { success: false; code: 'unknownCommand'|'unavailable'|'failed' }`. Preserve the source `CommandContext` functions `openPreferences` and `showToast`; add only named capabilities consumers actually require. Palette/native-menu/local-key adapters invoke the same registered actions; navigation to Home/Developer; real persisted settings and existing probe controls.

In `src/lib/commands/commands.test.ts`, `unavailable commands do not execute` registers an action with `isAvailable: () => false`, asserts `await executeCommand(id, context)` equals `{ success: false, code: 'unavailable' }`, and asserts the action spy has zero calls. Never return a thrown OS/error message as a command result.

- [x] Write/run failing command tests: palette/menu/key invocation of the same ID executes once; editable fields retain ordinary editing keys; unsupported modifiers/repeat do not duplicate actions; available/unavailable/error results are consistent. Test listeners clean up through StrictMode/unmount/language rebuilds and display platform-correct labels.
- [x] Port command palette, command/context utilities and native menus. Route shared actions consistently through the registry instead of retaining upstream bypasses. Native editing/predefined menu actions remain native; do not invent document Open/Save. Window Close hides Texpio through the preserved native lifecycle; Quit follows native shutdown.
- [x] Port persisted theme/language/shortcut settings and accessible dialogs; query failure shows fixed retry feedback and must not silently save default data over a corrupt file. Shortcut registration is connected in Task 5. Wire Home/Developer navigation and place existing metadata/probe controls in the Developer view without changing engine matching/insertion behavior.
- [x] Add integration regressions for navigation to Developer, initial Stopped, selected-profile requirement, disabled clipboard reason, polling cleanup and stale responses after Stop. Run full frontend tests and build. Expected: command/menu/prefs behavior and current probe boundary preserved. Commit `feat: add shared commands preferences and developer view`.

### Task 5: Quick pane, global shortcut and native lifecycle

**Files:** Create/adapt `quick-pane.html`, `src/quick-pane-main.tsx`, `src/quick-pane.css`, `components/quick-pane/QuickPaneApp.tsx`, `src-tauri/src/commands/quick_pane.rs`; modify `lib.rs`, `desktop.rs`, `bindings.rs`, platform utilities, Cargo lock/configs and window capabilities; add `tauri.{windows,macos,linux}.conf.json` and native/React tests.

**Consumes:** Task 2 typed bridge/preferences, Task 3 theme, Task 4 settings/commands and existing tray/engine. **Produces:** typed `show_quick_pane`, `dismiss_quick_pane`, `toggle_quick_pane`, `get_default_quick_pane_shortcut`, `update_quick_pane_shortcut`; default `CommandOrControl+Shift+.`. The app event `quick-pane-submit` carries only explicit text entered inside this app, and never triggers external insertion.

In `QuickPaneApp.test.tsx`, `Escape never submits entry` types a known synthetic entry, sends Escape, asserts `dismissQuickPane` is called once and the `quick-pane-submit` emitter has zero calls. Rust's fake shortcut backend test `conflict_keeps_previous_binding` asserts the old chord and saved preference remain unchanged and no second handler is installed.

- [x] Write/run failing fake-backend shortcut tests for conflict, malformed chord, repeated same chord, concurrent changes, save failure/rollback and unregister failure. Failed replacement must preserve prior binding where possible, return a fixed error and remain recoverable. Startup conflict must not abort main app/tray startup. No tests register actual global shortcuts.
- [x] Port narrow native shortcut/window infrastructure. Serialize replacement with explicit ownership/generation; do not hold blocking application locks across main-thread-dispatched plugin APIs. Register a valid replacement before releasing the old binding; remove the replacement if persistence fails. Record/handle failed cleanup without false success. Keep the quick pane out of the window-state plugin's saved-state operations.
- [x] Port hidden floating-window creation/positioning and OS branches, pin the macOS NSPanel dependency revision, and build the second HTML entry. Write/run UI tests for Enter submit once, Escape/blur cancel, listener cleanup, translated placeholder and live theme updates. Empty input emits nothing; received entries are not logged or persisted as probe state.
- [x] Reconcile window-state restore with Texpio's tray lifecycle, show/unminimize/focus second-instance behavior and explicit Stop on exit. Keep quick-pane and native engine shortcuts distinct. Add pure lifecycle/adapter tests verifying the engine stop call precedes exit and no hidden-window cleanup re-arms it.
- [x] Run frontend/Rust suites, strict Clippy and dual-entry Windows no-bundle build. Expected: compile/tests pass; native shortcut/focus acceptance remains explicitly pending until Task 8. Commit `feat: add template quick pane and preserve tray lifecycle`.

### Task 6: Notifications, recovery feedback, redacted logging and updater gating

**Files:** Import/adapt `src/lib/{logger,recovery,notifications,updater,context-menu}.ts`, `components/ErrorBoundary.tsx`, notification commands/registration; create `src-tauri/src/commands/lifecycle.rs`; modify `App.tsx`, menu/preferences/version UI, command collector, capabilities/config and locales; add focused tests.

**Consumes:** Typed Task 2 services and Task 5 lifecycle. **Produces:** `getUpdateConfiguration(): { configured: boolean }`, `checkForUpdates(): Promise<UpdateCheckResult>` with explicit unconfigured/no-update/available/error states; native `prepare_for_relaunch() -> Result<(), RefusalReason>` stops the probe and refuses uncertain shutdown; notification/recovery adapters with fixed user-facing errors.

In `src/lib/updater.test.ts`, `unconfigured updates never call plugin check` asserts an absent endpoint/key returns `{ status: 'unconfigured' }` and the mocked updater `check` has zero calls. `UpdateCheckResult` uses the `status` discriminator; available results carry version metadata only, while installation is a separate explicit action. No public result includes credentials or raw native errors.

- [x] Write/run failing tests that an unconfigured feed/key makes zero network/plugin-update calls; cancellation performs no download/install/relaunch; installing/relaunching awaits confirmed native Stop; Fault/timeout prevents the action. Configuration checks must reject placeholders and invalid/non-HTTPS feeds, rather than trusting a toggle alone.
- [x] Port update flow behind actual validated configuration, with localized accessible confirmation/progress/error UI. Keep active endpoints/keys absent by default. Stop before installation/relaunch; normal UI checks never generate keys, tags or releases.
- [x] Write/run tests that quick-entry text, fake captured/clipboard payloads, arbitrary OS error paths and native target fields do not enter logs/recovery/public error feedback. Port logger/notification/context-menu utilities and error boundary. Save only allowed diagnostic metadata through bounded recovery; accurately say diagnostics were saved only after success, never unsaved-work recovery. Native notification failures may fall back to a toast.
- [x] Scope capabilities by actual window/action. Inspect production output/CSP to ensure no new remote scripts/images or quick-pane FS/clipboard/updater powers. Run full checks and build. Expected: closed failures, no placeholder startup requests and no probe privacy regression. Commit `feat: add safe desktop services and configured update flow`.

### Task 7: Portable agents, documentation, static checks and release preparation

**Files:** Create `.agents/README.md`, four `.agents/skills/{check,init,cleanup,change-package-manager}/SKILL.md`, two `.agents/agents/{cleanup-analyzer,userguide-reviewer}.md`, `third_party/tauri-template/LICENSE.md` and root `THIRD_PARTY_NOTICES.md`; merge `AGENTS.md` and optional thin `CLAUDE.md`. Import/adapt all 30 pinned upstream Markdown guides under `docs/`, relevant `.vscode`/Cursor/review settings, `.ast-grep/`, `sgconfig.yml`, `knip.json`, `.jscpd.json`, `scripts/{complete-task,prepare-release}.js`, and `.github/workflows/{checks,release}.yml`.

**Consumes:** Implemented commands/modules and selected Bun version. **Produces:** complete six-file portable mapping, correct developer/user docs and working check/task/release-preparation scripts. `TASKS.md` remains canonical. No duplicated canonical `.claude` tree; no external service/publishing action is invoked.

- [x] Before adapting skill behavior, capture baseline portability failures from source references and run focused read-only skill pressure cases: existing project must not reinitialize on unrelated work; cleanup must remain report-only until authorized changes; package migration must preserve pinned versions and never publish. Use the skill-creator/writing-skills guidance and static manifest/link validation; do not claim universal automatic role/slash registration.
- [x] Port all four skills and both role prompts, correct paths/tool names/arguments, link the index from root instructions and preserve source attribution. Verify all six source paths are accounted for and every referenced local file/script exists. Preserve computer-availability instructions byte-for-byte unless only link formatting changes; adapt PM-specific guidance to Bun without losing provenance.
- [x] Import/adapt all 30 guides with current module names/actual feature states; merge existing indexes instead of overwriting Texpio plans. Correct onboarding/security/contributing placeholders without inventing support contacts or response promises. Keep source examples distinguishable from product behavior. Update skills/docs to actual Bun commands while retaining historical evidence as historical.
- [x] Add ast-grep rule fixture cases for positive/negative architecture patterns, configure both HTML entries and generated code exclusions in Knip/jscpd, and run their reports. Do not delete the copied UI library merely because a cleanup tool reports unused reusable exports. Make `check:all` include real commands only and all required fixture/native checks.
- [x] Adapt task helpers to safe project-contained paths and unambiguous task selection. Test with disposable fixture directories that traversal/ambiguous names refuse and only the selected task moves. Separate `release:prepare` from publication: dry-run/version preparation cannot commit/tag/push or run signing tools. Test no subprocess publishes without an explicit publishing invocation, which is not run here.
- [x] Convert existing CI to SHA-pinned setup-bun and exact selected Bun/Node/Rust versions, explicit lock existence/frozen install, Linux frontend/Windows non-injecting native checks. Adapt release preparation for the three OS targets with pinned actions; keep workflow manual/opt-in and gated on real signing/feed setup. No automatic migration release. Commit `chore: add portable agents docs and release tooling` after validation.

### Task 8: Complete verification, parity evidence and handoff

**Files:** Update parity tracker, `README.md`, `TASKS.md`, `docs/development.md`, relevant user/developer docs and CI evidence. Fix supported implementation findings in their owning files with regression tests.

**Consumes:** Tasks 1–7 and recorded execution base. **Produces:** row-by-row final parity evidence, explicit remaining platform/external gates and preserved P0 status.

- [ ] Run the full commands below with frozen/locked inputs. Inspect each exit status; generated binding export must leave no diff on its second run. Confirm Vitest runs, both frontend entries build, and ordinary tests never start hooks/send input.

```powershell
if (-not (Test-Path -LiteralPath bun.lock)) { throw 'Missing committed Bun lockfile' }
bun install --frozen-lockfile
bun run check:all
bun run test:coverage
bun run knip
bun run jscpd
bun run build
bun run rust:bindings
git diff --exit-code -- src/lib/bindings.ts
bun run tauri:build --no-bundle -- --locked
```

- [ ] Verify native Windows on that binary in bounded sessions: correct Texpio chrome/menu/palette, keyboard behavior in editable fields, real settings/layout persistence after restart, theme/language/RTL and quick-pane synchronization, shortcut conflict/rebind/Enter/Escape/blur, monitor placement, hidden/minimized/second-instance restoration, tray parity and clean Quit/relaunch with Stopped engine. Do not arm input to test the shell. Record untested DPI/packaged-notification/platform cases explicitly.
- [ ] Update every parity row with file/test/native evidence and one of the tracker's honest status labels. Updater/signing remain external setup pending; macOS/Linux native behavior unverified on this host. No feature missing implementation may be marked verified. Keep P0 trial counts and unresolved India/provider gate unchanged.
- [ ] Obtain the Native method's one fresh independent whole-execution review against the recorded base, with the Review Focus list and ledger rulings. Address supported Important/Critical findings in one RED→GREEN fix pass and rerun affected suites; ledger minor deferrals. Do not use an empty diff against current `main`.
- [ ] Commit verified changes and evidence, push already-authorized project updates, confirm remote SHA and actual CI status or exact access limitation. Leave a stopped preview if requested. Migration completion is separate from P0 feasibility completion; resume P0 only after this handoff and preserve its evidence ledger.

## Self-review and coverage

The source parity inventory maps to Tasks 1/7 for tooling/agents/docs; Task 2 for typed services and plugins; Tasks 3/4 for shell/state/preferences/i18n/commands; Task 5 for quick pane/platform/window lifecycle; Task 6 for notifications/logging/recovery/updates; Task 8 for native evidence and final review. All four skills and two roles are listed. Every Review Focus class has explicit tests in its owning task. Existing probe semantics, dependency pins, CSP/privacy, license notice, inactive release setup and human-computer availability are global constraints. New signatures are defined before consumers; generated DTOs remain distinct from private native tokens.

## Execution handoff

The written design and implementation plan were approved on 23 October 2026. **Execution is in progress using the retained Native/current-checkout method.** Do not ask for the execution method again. The execution ledger records completed tasks and evidence; the parity tracker distinguishes actual migration progress from external/native acceptance gates.
