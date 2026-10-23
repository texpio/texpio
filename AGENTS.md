# Texpio agent instructions

## Keep the user's computer available

- Prefer code, CLI, build, and test tools for development. Use Computer Use only when a native visual or interaction check is necessary.
- Keep each Computer Use session brief and focused on one small verification. Explain what will be checked before starting. Do not leave a session active while writing code, running builds/tests, waiting, or doing unrelated work.
- Use supported timeouts for UI actions. Treat an action that has not completed within about 15 seconds as stalled; do not wait indefinitely, loop on closing an app, or queue more input behind a stalled request.
- When a UI action stalls, stop sending input and end/reset the control session. Inspect the failure through read-only development tools. Reobserve the app before any later action; an unknown outcome is not permission to repeat a click or close request.
- End Computer Use promptly after the check, on failure, and before returning control to the user. Use the supported session stop/close mechanism and clear stale control bindings. Do not assume resetting JavaScript bindings alone proves that the visible Computer Use session has closed. If the available tools cannot close it, tell the user immediately rather than leaving them waiting.
- Tell the user when Computer Use has actually ended. Leave the app open only when it is needed for their preview; ending automation and closing the app are separate actions.
- If the user presses Escape or stops Computer Use, stop UI automation immediately. Do not restart it in the same turn. Resume only after the user explicitly asks to continue.
- If native UI verification cannot be completed promptly, record the exact unverified check and continue independent code/test work. Do not claim that the native check passed or enable unverified input behavior to bypass the acceptance gate.

These rules apply to every agent working in this repository, including delegated agents. Only one agent may control the desktop at a time.

## Dependency skills

Use installed dependency skills for relevant work. Read their `SKILL.md` and only the references needed for the task. Preserve this project's pinned tools, lockfiles, Radix components, and existing theme; examples in a skill do not authorize dependency upgrades or reinitializing the app.

Installed in `C:/Users/mohda/.codex/skills/` on 23 October 2026:

| Skill                    | Provenance                                                                                                                                                                                            | Installed source revision                  |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| `shadcn`                 | [Official shadcn/ui skill](https://github.com/shadcn-ui/ui/tree/3d98900e9b73c45f035f8109e9d41ba6a51685e7/skills/shadcn)                                                                               | `3d98900e9b73c45f035f8109e9d41ba6a51685e7` |
| `vite`, `vitest`, `pnpm` | [Anthony Fu's maintainer-provided collection](https://github.com/antfu/skills/tree/d02c48452d782231e4c32d7069cde731a4c7db42/skills), generated from upstream docs; not organization-official releases | `d02c48452d782231e4c32d7069cde731a4c7db42` |

The selected collection covers Vite 8.3.1, Vitest 5.0.1, and pnpm 12.x, matching this project's major versions. Check current installed APIs before applying examples.

- For shadcn work, inspect `components.json` and existing components, use the installed CLI through `pnpm exec shadcn`, fetch relevant component docs, and preserve semantic theme tokens and accessible composition.
- For Vite/Vitest work, use the installed skills for configuration, test isolation, and mocks; keep native Rust and browser-only tests distinct.
- For package changes, use pnpm and frozen installs for verification; do not copy obsolete CI version numbers from illustrative skill snippets.
- No relevant official Tauri, Rust/windows-rs, or Tailwind consumer-development skill was verified during this search. Use official documentation and the exact installed bindings for those dependencies. Do not label community skills as official or install unrelated contributor workflows.

To reproduce installation on another machine, use the `skill-installer` helper with the repository, path, and revision above: `shadcn-ui/ui` + `skills/shadcn`; `antfu/skills` + `skills/vite skills/vitest skills/pnpm`.
