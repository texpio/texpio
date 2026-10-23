# Texpio development

The desktop foundation uses the official React/TypeScript/Vite Tauri 2 starter. The guarded P0 engine is available in Developer; physical qualification is incomplete. Future accounts, billing, and sync use a separate Laravel/PostgreSQL project.

## Starter provenance

Generated on 23 October 2026 using npm package `create-tauri-app@4.7.5` (its native CLI reports `4.8.0`):

```powershell
npm exec --yes --package=create-tauri-app@4.7.5 -- create-tauri-app texpio --template react-ts --manager pnpm --identifier io.github.texpio.texpio --tauri-version 2 --yes
```

Generation ran in an ignored staging directory. Application source/configuration was inspected and copied into the existing checkout; the repository history and planning documents were preserved. The starter's React/Vite/Tauri source remains the baseline for the foundation changes.

## Windows prerequisites

Use Windows x64 with Node `24.18.0` and Bun `1.4.4`. `.node-version`, `.bun-version` and `package.json` pin these versions. The previous global Bun `1.3.13` cannot migrate the old pnpm multi-document lock. This host has an integrity-verified local Bun at `.tools/bun/1.4.4/package/bin/bun.exe`; prepend that directory to the current process PATH when using it, including for nested Tauri scripts. Other checkouts should install the same pinned Bun version from its official distribution. The repository uses text `bun.lock`; require it to exist before a frozen install. Rust/MSVC and the Microsoft C++ desktop workload with a Windows SDK are required. The existing WebView2 runtime is reused. See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

Rust `1.99.1` is pinned in `rust-toolchain.toml`. On the development host, Rust/Cargo `1.99.1`, Visual Studio Build Tools `17.14.37806.7`, Windows SDK `10.0.26100.0`, and WebView2 `155.0.4283.62` were detected after installation. The Rust default host is `x86_64-pc-windows-msvc`. Open a new terminal after first installing the tools so Cargo's PATH entry is available.

The initial official-starter frontend production build and Windows native compilation passed, including `pnpm tauri build --no-bundle -- --locked`. The foundation behavior and final acceptance checks remain in progress.

## Development commands

From the repository root:

```powershell
bun install --frozen-lockfile
bun run tauri:dev
```

For a frontend browser preview use `bun run dev`; native commands require the Tauri application. Bun launches existing Node-based tools normally; use `bun run test` for Vitest, not `bun test`. To compile the Windows binary without producing installers:

```powershell
bun run tauri:build --no-bundle -- --locked
```

Application tests, lifecycle behavior, CI, and final verification instructions are added with the corresponding foundation tasks.

## Earlier Windows preview acceptance (23 October 2026)

The original starter exited when its main window was closed. The following checklist verifies the foundation lifecycle against the rebuilt native application; results are pending until observed.

- [x] Rust metadata renders version `0.1.0` and platform `windows` with Connected status (observed in the Windows preview on 23 October 2026).
- [ ] System, Light, and Dark themes work through the native app UI.
- [ ] Closing hides the window while one Texpio process remains.
- [ ] Tray **Open Texpio** restores a hidden window.
- [ ] Tray **Open Texpio** restores a minimized window.
- [ ] A second launch restores/focuses the original window and leaves one Texpio process.
- [ ] Tray **Quit** terminates the process; a subsequent launch works.

## Adopted template shell

The approved shell uses Bun and retains the engine in Developer. See [developer guides](developer/README.md), [user guide](userguide/userguide.md) and [template parity](research/tauri-template-parity.md). Run `bun run check:all` and `bun run build`; native acceptance remains separate.

## Migrated shell verification — 24 October 2026

`bun run check:all` passed with 118 frontend tests, 77 Rust tests, 11 tooling checks, one loopback fixture test and six ast-grep rule groups. Coverage passed: 73.49% statements, 70.57% branches, 68.8% functions and 74.67% lines. Coverage is an inventory, not proof of native titlebar/window behavior. Both production HTML entries build. Binding export twice leaves no diff.

Native interaction acceptance on the migrated binary remains pending; Computer Use was not started because this session's documented API cannot prove control-session closure as required by AGENTS.md. [Parity](research/tauri-template-parity.md) lists each pending check. P0 remains 0/300 physical trials with the India layout/provider question unresolved; no input was armed in this migration.
