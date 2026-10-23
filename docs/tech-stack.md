# Texpio technology stack

Selected direction · 23 October 2026

Texpio will start from the official Tauri React and TypeScript template. Windows comes first, followed by macOS. The user selected React, shadcn/ui, and the official starter, and confirmed that a separate existing Laravel/PostgreSQL project will provide accounts, billing, and future sync infrastructure.

## Desktop choices

| Area | Choice | When it enters the app |
| --- | --- | --- |
| Starter | Official `create-tauri-app`, React + TypeScript + Vite template | Foundation |
| Desktop shell | Stable Tauri 2, Rust stable with the Windows MSVC toolchain | Foundation |
| UI | React, strict TypeScript, Vite SPA | Foundation |
| Styling | Tailwind CSS 4, shadcn/ui components, Lucide icons | Foundation; add components as used |
| Theme | System default plus light/dark override | Foundation |
| IPC | Narrow Tauri commands, Serde DTOs, typed TypeScript wrappers | Foundation |
| Local storage | SQLite through Rust-owned `rusqlite` with bundled SQLite | First persistent library feature |
| Forms | React Hook Form + Zod | First snippet editor |
| Async data | TanStack Query 5 over the IPC wrappers; React state for transient UI | First library queries/mutations |
| Native input | Separate Rust engine; Windows adapter using Microsoft's `windows` crate | P0 feasibility work |
| Desktop integration | Tauri tray API and official single-instance plugin | Foundation |
| Other integration | Official global-shortcut, autostart, dialog, and updater plugins | Add when each feature is implemented |
| Frontend checks | Vitest, React Testing Library, ESLint, Prettier, TypeScript | Foundation |
| Rust checks | Cargo tests, rustfmt, Clippy | Foundation |
| Native UI smoke checks | WebdriverIO with the supported Tauri test integration | After a testable native workflow exists |
| Package management | pnpm, Node LTS, Cargo; committed JavaScript and Cargo lockfiles | Foundation |
| CI and distribution | GitHub Actions; official Tauri action for release packaging later | Checks now; signed distribution at release |

The official generator supplies the React/Vite/Tauri starting structure. Texpio adds styling, desktop lifecycle, testing, and its own application boundaries. The generator does not supply a text expansion engine. See the [official starter guide](https://v2.tauri.app/start/create-project/) and [shadcn's existing Vite project setup](https://ui.shadcn.com/docs/installation/vite).

Resolve compatible stable package versions during scaffolding, then commit `pnpm-lock.yaml`, `Cargo.lock`, an exact `packageManager` value, and the tested Node/Rust toolchain versions. Keep Tauri on major version 2. Record the generator version used so the starting point can be reproduced; do not rely on a floating `latest` command in CI.

## Application boundaries

React owns presentation, form state, and interaction. Rust owns persisted records, validation at the command boundary, and native desktop behavior. Keep IPC wrappers in one frontend module and return structured errors. Frontend types and Rust DTOs must agree; test the serialized contract. Start with explicit types rather than introducing a prerelease code-generation dependency.

The later storage layer will own a connection on a dedicated database worker and expose repository operations. Keep blocking SQLite work off the UI thread, input callback, and async executor. Use migrations and transactions, stable record IDs, and JSON interchange. React receives records through commands rather than raw SQL access. The [rusqlite API](https://docs.rs/rusqlite/latest/rusqlite/) provides the Rust SQLite integration.

The native engine will keep a bounded transient matching buffer and an in-memory snippet index. It must operate independently of an open editor window. Do not forward a raw keyboard stream to React or store typed history. Actual observation, deletion, insertion, focus, clipboard, Unicode, and IME behavior remain subject to the [P0 evidence gate](phased-prds.md#p0--native-expansion-feasibility).

The [Espanso source audit](research/espanso-source-audit.md) proposes shared Rust pipeline boundaries with separate Windows, macOS, Linux/X11 and Linux/Wayland adapters. This informs the next engine design; it does not select a fork, change Texpio's license, replace the approved P0 contract or qualify another platform.

## Future service

Reuse the user's existing Laravel + PostgreSQL project for accounts and billing. This desktop repository will contain no Laravel application, PostgreSQL service, authentication UI, payment integration, or sync server during the foundation or P0 work.

A later sync design must establish device authorization, a versioned protocol, conflict/recovery behavior, encryption/key handling, quotas, and cancellation. An existing billing platform does not settle those contracts. Local use remains independent of service availability and subscription state, as specified in the [feature plan](feature-plan.md).

## Decisions reserved for their phase

The [foundation design](superpowers/specs/2026-10-23-desktop-foundation-design.md) defines the first implementation scope. Native engine behavior follows a separate P0 spec. App-store distribution, signing credentials, update endpoints, the full supported OS/app matrix, and the project license need their own decisions before public release. No license is applied by selecting this stack.

## Local development readiness

The read-only Windows audit on 23 October 2026 found Node `v24.18.0` and WebView2 `155.0.4283.62`. Cargo/Rust were absent from PATH and the usual user installation directories. Standard Visual Studio Build Tools and Windows SDK directories/registration were absent. Install the MSVC Rust toolchain and the C++ desktop workload with a Windows SDK before native verification; a nonstandard installation, if present, still needs locating and testing.

A pnpm/Corepack shim exists, but its version lookup failed because the sandbox could not resolve the npm registry. That lookup does not establish a broken pnpm installation. Resolve and pin pnpm when network-enabled dependency setup begins. No native compiler, SDK, or application dependency was installed during this audit.

## Official references

- [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Tauri single-instance plugin](https://v2.tauri.app/plugin/single-instance/)
- [Tauri system tray API](https://v2.tauri.app/learn/system-tray/)
- [Tauri tests](https://v2.tauri.app/develop/tests/)
- [React Hook Form](https://react-hook-form.com/)
- [Zod](https://zod.dev/)
- [TanStack Query](https://tanstack.com/query/latest/docs/framework/react/overview)
- [Microsoft Windows Rust bindings](https://github.com/microsoft/windows-rs)
