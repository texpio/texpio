# Full template adoption and Bun migration

Status: **written design awaiting user approval**. The user selected the full template shell while preserving the existing engine on 23 October 2026. This document makes the migration reviewable; implementation planning follows approval.

## Intent and source

Make Texpio inherit the full reusable application foundation from [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c), pinned at `1295f691bbdf373896e7c0f1cc969e2040fbf42c`, instead of building those systems individually. Use Bun for package management and scripts. Store portable project skills/role prompts in `.agents`, merge upstream agent guidance into `AGENTS.md`, and deliver an evidence-based feature parity table.

The detailed [source audit and parity tracker](../../research/tauri-template-parity.md) is the scope checklist. Its distinctions between implemented code, examples and configuration placeholders are binding: copying a placeholder is not a working feature.

## Approaches considered

1. **Full shell adoption with engine integration — selected by the user.** Port the upstream layout, state/services, preferences, command system, quick pane and supporting infrastructure together. Keep Texpio identity, existing Rust engine, tray lifecycle, planning documents and verification history. This is a substantial foundation migration, but avoids maintaining two competing application architectures.
2. **Individual feature ports into today's foundation screen.** Smaller initial diffs, but many architectural seams and duplicated state/menu/window infrastructure. The user did not select this approach.

A wholesale checkout replacement is unsuitable: it would discard the engine and Texpio-specific lifecycle/privacy rules. Import from the pinned source selectively by file responsibility, with the full feature inventory accounted for.

## Existing work and invariants

- Keep `io.github.texpio.texpio`, Texpio branding, version history, current repository and existing original icons unless deliberately regenerated for Texpio.
- Preserve the complete Rust engine and validated probe boundary. The engine starts Stopped, can arm only after explicit Start, keeps target/input state in Rust, refuses uncertain contexts, and stops on pause/quit/fault. Direct mode contains real native dispatch; it is not globally compile-disabled. Clipboard insertion remains disabled. The unresolved India-layout refusal and **0/300** physical trials stay recorded, with no native qualification claim.
- Preserve Open/Pause/Stop/Quit tray controls, close-to-tray, unminimize/show/focus restoration, single-instance initialization first, and `ProbeHandle.stop()` on exit/update/relaunch paths.
- Keep Windows first. Import portable macOS/Linux shell branches and configurations, but do not imply those platforms or native expansion adapters are qualified.
- No accounts, billing, sync backend, snippet database/editor or new expansion behavior in this migration. The separate existing Laravel/PostgreSQL service remains the future backend.

## Application architecture

Use upstream's main/quick-pane HTML entries, platform chrome, resizable sidebars, command palette, native menus, preferences panes, i18n/RTL, notifications, error boundary, query provider and UI store. The original foundation/probe becomes a **Developer** view reachable from navigation and the command palette, rather than the entire main application.

Port the reusable shadcn component collection into one consistent system, retaining semantic theme tokens and accessible composition. Keep existing fixture/probe tests while adapting providers and imports. Do not introduce duplicate components for the same role merely to avoid resolving import conflicts.

Use component state for local UI, Zustand selectors for shared UI, and TanStack Query for persisted Rust preferences. Normal application actions route through a common command registry so palette, local shortcuts and native menus behave consistently. Native editing commands remain platform-native. Do not invent document Open/Save workflows without an editor.

Persist actual theme/language/quick-pane shortcut settings. Add small persisted sidebar visibility/width state to satisfy the README's layout-persistence feature. Keep nonfunctional General/Advanced examples out of normal product controls. Migrate the existing `texpio.theme` preference once, preserving the user's selection; all windows follow later theme/OS changes.

Port English/French/Arabic dictionaries and direction handling. New Texpio shell/probe strings participate in the same translation system. Unknown/browser platform detection must not pretend to be macOS; native-only actions retain an explicit unavailable state.

The template quick pane is a text-entry window with an app event, **not** a captured-target snippet insertion picker. Retain its Enter/Escape/blur behavior, monitor positioning and shortcut. It may display the user's explicit entry inside Texpio, but never logs it or calls the expansion engine. P0's captured-target insertion experiment remains separate. Shortcut conflicts must leave the application usable and preserve a previous valid binding where possible.

## Rust bridge, capabilities and privacy

Port the typed tauri-specta bridge using its exact mutually compatible Specta versions. Add Texpio metadata/probe commands to the collector and derive only serializable public DTOs. Native target tokens, HWNDs, process handles, matching suffixes and captured text never become generated public types. Keep runtime validation, safe-integer counter checks, unknown-field stripping and closed refusal codes at the existing frontend probe boundary; generated types do not replace those protections.

Include the template's relevant plugin infrastructure with capabilities scoped to the windows and actions that need it. The quick pane gets no filesystem/clipboard/updater powers merely because the main window has a plugin. Preserve Texpio's restrictive production CSP; do not import inline-script or arbitrary remote-image permissions for unused examples. Installed clipboard infrastructure never becomes an expansion fallback.

Port structured lifecycle logging without logging snippet bodies, quick entries, captured text, clipboard data, matching input or native target identity. Keep the source's Linux logging workaround. Port bounded emergency JSON save/load/cleanup and accurate error-boundary feedback. This provides diagnostic recovery infrastructure; it does not promise restoration of unsaved editor work that Texpio does not yet have. Do not copy raw OS error/path payloads into the probe UI or diagnostics.

## Bun and toolchain

Use Bun as the sole active package manager/script launcher, with text `bun.lock`, an exact `packageManager`/version pin, Tauri hooks, CI, docs and agent instructions aligned. Start qualification with the already installed **Bun 1.3.13**. If actual migration/CLI checks show incompatibility, select and pin a tested stable version explicitly; do not silently upgrade the host or mix lockfiles. Keep the old lock until conversion and version/integrity checks pass, then remove it in the migration commit. Require both an existing lock and a successful frozen install in CI.

Retain Node `24.18.0` for tools with Node shebangs and the Node fixture test; retain Rust `1.99.1`, Vite 8, TypeScript 6 and Vitest 5. Use `bun run test` to launch Vitest, not Bun's different `bun test` runner; do not force every tool onto the Bun runtime with `--bun`.

Adapt React Compiler to the installed Vite/plugin API in both app/test configurations. Match Vitest UI/coverage packages to the core version. Resolve ESLint React-plugin peer compatibility rather than suppressing it: prefer supported rules on current tooling; if a tool version change is necessary, record the exact reason and affected checks. Replace POSIX `source` scripts with portable Cargo commands and preserve locked native checks.

Port ast-grep architecture rules, Knip, jscpd, useful editor settings and task/release preparation helpers. `bun run check:all` must be a real cross-platform aggregate check, including Texpio fixture coverage. Keep destructive fix/publish helpers outside that command. Inspect lifecycle scripts and trust only packages needed by the validated build; do not blindly trust all dependencies.

## Portable agent materials and documentation

Port all four `.claude/skills/*/SKILL.md` files to `.agents/skills/` and both role prompts to `.agents/agents/`, using the mapping in the parity tracker. Add a portable index linked from root `AGENTS.md`. Preserve purpose and substantive guidance while removing Claude-specific tool names, `$ARGUMENTS`, nonexistent paths and implicit slash-command/permission assumptions. `init` requires an explicit request and must not reinitialize this existing project.

Merge useful upstream architecture/i18n/typed-command guidance into `AGENTS.md`. The user's Bun decision overrides both upstream npm-only and current pnpm-only rules. Preserve the complete computer-availability section and installed dependency-skill provenance. Use normal capability-aware documentation lookup, not an unconditional dependency on unavailable tools. Do not copy `rm -f` or the upstream prohibition on agents running development servers. A thin `CLAUDE.md` may point to the shared root instructions; no duplicate canonical `.claude` skills or nonexistent local imports.

Import/adapt the 30 upstream Markdown guides and useful configuration files without overwriting Texpio's existing plans/evidence. Keep `TASKS.md` canonical; link detailed task helpers/documents to it. Correct placeholder branding, paths and feature claims. Preserve the complete upstream MIT notice and source revision in third-party attribution; this does not silently settle the license of all original Texpio code.

## Updates and releases

Port update UI/service and release tooling, but default to an explicit **not configured** state until a real endpoint, public signing key and release setup exist. No checks against `YOUR_*` URLs, automatic network requests to placeholder feeds, signing-key generation, tag push, workflow dispatch or publication during migration. User confirmation is required before any later update installation/relaunch; the probe stops before those actions.

Adapt the three-platform release workflow with pinned actions/toolchains and Texpio metadata. It remains opt-in/prepared infrastructure until signing and platform acceptance exist. Ordinary CI remains non-injecting; preserve Linux frontend and Windows native checks. No external review service is invoked merely because a config file was copied.

## Validation and completion

Use focused behavior tests for command consistency, preference and layout persistence, theme migration/cross-window changes, shortcut failure/rollback, error redaction, unconfigured updates and stopping before exit/relaunch. Validate skill manifests, referenced files, portability and the complete six-file provenance mapping. Preserve existing engine/bridge regression tests; ordinary tests never install hooks or send input.

Run clean/frozen Bun installation, typecheck, lint, formatting, ast-grep, frontend/fixture tests, production dual-entry build, generated-binding consistency, Cargo fmt/strict Clippy/tests, and a Windows no-bundle build. Run Knip/jscpd and distinguish intended library exports from findings. Check repository docs/links and obsolete active package-manager commands.

Native Windows acceptance covers titlebar/menu/window state, command palette, actual settings persistence/relaunch, quick-pane shortcut/rebind/conflict/Enter/Escape/blur, multi-window themes, and the preserved tray/single-instance lifecycle. Keep UI-control sessions brief and close them immediately; no Computer Use during builds/waits. Report packaged notification/update/signing and macOS/Linux limits separately.

Deliver the updated parity table with per-row implementation and evidence statuses, commit/push authorized changes, verify remote SHA/CI where accessible, and leave a stopped-by-default preview if requested. Do not label the migration fully verified while any required code, native check or external setup is incomplete. Resume the separate P0 layout/provider investigation afterward.

## Builder discretion and review

The builder may settle module names, small component composition, exact compatible dependency patch versions and test organization. Any substantive source deviation must have a stated reason and a parity row. The broad feature scope, privacy/lifecycle boundaries, Bun selection and full-shell approach above are not discretionary.

A fresh builder review checks this written design before user approval. Only after approval will a separate implementation plan define the execution tasks and checks; the existing Native/current-checkout execution preference is retained.
