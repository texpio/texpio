# Texpio development

The desktop foundation uses the official React/TypeScript/Vite Tauri 2 starter. Native text expansion is a later P0 milestone. Future accounts, billing, and sync use a separate Laravel/PostgreSQL project.

## Starter provenance

Generated on 23 October 2026 using npm package `create-tauri-app@4.7.5` (its native CLI reports `4.8.0`):

```powershell
npm exec --yes --package=create-tauri-app@4.7.5 -- create-tauri-app texpio --template react-ts --manager pnpm --identifier io.github.texpio.texpio --tauri-version 2 --yes
```

Generation ran in an ignored staging directory. Application source/configuration was inspected and copied into the existing checkout; the repository history and planning documents were preserved. The starter's React/Vite/Tauri source remains the baseline for the foundation changes.

## Windows prerequisites

Use Windows x64 with Node `24.18.0` and pnpm `12.15.0`. `.node-version` and `package.json` pin these versions. Rust/MSVC and the Microsoft C++ desktop workload with a Windows SDK are required. The existing WebView2 runtime is reused. See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

Rust `1.99.1` is pinned in `rust-toolchain.toml`. On the development host, Rust/Cargo `1.99.1`, Visual Studio Build Tools `17.14.37806.7`, Windows SDK `10.0.26100.0`, and WebView2 `155.0.4283.62` were detected after installation. The Rust default host is `x86_64-pc-windows-msvc`. Open a new terminal after first installing the tools so Cargo's PATH entry is available.

The initial official-starter frontend production build and Windows native compilation passed, including `pnpm tauri build --no-bundle -- --locked`. The foundation behavior and final acceptance checks remain in progress.

## Development commands

From the repository root:

```powershell
pnpm install --frozen-lockfile
pnpm tauri dev
```

For a frontend browser preview use `pnpm dev`; native commands require the Tauri application. To compile the Windows binary without producing installers:

```powershell
pnpm tauri build --no-bundle
```

Application tests, lifecycle behavior, CI, and final verification instructions are added with the corresponding foundation tasks.
