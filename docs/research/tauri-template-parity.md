# Tauri template audit and parity tracker

Audited on 23 October 2026. Source: [dannysmith/tauri-template at 1295f691](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c). The revision was confirmed with `git ls-remote` and a local inspection checkout. No upstream installation, release script, or app was run during the audit.

The user selected **the full template shell while preserving Texpio's existing engine**, with Bun replacing pnpm/npm. This table is the migration checklist, not a completed-parity claim. Update the final column with implementation and actual verification evidence as work lands. Configuration, browser tests, Windows native acceptance, and other-platform acceptance are distinct.

## Runtime and architecture

Paths below refer to the pinned upstream source.

| Area                       | What the source actually provides                                                                 | Texpio migration target                                                       | Current parity                               |
| -------------------------- | ------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | -------------------------------------------- |
| Shell                      | Resizable left/main/right layout in `src/components/layout/`                                      | Adopt full shell; keep probe in Developer view                                | Planned                                      |
| Command palette            | Searchable grouped cmdk launcher, keyboard navigation, availability/errors                        | Port and brand; platform-correct shortcut labels                              | Planned                                      |
| Command registry           | Registry and navigation/window/notification actions in `src/lib/commands/`                        | Route menus, palette and local shortcuts consistently through it              | Planned                                      |
| Native menus               | App and View menus; rebuilt after language changes                                                | Port appropriate native menus; no fake document Open/Save actions             | Planned                                      |
| Local shortcuts            | Platform-aware keyboard handler                                                                   | Preserve shortcuts and avoid duplicate handling in editable fields            | Planned                                      |
| Quick pane                 | Second HTML entry; hidden floating window; monitor positioning; Enter submit; Escape/blur dismiss | Port as explicit app text-entry UI; never imply external-app insertion        | Planned                                      |
| Global quick-pane shortcut | Saved shortcut; default `CommandOrControl+Shift+.`; registration/rebinding                        | Port; conflicts must not abort app or discard a working binding               | Planned                                      |
| Quick-pane submission      | Updates demo `lastQuickPaneEntry`; no expansion into another app                                  | Keep explicit quick-entry behavior without content logging                    | Planned                                      |
| Preferences                | App-data JSON; theme, language, quick-pane shortcut; defaults and save/load hooks                 | Port real persisted fields; integrate Texpio diagnostics/settings             | Planned                                      |
| Example settings           | General/Advanced demo fields are not persisted features                                           | Do not present those controls as functioning product settings                 | Adaptation required                          |
| Sidebars                   | Collapsible/resizable empty sidebars                                                              | Port; add persisted visibility/widths to meet README's persistence claim      | Planned                                      |
| Theme                      | System/light/dark; saved preference; cross-window event                                           | Port and migrate existing `texpio.theme`; keep live OS changes consistent     | Partial: main-window theme exists            |
| i18n/RTL                   | English/French/Arabic; saved/system fallback; document language/direction                         | Port, translate new shell/probe labels, retain logical CSS properties         | Planned                                      |
| In-app notifications       | Sonner helpers and demo commands                                                                  | Port with content-free probe messages                                         | Planned                                      |
| Native notifications       | Rust command/plugin and fallback path                                                             | Port; packaged Windows behavior needs separate acceptance                     | Planned                                      |
| Updates                    | Check/download/install/relaunch flow and plugin                                                   | Port with explicit unconfigured state; no live feed/key yet                   | External configuration pending               |
| Rust logging               | Stdout; Webview except Linux; macOS log directory                                                 | Port lifecycle logging without snippet/input/clipboard/native-target payloads | Planned                                      |
| Frontend logging           | Development console logger; production forwarding commented out                                   | Preserve honest scope; never log quick-entry/probe text                       | Planned                                      |
| Error boundary             | Fallback/retry/reload and emergency diagnostic save                                               | Port with accurate saved-diagnostics wording                                  | Planned                                      |
| Recovery API               | Bounded JSON save/load/cleanup; filename validation                                               | Port API and privacy boundary; no unsaved-editor recovery claim               | Planned                                      |
| Window state               | Position/size/state plugin; quick pane denylisted                                                 | Port while retaining close-to-tray and usable startup restore                 | Planned                                      |
| Platform title bars        | Windows controls; macOS traffic lights/vibrancy; Linux native toolbar                             | Port branches; Windows first, other native acceptance pending                 | Planned                                      |
| Typed commands             | tauri-specta command collection and generated TypeScript                                          | Port, including Texpio metadata/probe DTOs; retain runtime validation         | Partial: manually typed validated IPC exists |
| React Compiler             | Production Vite compiler integration                                                              | Port using supported Vite 8 API in app/tests                                  | Planned                                      |
| UI state                   | Zustand selectors                                                                                 | Port store; keep transient native target/input exclusively Rust-owned         | Planned                                      |
| Persistent data hooks      | TanStack Query/preferences/devtools                                                               | Port; do not treat transient probe status as persisted domain data            | Planned                                      |
| Context menus              | Native custom/edit/text-edit utilities                                                            | Port utilities with user-gesture usage                                        | Planned                                      |
| UI components              | Reusable shadcn/Radix collection                                                                  | Port/adapt library into one consistent component system                       | Partial: existing component subset           |
| Supporting plugins         | FS, persisted scope, dialog, clipboard, opener, OS, process                                       | Include relevant infrastructure with narrowly scoped capabilities             | Planned                                      |
| Tray and engine            | Absent upstream                                                                                   | Preserve Texpio Open/Pause/Stop/Quit and existing engine shutdown             | Existing; must retain                        |

## Development and operations

| Area                   | Source reality                                                       | Texpio migration target                                                                      | Current parity                        |
| ---------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------- |
| Package management     | npm; package lock; npm instructions                                  | Bun exact version and committed text `bun.lock`; no mixed active lockfiles                   | Planned; installed Bun reports 1.3.13 |
| Cross-platform scripts | Several Rust scripts use POSIX `source`                              | Portable Cargo `--manifest-path` scripts, locked verification                                | Planned                               |
| Aggregate checks       | TypeScript, ESLint, ast-grep, formatting, Rust and Vitest            | Working `bun run check:all`, including existing fixture test                                 | Partial: separate checks exist        |
| Static rules           | ESLint/Prettier plus three ast-grep rules                            | Port rules and test their negative/positive cases                                            | Planned                               |
| Cleanup analysis       | Knip/jscpd scripts/config                                            | Port with both HTML entries/generated bindings accounted for                                 | Planned                               |
| Test modes             | Vitest run/watch/coverage; UI script lacks its dependency            | Match Vitest 5 UI/coverage packages; preserve native-free tests                              | Partial: Vitest 5 tests exist         |
| Bundle analysis        | Build plus printed hint, not an analyzer                             | Preserve meaningful build/size report; label actual capabilities                             | Planned                               |
| Task helpers           | Complete/rename task documents                                       | Adapt to canonical `TASKS.md` and docs; no automatic completion claims                       | Planned                               |
| Release helper         | Version writes and optional commit/tag/push                          | Separate preparation from explicit publishing; no accidental push                            | Planned                               |
| Release workflow       | Three OS targets; draft releases/updater artifacts; floating actions | Adapt, pin actions/toolchains; signing/feed configuration still required                     | Planned; not activated                |
| Developer docs         | 24 guides plus docs index/setup/security/contributing/user guide     | Import/adapt; preserve existing Texpio plans and evidence                                    | Planned                               |
| Editor/review config   | VS Code recommendations/settings, Cursor ignore, CodeRabbit config   | Adapt useful settings; external review services remain opt-in                                | Planned                               |
| Upstream license       | MIT, copyright 2025 Tauri Template Contributors                      | Retain complete notice and source revision; do not silently license all original Texpio code | Required before import                |

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

Root `AGENTS.md` will link a `.agents/README.md` index. Preserve the existing computer-availability rules and dependency provenance. A thin `CLAUDE.md` may point to `AGENTS.md`; there will be no duplicate canonical `.claude` skills. Storage in `.agents` does not guarantee every client automatically registers role prompts, permissions or slash commands.

## Important differences from the README

- The source sidebars do not persist layout, despite the README claim.
- The emergency API exists, but there is no unsaved-work snapshot/restore integration. The error boundary saves crash context.
- The source's native menu is App/View, not File/Edit/View; some menus/shortcuts bypass the command registry.
- Updater endpoint/public key are placeholders. The release workflow is infrastructure, not evidence of working signed updates.
- Supporting FS/dialog/clipboard/opener plugins are installed infrastructure, not completed user workflows. The quick pane never inserts into another app.
- The actual lock uses Vite 8/TypeScript 6, despite README/agent references to Vite 7. Vitest 4 and ESLint 9 differ from Texpio's newer versions; the source test compiler configuration also needs adaptation.

## Completion labels

Use **verified**, **implemented, native check pending**, **configured, external setup pending**, **other platform unverified**, or **not implemented** in the final parity table. Never make a blanket 100% parity claim while signing, native platform checks, or code features remain open. P0 engine qualification and its 300 physical-input trials remain a separate gate.
