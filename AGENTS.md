# Texpio agent instructions

## Development build and restart

- For development work in this checkout, after implementation and applicable checks succeed, build the Windows app with the pinned tools (`bun run tauri build --no-bundle -- --locked`). If the running executable would block an in-place build, build to a separate target directory first, then replace the development executable after stopping the app. Do not replace a working build with a failed build.
- After a successful build, identify the running Texpio development process by its PID and exact resolved executable path, then force-terminate that process before starting the newly built app. Skip termination if it is not running. The user has authorized force termination in this development environment: do not ask them to save drafts or quit through the tray. Never terminate unrelated processes or another Texpio installation.
- Start the new build and verify the work with focused checks appropriate to the change, including an actual-app check where relevant, before marking the work complete. Leave the app open for the user's development preview. If a required check cannot be performed, report precisely what remains unverified and do not claim it passed. Do not restart or build for read-only questions and reviews.

## Keep the user's computer available

- If the user has not replied to a question, assume they are offline. Do not infer readiness or approval from silence, and do not start a check requiring their participation until they reply. For a manual typing check, announce when the window is actually visible and allow enough time for the user to focus it and type; do not end it after a short countdown while they are participating.
- Prefer code, CLI, build, and test tools for development. Use Computer Use only when a native visual or interaction check is necessary.
- Keep each Computer Use session brief and focused on one small verification. Explain what will be checked before starting. Do not leave a session active while writing code, running builds/tests, waiting, or doing unrelated work.
- Use supported timeouts for UI actions. Treat an action that has not completed within about 15 seconds as stalled; do not wait indefinitely, loop on closing an app, or queue more input behind a stalled request.
- When a UI action stalls, stop sending input and end/reset the control session. Inspect the failure through read-only development tools. Reobserve the app before any later action; an unknown outcome is not permission to repeat a click or close request.
- Check each browser/UI result for `isError` or `success: false` before continuing. A failed navigation must not be followed by interactions or a snapshot that could implicitly launch another browser. Close only this task's owned session. If an MCP cleanup call stalls, a documented local CLI may be used after verifying its session identity; never close unrelated browsers or all Chrome processes.
- End Computer Use promptly after the check, on failure, and before returning control to the user. Use the supported session stop/close mechanism and clear stale control bindings. Do not assume resetting JavaScript bindings alone proves that the visible Computer Use session has closed. If the available tools cannot close it, tell the user immediately rather than leaving them waiting.
- Tell the user when Computer Use has actually ended. Leave the app open only when it is needed for their preview; ending automation and closing the app are separate actions.
- If the user presses Escape or stops Computer Use, stop UI automation immediately. Do not restart it in the same turn. Resume only after the user explicitly asks to continue.
- If native UI verification cannot be completed promptly, record the exact unverified check and continue independent code/test work. Do not claim that the native check passed or enable unverified input behavior to bypass the acceptance gate.

These rules apply to every agent working in this repository, including delegated agents. Only one agent may control the desktop at a time.

## Dependency skills

The user approved Bun as the package manager for this migration. Use Bun **1.4.4** (pinned in `.bun-version` and `package.json`) and `bun install --frozen-lockfile`; retain Node 24.18.0 for Node tools. The global Bun 1.3.13 cannot migrate this pnpm lock format. On this host the qualified local executable is `.tools/bun/1.4.4/package/bin/bun.exe`; prepend its directory to the current process PATH when running nested scripts or Tauri. Do not change global tooling silently. Use `bun run test` for Vitest, not Bun's separate `bun test` runner.

Use installed dependency skills for relevant work. Read their `SKILL.md` and only the references needed for the task. Preserve this project's pinned tools, lockfiles, Radix components, and existing theme; examples in a skill do not authorize dependency upgrades or reinitializing the app.

Installed in `C:/Users/mohda/.codex/skills/` on 23 October 2026:

| Skill                    | Provenance                                                                                                                                                                                            | Installed source revision                  |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| `shadcn`                 | [Official shadcn/ui skill](https://github.com/shadcn-ui/ui/tree/3d98900e9b73c45f035f8109e9d41ba6a51685e7/skills/shadcn)                                                                               | `3d98900e9b73c45f035f8109e9d41ba6a51685e7` |
| `vite`, `vitest`, `pnpm` | [Anthony Fu's maintainer-provided collection](https://github.com/antfu/skills/tree/d02c48452d782231e4c32d7069cde731a4c7db42/skills), generated from upstream docs; not organization-official releases | `d02c48452d782231e4c32d7069cde731a4c7db42` |

The selected collection covers Vite 8.3.1, Vitest 5.0.1, and pnpm 12.x, matching this project's major versions. Check current installed APIs before applying examples.

- For shadcn work, inspect `components.json` and existing components, use the installed CLI through `bun x --no-install shadcn`, fetch relevant component docs, and preserve semantic theme tokens and accessible composition. Do not upgrade the CLI implicitly.
- For Vite/Vitest work, use the installed skills for configuration, test isolation, and mocks; keep native Rust and browser-only tests distinct.
- For package changes, use pinned Bun and frozen installs for verification; the pnpm skill remains provenance/reference for historical migration only. Do not copy obsolete CI version numbers from illustrative skill snippets.
- No relevant official Tauri, Rust/windows-rs, or Tailwind consumer-development skill was verified during this search. Use official documentation and the exact installed bindings for those dependencies. Do not label community skills as official or install unrelated contributor workflows.

To reproduce installation on another machine, use the `skill-installer` helper with the repository, path, and revision above: `shadcn-ui/ui` + `skills/shadcn`; `antfu/skills` + `skills/vite skills/vitest skills/pnpm`.

## Application architecture and portable workflows

Read [.agents/README.md](.agents/README.md) for four local skills and two report-only role prompts. Their scope follows the user's request; explicit authorization already in the conversation remains valid. Texpio identity and the approved current-checkout execution choice are established.

- UI state belongs in `src/store/ui-store.ts`; use narrow Zustand selectors in components and `getState()` in pure `src/lib/` functions. Server/native data belongs in `src/services/` TanStack Query hooks; persisted preferences are Rust-owned. Do not persist snippets, quick-pane entries, probe input or diagnostic content in UI layout state.
- Register actions once in `src/lib/commands/`; route palette, menus and shortcuts through that registry. Use typed native facades and regenerate `src/lib/bindings.ts` with `bun run rust:bindings` after command changes. Keep Rust tokens and input decisions off the webview.
- Compose existing shadcn/Radix primitives, semantic colors and logical CSS. Add visible strings to all three root locales. Preserve cancellation and listener cleanup across React StrictMode and multiple windows.
- Follow [architecture](docs/developer/architecture-guide.md), [testing](docs/developer/testing.md) and [security](docs/SECURITY.md). Run `bun run check:all` and relevant builds before completion. Never arm global input in ordinary automated checks. P0 physical qualification remains a separate evidence gate.
- Task/release helpers prepare only requested local work. `TASKS.md` is canonical; release preparation cannot publish. Reusable UI/plugin infrastructure is intentional even when static tools report it unused.
