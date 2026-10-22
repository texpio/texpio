# Desktop Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a reproducible Texpio Windows desktop shell from the official Tauri starter, ready for the separate native-engine feasibility work.

**Architecture:** One root React/Vite application calls narrow Rust commands in `src-tauri`. Rust owns native window/tray lifecycle. The foundation has no database, keyboard hook, sync, or account dependency.

**Tech Stack:** Tauri 2, Rust/MSVC, React, strict TypeScript, Vite, Tailwind CSS 4, shadcn/ui, Lucide, pnpm, Vitest, React Testing Library, ESLint, Prettier, rustfmt, Clippy, GitHub Actions.

**Spec:** [Approved desktop foundation design](../specs/2026-10-23-desktop-foundation-design.md). Read it together with the [stack record](../../tech-stack.md). Plan review and execution-method selection are pending.

## Global Constraints

- Product name `Texpio`, package names `texpio`, application identifier `io.github.texpio.texpio`.
- Windows x64 development target; stable Tauri 2 and Rust with the Windows MSVC toolchain.
- Preserve the existing Git history and planning documents; keep a single application at the repository root.
- Use private JavaScript packaging and disable Cargo publication; apply no project license during setup.
- Pin tested Node and Rust versions and the exact pnpm version; commit both lockfiles.
- Close hides the main window; the tray menu's Quit action exits the process.
- No backend, database, keyboard hook, clipboard manipulation, launch-at-login registration, or global shortcut.
- No public release, updater feed, signing credentials, or signed installer in this milestone.

## Review Focus

1. Malformed IPC values must become the connection error state, rather than rendering misleading metadata; test in Task 2.
2. Denied localStorage access must preserve a working System theme; test in Task 2.
3. OS theme changes during a running session must affect System mode while preserving explicit Light/Dark overrides; test in Task 2.
4. Native-window absence in a browser preview must produce an honest unavailable state; test in Task 2.
5. A hidden or minimized main window must reopen from the tray and from a second launch, while Quit must terminate the background process; verify on Windows in Task 3.

## File responsibilities

Paths describe the intended final layout. Keep generated support files the pinned official template requires, even if not listed individually here.

| Files | Responsibility |
| --- | --- |
| `package.json`, `pnpm-lock.yaml`, `.node-version`, `rust-toolchain.toml` | Package scripts and reproducible tools/dependencies |
| `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `index.html` | Official Vite/TypeScript application, aliases, development settings |
| `components.json`, `src/styles.css`, `src/components/ui/`, `src/lib/utils.ts` | shadcn configuration, theme tokens, used components and class utility |
| `src/main.tsx`, `src/App.tsx` | React bootstrap and foundation composition |
| `src/features/foundation/FoundationScreen.tsx`, `.test.tsx` | Metadata status and foundation UI |
| `src/lib/desktop.ts`, `src/lib/desktop.test.ts` | Typed Tauri metadata bridge and validation |
| `src/features/theme/ThemeProvider.tsx`, `.test.tsx` | Persisted theme preference and live system-theme handling |
| `src/test/setup.ts`, `vitest.config.ts` | DOM test setup and runner |
| `eslint.config.js`, `.prettierrc.json`, `.prettierignore` | Frontend lint and formatting |
| `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json` | Native dependencies, identity, build configuration and permissions |
| `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs` | Native entry, registration and metadata contract |
| `src-tauri/src/desktop.rs` | Tray, window restoration, close and quit handling |
| `.github/workflows/checks.yml`, `README.md`, `TASKS.md`, `docs/development.md` | CI, developer instructions and verification record |

## Task 1: Reproducible official starter

**Files:** Generate the root Vite/React and `src-tauri` starter files; create the toolchain pins, JavaScript and Cargo lockfiles, and initial `docs/development.md`. Preserve the existing `.gitignore`, `.gitattributes`, README content, and `docs/`.

**Interfaces:** Consumes the existing documentation-only repository. Produces `pnpm dev`, `pnpm build`, and `pnpm tauri dev/build` backed by the official starter, plus pinned toolchains for later tasks.

- [ ] Verify the clean checkout and Git context, then resolve the execution environment using the worktree skill. Do not create a worktree without its required consent. Keep unrelated changes out of the implementation.
- [ ] Locate or install missing prerequisites through official distribution channels: Rust/MSVC and the Visual Studio C++ desktop workload including a Windows SDK. Reuse the installed WebView2. This entails machine-level developer tooling and potentially a multi-gigabyte download. Do not reboot automatically. Verify `rustc -V`, `cargo -V`, the installed MSVC toolchain, and the SDK/build-tool registration.
- [ ] Verify the existing Node `v24.18.0` and select a compatible stable pnpm release. Pin both in `.node-version` and `package.json`; resolve an exact stable Rust version into `rust-toolchain.toml` with rustfmt/Clippy. Record the versions in `docs/development.md`.
- [ ] Resolve an exact official `create-tauri-app` generator version, inspect its CLI help, and generate the React/TypeScript/pnpm template in a temporary staging directory. Record the exact generator command/version; inspect before copying application files into the repository. Do not overwrite planning documents or copy a nested `.git` directory.
- [ ] Apply the specified product/package identity and publication restrictions. Install dependencies, commit both lockfiles, and extend ignore rules only for actual generated artifacts. Preserve upstream notices.
- [ ] Run `pnpm build`, `pnpm exec tauri info`, and `pnpm tauri build --no-bundle`. Expected: successful frontend and Windows binary compilation with the pinned tools. Record any unresolved prerequisite instead of claiming this task passed.
- [ ] Inspect the diff and commit only the starter, pins, and development instructions as `chore: scaffold official Tauri React starter`.

Generated code and configuration are verified by their build commands. Custom behavior in later tasks follows a failing-test-first cycle.

## Task 2: Typed metadata and themed foundation screen

**Files:** Create/modify the frontend screen, bridge, theme, styling, test and lint files in the map above; add `src-tauri/src/commands.rs`, modify Rust registration, remove unused starter examples/opener integration, and limit capabilities/content policy to actual operations.

**Interfaces:** Consume Task 1's build. Produce Rust `get_app_info() -> AppInfo`, with `AppInfo { version: String, platform: String }` serialized to the matching JSON keys; frontend `getAppInfo(): Promise<AppInfo>`; `ThemePreference = "system" | "light" | "dark"`; and `useTheme(): { preference: ThemePreference; setPreference: (value: ThemePreference) => void }` under `ThemeProvider`.

- [ ] Configure Tailwind's Vite integration and the `@/*` alias, then initialize shadcn for the existing project. Add only Button, Card, Select and their required utilities, plus Lucide. Use a neutral style and system fonts. Configure Vitest/jsdom, React Testing Library, ESLint, Prettier, and scripts `typecheck`, `lint`, `format:check`, and `test` (`vitest run`).
- [ ] Write the Rust contract test before implementing the command:

```rust
#[test]
fn app_info_contract() {
    let info = get_app_info();
    assert_eq!(serde_json::to_value(info).unwrap(), serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
    }));
}
```

- [ ] Run `cargo test --locked --manifest-path src-tauri/Cargo.toml`; expect failure for the missing command/DTO.
- [ ] Implement the metadata DTO/command and registration. Run the same tests and require a pass. Keep this logic independent of WebView/tray initialization.
- [ ] Write bridge tests with Vitest mocks of Tauri's `invoke` and `isTauri`. Assert invocation of `get_app_info` once, acceptance of a valid response, rejection of missing/empty/non-string fields and native invocation errors, and refusal in browser preview without invoking native IPC. All bridge failures use `Native desktop information is unavailable.`. Representative assertions:

```typescript
it("rejects malformed native metadata", async () => {
  vi.mocked(isTauri).mockReturnValue(true);
  vi.mocked(invoke).mockResolvedValue({ version: 3, platform: "windows" });
  await expect(getAppInfo()).rejects.toThrow("Native desktop information is unavailable.");
});

it("accepts valid native metadata", async () => {
  vi.mocked(isTauri).mockReturnValue(true);
  vi.mocked(invoke).mockResolvedValue({ version: "0.1.0", platform: "windows" });
  await expect(getAppInfo()).resolves.toEqual({ version: "0.1.0", platform: "windows" });
  expect(invoke).toHaveBeenCalledWith("get_app_info");
  expect(invoke).toHaveBeenCalledTimes(1);
});
```

- [ ] Write theme tests around `ThemeProvider` and `useTheme`: key `texpio.theme`; absent/invalid/denied storage selects `system`; dark OS preference adds the root `dark` class; live OS changes update System mode; explicit Light/Dark overrides persist and ignore OS changes. Write screen tests using a mocked bridge: initial loading; rendered version/platform; concise error; Retry causes a new request; browser preview is unavailable; unimplemented-expansion and close-to-tray explanations are visible.
- [ ] Run `pnpm test`; expect failures for the missing bridge, provider, and screen implementations.
- [ ] Implement `getAppInfo`, theme provider, and screen to satisfy those assertions. Use runtime response guards at the bridge, generic display errors without paths/stacks, semantic controls, and visible focus. Keep the bridge request out of render and prevent late responses from changing an unmounted screen. Connect the provider and screen in the application entry.
- [ ] Run `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm test`, `pnpm build`, and the Rust tests. Expected: all pass. Inspect the production content policy and capability diff; remove unused opener/sample permissions and dependencies.
- [ ] Commit as `feat: add themed foundation screen and typed metadata`.

## Task 3: Windows tray and single-instance lifecycle

**Files:** Create `src-tauri/src/desktop.rs`; modify `src-tauri/src/lib.rs`, Cargo dependencies, and development verification notes. Reuse the starter icon until dedicated branding exists.

**Interfaces:** Consume Task 2's main window. Produce Rust `show_main_window(app: &tauri::AppHandle) -> tauri::Result<()>`, which restores/shows/focuses the window, and `setup_tray(app: &tauri::AppHandle) -> tauri::Result<()>`. The frontend requires no new native permissions or lifecycle command.

- [ ] Add the official single-instance plugin and tray feature. Before lifecycle implementation, write the Windows acceptance checklist below in `docs/development.md`; record its initial failures on the current build. A UI/browser mock is not the acceptance evidence for this task.
- [ ] Register single-instance first and call `show_main_window` for second launches. Implement tray menu items `Open Texpio` and `Quit`; Open uses the same helper and Quit calls native process exit. Intercept the main window's close request to hide it; do not intercept explicit exit into another hide. Return initialization failures instead of silently running without a usable tray.
- [ ] Run Rust formatting/tests/Clippy and rebuild the Windows binary. Expected: pass with no unused plugin permissions.
- [ ] Launch that binary and record each result: real metadata; close hides without exiting; Open restores a hidden window; Open restores a minimized window; a second launch restores/focuses the original and leaves one app process; tray Quit terminates it; reopening after Quit succeeds. Include build/OS and whether observations used native UI tooling or a maintainer. If no native interaction mechanism is available, leave those observations incomplete and obtain human verification before declaring completion.
- [ ] Commit as `feat: add tray lifecycle and single-instance behavior` after the checklist passes, or explicitly record the remaining verification gap.

## Task 4: Repeatable checks and handoff

**Files:** Create `.github/workflows/checks.yml`; finish `docs/development.md`, README developer instructions, and TASKS status. Adjust scripts/formatting as required by the completed checks.

**Interfaces:** Consume Tasks 1–3's commands and lockfiles. Produce a pull-request/push check workflow, documented local commands, and an evidence record that distinguishes builds, automated tests, and native observations.

- [ ] Add frontend checks/build on a Linux runner and Rust formatting/Clippy/tests/native compilation on a Windows x64 runner, triggered by pull requests and pushes to `main`. Pin actions to reviewed commit SHAs, use the toolchain files/exact pnpm version, install with `pnpm install --frozen-lockfile`, and cache against lockfiles. Use read-only repository permissions and no release/signing secrets.
- [ ] Run the local check sequence: `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm test`, `pnpm build`, `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --locked --manifest-path src-tauri/Cargo.toml`, and `pnpm tauri build --no-bundle`. Require each command to pass. Record remote workflow results separately; local success alone does not prove CI passed.
- [ ] Finish PowerShell setup/run/check instructions and the verification record. Explain close-to-tray and Quit, confirm no text expansion yet, link the selected stack/spec, and identify the detailed P0 probe as the next feature work. Check local documentation links, `git diff --check`, staged contents, and absence of secrets/dependency/build/runtime files.
- [ ] Run the chosen execution method's independent final review and resolve confirmed issues. Commit the CI/docs completion as `ci: verify Texpio desktop foundation`.
- [ ] Push the implementation commits to the authorized repository/branch without force, verify the remote SHA and working-tree state, and report passed checks plus any incomplete native/CI observations. Do not mark the foundation complete while its acceptance items remain unmet.
