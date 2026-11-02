# Tauri template audit and parity tracker

Audited on 23 October 2026. Source: [dannysmith/tauri-template at 1295f691](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c). The revision was confirmed with `git ls-remote` and a local inspection checkout. No upstream installation, release script, or app was run during the audit.

The user selected **the full template shell while preserving Texpio's existing engine**, with Bun replacing pnpm/npm. Implementation is ported/adapted; this table distinguishes automated evidence from native and external acceptance. No blanket 100% runtime parity is claimed. Configuration, browser tests, Windows native acceptance, and other-platform acceptance are distinct.

## Runtime and architecture

Paths below refer to the pinned upstream source.

| Area                       | What the source actually provides                                                                 | Texpio migration target                                                       | Current parity                               |
| -------------------------- | ------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | -------------------------------------------- |
| Shell                      | Resizable left/main/right layout in `src/components/layout/`                                      | Adopt full shell; keep probe in Developer view                                | implemented, native check pending — `MainWindow.tsx` navigation/layout tests |
| Command palette            | Searchable grouped cmdk launcher, keyboard navigation, availability/errors                        | Port and brand; platform-correct shortcut labels                              | implemented, native check pending — `CommandPalette.test.tsx`, shared registry |
| Command registry           | Registry and navigation/window/notification actions in `src/lib/commands/`                        | Route menus, palette and local shortcuts consistently through it              | verified — `commands/commands.test.ts`, closed results/availability |
| Native menus               | App and View menus; rebuilt after language changes                                                | Port appropriate native menus; no fake document Open/Save actions             | implemented, native check pending — `menu.test.ts`, translated in-place ownership |
| Local shortcuts            | Platform-aware keyboard handler                                                                   | Preserve shortcuts and avoid duplicate handling in editable fields            | implemented, native check pending — `use-keyboard-shortcuts.test.tsx`, exact modifier/editable/repeat guards |
| Quick pane                 | Second HTML entry; hidden floating window; monitor positioning; Enter submit; Escape/blur dismiss | Port as explicit app text-entry UI; never imply external-app insertion        | implemented, native check pending — dual HTML build, `QuickPaneApp.test.tsx`, native cfg branches |
| Global quick-pane shortcut | Saved shortcut; default `CommandOrControl+Shift+.`; registration/rebinding                        | Port; conflicts must not abort app or discard a working binding               | implemented, native check pending — `shortcut_tests.rs` conflict/rollback/concurrency fake backend |
| Quick-pane submission      | Updates demo `lastQuickPaneEntry`; no expansion into another app                                  | Keep explicit quick-entry behavior without content logging                    | implemented, native check pending — Enter/cancel/failure tests; transient store only |
| Preferences                | App-data JSON; theme, language, quick-pane shortcut; defaults and save/load hooks                 | Port real persisted fields; integrate Texpio diagnostics/settings             | implemented, native check pending — `service_tests.rs`, `preferences.test.ts`, dialog tests; atomic/error paths |
| Example settings           | General/Advanced demo fields are not persisted features                                           | Do not present those controls as functioning product settings                 | verified (adapted) — source demo-only General/Advanced controls omitted; real theme/language/shortcut controls provided |
| Sidebars                   | Collapsible/resizable empty sidebars                                                              | Port; add persisted visibility/widths to meet README's persistence claim      | implemented, native check pending — normalized persisted layout, `ui-store.test.ts` and `MainWindow.test.tsx` |
| Theme                      | System/light/dark; saved preference; cross-window event                                           | Port and migrate existing `texpio.theme`; keep live OS changes consistent     | implemented, native check pending — theme tests, legacy migration, live OS/cross-window native rereads |
| i18n/RTL                   | English/French/Arabic; saved/system fallback; document language/direction                         | Port, translate new shell/probe labels, retain logical CSS properties         | implemented, native check pending — en/fr/ar, `language.test.ts`, System resolution, logical layout |
| In-app notifications       | Sonner helpers and demo commands                                                                  | Port with content-free probe messages                                         | verified — Sonner helpers, `diagnostics.test.ts` native/toast/unavailable outcomes |
| Native notifications       | Rust command/plugin and fallback path                                                             | Port; packaged Windows behavior needs separate acceptance                     | implemented, native check pending — bounded Rust command; packaged delivery untested |
| Updates                    | Check/download/install/relaunch flow and plugin                                                   | Port with explicit unconfigured state; no live feed/key yet                   | configured, external setup pending — `updater.test.ts`, `UpdateDialog.test.tsx`; no feed/key/startup checks |
| Rust logging               | Stdout; Webview except Linux; macOS log directory                                                 | Port lifecycle logging without snippet/input/clipboard/native-target payloads | verified — closed diagnostic DTO, exact native target filter, bounded rotation, no Webview target |
| Frontend logging           | Development console logger; production forwarding commented out                                   | Preserve honest scope; never log quick-entry/probe text                       | verified — fixed event/count adapter; redaction tests; no user content |
| Error boundary             | Fallback/retry/reload and emergency diagnostic save                                               | Port with accurate saved-diagnostics wording                                  | verified — actual-save feedback and safe restart tests; native rendering/restart pending |
| Recovery API               | Bounded JSON save/load/cleanup; filename validation                                               | Port API and privacy boundary; no unsaved-editor recovery claim               | verified — bounded/path-safe atomic commands and failure tests; crash metadata only |
| Window state               | Position/size/state plugin; quick pane denylisted                                                 | Port while retaining close-to-tray and usable startup restore                 | implemented, native check pending — size/position/maximized/fullscreen only; quick pane/visibility excluded |
| Platform title bars        | Windows controls; macOS traffic lights/vibrancy; Linux native toolbar                             | Port branches; Windows first, other native acceptance pending                 | implemented, native check pending — macOS window rendered on 13.7.8 x64; Linux runtime unverified |
| Typed commands             | tauri-specta command collection and generated TypeScript                                          | Port, including Texpio metadata/probe DTOs; retain runtime validation         | verified — Specta exporter plus validating typed facades and existing probe boundary tests |
| React Compiler             | Production Vite compiler integration                                                              | Port using supported Vite 8 API in app/tests                                  | verified — Babel/Rolldown configuration; both production entries build |
| UI state                   | Zustand selectors                                                                                 | Port store; keep transient native target/input exclusively Rust-owned         | verified — Zustand selectors and layout-only persistence tests; no native tokens/typed input |
| Persistent data hooks      | TanStack Query/preferences/devtools                                                               | Port; do not treat transient probe status as persisted domain data            | verified — Query native preferences, serialized saves, live sync and cancellation tests |
| Context menus              | Native custom/edit/text-edit utilities                                                            | Port utilities with user-gesture usage                                        | implemented, native check pending — `context-menu.test.ts` availability/labels/cancel/disposal |
| UI components              | Reusable shadcn/Radix collection                                                                  | Port/adapt library into one consistent component system                       | verified — full source collection consolidated with existing primitives; React19 ref/calendar adaptations |
| Supporting plugins         | FS, persisted scope, dialog, clipboard, opener, OS, process                                       | Include relevant infrastructure with narrowly scoped capabilities             | implemented, native check pending — all source plugin backends pinned/registered; narrow capabilities retained |
| Tray and engine            | Absent upstream                                                                                   | Preserve Texpio Open/Pause/Stop/Quit and existing engine shutdown             | implemented, native check pending — tray/lifecycle tests retained; prior-build user Open/Quit evidence does not qualify this binary |

## Development and operations

| Area                   | Source reality                                                       | Texpio migration target                                                                      | Current parity                        |
| ---------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------- |
| Package management     | npm; package lock; npm instructions                                  | Bun exact version and committed text `bun.lock`; no mixed active lockfiles                   | verified — project Bun1.4.4, text lock, frozen install; Node24.18.0/Rust1.99.1 retained |
| Cross-platform scripts | Several Rust scripts use POSIX `source`                              | Portable Cargo `--manifest-path` scripts, locked verification                                | verified — portable manifest-path/locked Cargo scripts and Bun entry points |
| Aggregate checks       | TypeScript, ESLint, ast-grep, formatting, Rust and Vitest            | Working `bun run check:all`, including existing fixture test                                 | verified — check:all includes lint/fixtures/frontend/tooling/Rust, no hooks/send input |
| Static rules           | ESLint/Prettier plus three ast-grep rules                            | Port rules and test their negative/positive cases                                            | verified — six parser-specific rule fixtures cover three policies in TypeScript/TSX |
| Cleanup analysis       | Knip/jscpd scripts/config                                            | Port with both HTML entries/generated bindings accounted for                                 | verified — both entries/generated exclusions; reports investigated, intentional library retained |
| Test modes             | Vitest run/watch/coverage; UI script lacks its dependency            | Match Vitest 5 UI/coverage packages; preserve native-free tests                              | verified — matched Vitest5 UI/coverage dependencies, run/watch/coverage scripts |
| Bundle analysis        | Build plus printed hint, not an analyzer                             | Preserve meaningful build/size report; label actual capabilities                             | verified — build plus emitted asset byte report; no interactive treemap/performance claim |
| Task helpers           | Complete/rename task documents                                       | Adapt to canonical `TASKS.md` and docs; no automatic completion claims                       | verified — exact/unique task selection, traversal/junction/collision refusal, archive normalization tests |
| Release helper         | Version writes and optional commit/tag/push                          | Separate preparation from explicit publishing; no accidental push                            | verified — stable local version preparation/dry-run; no subprocess/signing/Git publication API |
| Release workflow       | Three OS targets; draft releases/updater artifacts; floating actions | Adapt, pin actions/toolchains; signing/feed configuration still required                     | configured, external setup pending — 64-bit RustDesk-style artifacts (NSIS+MSI, universal DMG, deb+AppImage); manual opt-in/draft-only, pinned actions, real signing/feed gates; not dispatched |
| Developer docs         | 24 guides plus docs index/setup/security/contributing/user guide     | Import/adapt; preserve existing Texpio plans and evidence                                    | verified — 30/30 source topics adapted; 101 local links resolve; existing product plans preserved |
| Editor/review config   | VS Code recommendations/settings, Cursor ignore, CodeRabbit config   | Adapt useful settings; external review services remain opt-in                                | verified — workspace settings, maintained recommendations, Cursor ignores; external automatic review disabled |
| Upstream license       | MIT, copyright 2025 Tauri Template Contributors                      | Retain complete notice and source revision; do not silently license all original Texpio code | verified — complete MIT notice and pinned source attribution; original product license undecided |

## Portable agent inventory

There are exactly **four skills and two agent prompts**, with no commands directory, settings file or hooks in the pinned `.claude` tree.

| Source                                           | Destination                                      | Adaptation                                                                              |
| ------------------------------------------------ | ------------------------------------------------ | --------------------------------------------------------------------------------------- |
| `.claude/skills/check/SKILL.md`                  | `.agents/skills/check/SKILL.md`                  | Bun checks, real scripts and portable tool wording                                      |
| `.claude/skills/init/SKILL.md`                   | `.agents/skills/init/SKILL.md`                   | Explicit request only; protect existing Texpio identity/data; correct root `index.html` |
| `.claude/skills/cleanup/SKILL.md`                | `.agents/skills/cleanup/SKILL.md`                | Real task paths; supported delegation or inline fallback                                |
| `.claude/skills/change-package-manager/SKILL.md` | `.agents/skills/change-package-manager/SKILL.md` | Plain arguments, `.agents` references, safe lock migration and actual workflows         |
| `.claude/agents/cleanup-analyzer.md`             | `.agents/agents/cleanup-analyzer.md`             | Portable read-only role; remove Claude-only metadata                                    |
| `.claude/agents/userguide-reviewer.md`           | `.agents/agents/userguide-reviewer.md`           | Discover actual store/locale paths; portable report-only role                           |

Root `AGENTS.md` links a `.agents/README.md` index. Preserve the existing computer-availability rules and dependency provenance. A thin `CLAUDE.md` points to `AGENTS.md`; there is no duplicate canonical `.claude` skills. Storage in `.agents` does not guarantee every client automatically registers role prompts, permissions or slash commands.

## Important differences from the README

- The source sidebars do not persist layout, despite the README claim.
- The emergency API exists, but there is no unsaved-work snapshot/restore integration. The error boundary saves crash context.
- The source's native menu is App/View, not File/Edit/View; some menus/shortcuts bypass the command registry.
- Updater endpoint/public key are placeholders. The release workflow is infrastructure, not evidence of working signed updates.
- Supporting FS/dialog/clipboard/opener plugins are installed infrastructure, not completed user workflows. The quick pane never inserts into another app.
- The actual lock uses Vite 8/TypeScript 6, despite README/agent references to Vite 7. Vitest 4 and ESLint 9 differ from Texpio's newer versions; the source test compiler configuration also needs adaptation.

## Completion labels

Use **verified**, **implemented, native check pending**, **configured, external setup pending**, **other platform unverified**, or **not implemented** in the final parity table. Never make a blanket 100% parity claim while signing, native platform checks, or code features remain open. P0 engine qualification and its 300 physical-input trials remain a separate gate.

## Verification record — 24 October 2026

Automated results, binding repeatability, binary build, remote commit and CI outcome are recorded below after the final checks. The command suite never starts the P0 observer or sends input. Source-based skill pressure cases and deterministic mapping checks pass.

Computer Use was not started for this migration. The available Windows API documentation exposes no supported control-session stop/close operation, while AGENTS.md requires proving owned-session closure. To preserve the user's computer availability, new-binary native interaction checks remain explicitly pending: chrome/menu/palette/editable keys; settings/layout persistence after restart; theme/language/RTL and quick-pane sync; real shortcut conflict/rebinding/Enter/Escape/blur and monitor/DPI placement; hide/minimize/second instance; tray Open/Pause/Stop/Quit; clean Quit/relaunch with Stopped engine. Packaged notifications and macOS/Linux runtime remain unverified.

P0 remains separate: no insertion strategy qualified, 0/300 physical trials, unresolved English (India) provider/layout gate, clipboard experiments still disabled. No accounts/billing/sync service or snippet editor/database was added. Real signed update delivery, certificate/notarization, release environment protection and platform acceptance require external setup; no signing keys, tags, release or workflow dispatch were created.

Final automated evidence after fixes: full checks passed 118 frontend/77 Rust/11 tooling/1 fixture, six ast-grep groups; coverage passed 73.49% statements/70.57% branches/68.8% functions/74.67% lines. Two native binding exports leave no diff. Both frontend entries build. The independent review Important finding is fixed with RED-to-GREEN tests; the final preview and production rebuild are recorded below.

Cleanup reports were inspected: Knip returned exit 1 for retained reusable UI/tooling dependencies, helpers and configuration hints; jscpd returned exit 0 for intentional refusal-mapping/root-callback patterns. These reports did not trigger automatic deletion. Frozen installation completed without lock changes.

## macOS runtime record — 1 November 2026

The macOS runtime is no longer entirely unexecuted. On an Intel host running macOS
13.7.8 the release bundle built with `--bundles app --locked --no-default-features`
and launched; the window, library and Preferences shell rendered. Two adapter
defects that prevented any expansion on this host were fixed and re-observed:

- the US/ABC-only input-source allowlist refused `com.apple.keylayout.ABC-India`
  and reported `unavailable`; the gate now accepts ASCII-capable keyboard layouts
  and derives characters with `UCKeyTranslate` plus dead-key state;
- the default Paste + clipboard-restoration pair is refused by design, so every
  start failed; the settings store now clears restoration for macOS Paste, the
  switch is disabled with an explanation, and startup reports `invalidSettings`.

A non-fullscreen in-app permission prompt (plus the matching Settings card) now
lists the missing grant with a deep link; the link was observed opening System
Settings on the Accessibility pane. Windows-specific error copy in the banner was
replaced with platform-neutral text.

The permission gate now requires Accessibility only. The Input Monitoring pane was
empty on this host, while an Accessibility-trusted probe reported listen and post
access as true and created working HID and session event taps; the same probe in a
fresh, untrusted bundle reported both false and had both taps denied. Input
Monitoring is the grant macOS uses for processes that are *not* Accessibility
trusted, so it is reported for diagnosis and shown only when listening is still
withheld.

Remaining macOS gaps are recorded in the
[native acceptance record](native-platform-acceptance.md): no grant was issued, so
recognition, insertion, context evidence in a real editor, Unicode/newlines,
clipboard races, shutdown and ARM64 are unexecuted, and packaging/signing is
unverified. Task8 native acceptance therefore remains open; this section records
partial runtime evidence only.

## Independent review and fixes

A read-only gpt-6-astra review covered `c7b4038..71a6425` across the complete migration, including the five Review Focus classes. It found no Critical issues, one Important cancellation race, and two Minor items. The cancellation fix has six RED-to-GREEN deferred-promise regressions covering blur/Escape, late success/failure, reopening and a newer pending submission. Canceled UI text cannot be restored by stale continuations; an emission already submitted by Enter cannot be retracted from the event bus.

Final checks also exposed intermittent Windows Access denied (5) at atomic file replacement. A held delete-sharing lock reproduces that error class; the identity of the original interfering process is unknown. Replacement now retries eligible Windows errors at most five times with 10ms pauses, retaining the staged file. Three deterministic native regressions prove short-lock recovery, bounded persistent-lock failure and read-only refusal without losing previous bytes. Five complete native regression runs passed77 tests each. No raw error/path diagnostics were retained.

Deferred minors: hidden sidebar remembered widths can constrain a visible sidebar resize; custom Windows/macOS control labels/tooltips remain English in French/Arabic. Both are usability/localization polish, not represented as fixed.

Reviewer exclusions remain open gates: actual Windows focus/chrome/DPI/shortcut/tray/packaged notifications; macOS/Linux runtime/packaging; signed updates/secrets/environment protection/publication; and P0 physical/India/clipboard/picker qualification. Their cost is unverified behavior or unavailable capability in the respective area. No automatic check substitutes for these observations. Task8 native acceptance remains open.

## Implementation decisions and their costs

These are the complete execution rulings, in decision order. The two deferred minor findings above remain unfixed.

| Decision | Reason | Cost or remaining risk |
| --- | --- | --- |
| Qualify project-local Bun 1.4.4; leave global Bun unchanged | Installed Bun 1.3.13 could not migrate the existing pnpm lock format | Development must use the documented pinned binary or install that version |
| Retain ESLint 10 with supported community React and hooks plugins | The source plugins had incompatible peers; preserve JSX/Compiler safety categories | Rule categories are equivalent, not an identical upstream rule set |
| Give application-specific chords one DOM owner; custom native menu items have no duplicate accelerators | Preserve editable/composition/repeat guards and avoid duplicate execution; native Edit roles retain OS ownership | Application chords need webview focus; the global quick-pane shortcut is separate |
| Let the native shortcut worker own its persisted field; appearance saves patch only their fields and reread | Prevent concurrent appearance changes from bypassing registration or overwriting a binding | Shortcut changes require their dedicated command; appearance saves add one IPC read |
| Keep detailed Windows native acceptance open | Review and automated checks cannot establish actual focus/chrome/DPI/conflicts/tray/second-instance or packaged delivery | Native interaction defects may remain; basic opening confirmation covers only the shell and launchers |
| Keep macOS/Linux runtime and packaging unverified | They were not executed on this Windows host | Target-specific compilation or interaction defects may remain |
| Record macOS runtime evidence without closing the native gate | A macOS 13.7.8 x64 build, launch, permission UI and deep links were observed on 1 November 2026 | No grant was issued, so expansion itself, packaging and ARM64 remain unverified |
| Keep signed delivery and publication behind external setup gates | No real signing/feed/secrets/environment protection acceptance or release dispatch | Updates remain unavailable until setup and end-to-end qualification |
| Keep P0 physical/layout/provider/clipboard/picker qualification separate | No insertion strategy passed its evidence gate; 0/300 physical trials and clipboard disabled | The foundation is not yet qualified for everyday text expansion |

## Preview and remote evidence

The Windows debug preview built successfully from implementation commit `1aae8e16cba333736a3c9c7ca09cba056ec3865b` and was launched normally with no Computer Use session. Startup remained running as PID10604 with a Texpio main window and no stderr output. Preview SHA256: `FC204FA4254EA7ADFFD99656B12F46B155B75FAFBDC8CDD03CD88CD5BCF9B535`.

The user confirmed the new Home/Developer/Preferences shell and both the command palette and Ctrl+Shift+Period quick-pane launcher work. This is basic opening acceptance only. The remaining detailed Windows checks listed above remain open, as do packaged notifications, macOS/Linux runtime and external release setup. No probe was armed by the agent; physical qualification remains0/300.

The final production command `bun run tauri:build --no-bundle -- --locked` passed from the same implementation commit. Rust's optimized build finished in 7m 14s and produced `src-tauri/target/release/texpio.exe`, SHA256 `5FCAEC04BEDA99D877BCB71581A95BAF42CB62E3CAA3681869F0F309A0E36052`. No installer, signature or release publication was produced. The user's basic opening confirmation applies to the debug preview, not an observed production-executable interaction test.

Implementation pushed to `origin/main`; `git ls-remote` matched `1aae8e16cba333736a3c9c7ca09cba056ec3865b`. The GitHub connector's Actions query for this exact SHA returned404 NOT_FOUND. CI is unverified, not passing by inference. No release workflow was dispatched. Final documentation may advance main beyond this runtime implementation commit.
