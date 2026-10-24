# Texpio technology stack

Selected direction · updated 24 October 2026

Texpio will start from the official Tauri React and TypeScript template. Windows comes first, followed by macOS. The user selected React, shadcn/ui, and the official starter, and confirmed that a separate existing Laravel/PostgreSQL project will provide accounts, billing, and future sync infrastructure.

## Desktop choices

| Area | Choice | When it enters the app |
| --- | --- | --- |
| Starter | Official `create-tauri-app`, React + TypeScript + Vite template | Foundation |
| Desktop shell | Stable Tauri 2, Rust stable with the Windows MSVC toolchain | Foundation |
| UI | React, strict TypeScript, Vite SPA | Foundation |
| Styling | Tailwind CSS 4, shadcn/ui components, Lucide icons | Foundation; add components as used |
| Theme | System default plus light/dark override | Foundation |
| IPC | Narrow Tauri commands, Serde DTOs, pinned Specta-generated TypeScript and validated facades | Foundation and library |
| Local storage | SQLite through Rust-owned bundled `rusqlite` 0.40.2, transactional library/revisions and JSON snapshots | Offline MVP |
| Forms | Controlled React forms with frontend feedback and authoritative Rust validation; evaluate React Hook Form/Zod when needed | Offline MVP |
| Async data | TanStack Query 5 over the IPC wrappers; React state for transient UI | First library queries/mutations |
| Native input | Separate Rust engine; Windows adapter using Microsoft's `windows` crate | P0 feasibility work |
| Desktop integration | Tauri tray API and official single-instance plugin | Foundation |
| Other integration | Official global-shortcut, autostart, dialog, and updater plugins | Add when each feature is implemented |
| Frontend checks | Vitest, React Testing Library, ESLint, Prettier, TypeScript | Foundation |
| Rust checks | Cargo tests, rustfmt, Clippy | Foundation |
| Native UI smoke checks | WebdriverIO with the supported Tauri test integration | After a testable native workflow exists |
| Package management | Bun 1.4.4, Node 24.18.0 for Node tools, pinned Rust/Cargo; committed `bun.lock` and `Cargo.lock` | Adopted shell and offline MVP |
| CI and distribution | GitHub Actions; official Tauri action for release packaging later | Checks now; signed distribution at release |

The official generator supplies the React/Vite/Tauri starting structure. Texpio adds styling, desktop lifecycle, testing, and its own application boundaries. The generator does not supply a text expansion engine. See the [official starter guide](https://v2.tauri.app/start/create-project/) and [shadcn's existing Vite project setup](https://ui.shadcn.com/docs/installation/vite).

The original scaffold used pnpm; the user later selected Bun and the approved template migration replaced its lockfile/scripts. Use the pinned toolchains and `bun install --frozen-lockfile`, retain Tauri major version 2, and use `bun run test` for Vitest. Generator provenance and migration evidence remain in the foundation/template records; do not rely on floating `latest` commands in CI.

## Application boundaries

React owns presentation, form state, and interaction. Rust owns persisted records, validation at the command boundary, and native desktop behavior. Use typed services and closed/redacted errors. The adopted template uses exactly pinned Specta dependencies for generated bindings; regenerate them after command changes and retain facade validation.

The offline library serializes operations and uses SQLite transactions with expected revisions; command facades use blocking workers. Keep SQLite work off the UI thread/input callback. Each operation owns its connection and transaction; stable IDs and versioned JSON are exposed through commands rather than raw SQL. See the pinned [rusqlite API](https://docs.rs/rusqlite/0.40.2/rusqlite/) and [MVP design](superpowers/specs/2026-10-24-offline-mvp-design.md).

The native engine will keep a bounded transient matching buffer and an in-memory snippet index. It must operate independently of an open editor window. Do not forward a raw keyboard stream to React or store typed history. Actual observation, deletion, insertion, focus, clipboard, Unicode, and IME behavior remain subject to the [P0 evidence gate](phased-prds.md#p0--native-expansion-feasibility).

The user selected independent Rust implementation after the [Espanso source audit](research/espanso-source-audit.md). Immutable validated rule snapshots now feed the shared recognizer and native Windows effect worker. The MVP retains verified suffix/caret behavior and separate strict P0 profiles. macOS, X11 and Wayland remain separate future adapters; no source audit or build qualifies them. See [MVP acceptance](research/offline-mvp-acceptance.md) for current limits.

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
