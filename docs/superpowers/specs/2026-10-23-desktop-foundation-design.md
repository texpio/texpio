# Texpio desktop foundation design

23 October 2026 · Written spec awaiting user review

## Purpose and scope

Create a reproducible Windows development foundation from the official Tauri 2 React/TypeScript/Vite starter, with shadcn/ui and the agreed tooling. The intended user is the maintainer preparing to test native text expansion. Success means the desktop shell builds and runs, its React and Rust layers communicate, and subsequent features have clear boundaries and repeatable checks.

The user confirmed Windows first, React + TypeScript, shadcn/ui, the official Tauri starter, and future reuse of a separate Laravel/PostgreSQL project. This spec assumes a Windows x64 development target and a small foundation screen instead of a snippet manager. It precedes the separate P0 native-input probe in the [phased PRDs](../../phased-prds.md).

## Starting approach

Use the official starter and integrate its generated application files into the existing repository. Generate in a temporary staging directory, inspect the output, and preserve the existing Git history and planning documents. Keep a single application at the repository root, with `src/` for React and `src-tauri/` for Rust; a monorepo adds no benefit to this scope.

This choice follows the user's selection over two alternatives: a community Tauri/shadcn starter would save some setup but introduce third-party lifecycle and release assumptions; manual assembly would provide control but duplicate the official starter. Use shadcn's existing-project instructions after generating Tauri, so a second generator does not replace the application.

Set the product name to `Texpio`, package names to `texpio`, and application identifier to `io.github.texpio.texpio`. Use private JavaScript packaging and disable Cargo publication. Preserve upstream notices included in generated or copied files; applying a project license remains a separate decision.

## First screen

Replace the generated greeting screen with a small Texpio foundation screen. It shows the application name, version and platform returned by Rust, connection status, and a theme selector with System, Light, and Dark choices. Show a clear status that automatic text expansion has not been implemented. Do not show invented snippets, working expansion controls, or a sync login.

While metadata loads, display a loading state. If the command fails, show a concise connection error and a Retry button. The error must not expose a stack trace or local filesystem path. A plain browser preview must show that native desktop information is unavailable rather than fabricating a successful connection.

Use a neutral shadcn style, Lucide icons, system fonts, semantic landmarks, visible keyboard focus, and text labels for controls. Support keyboard operation and Windows light/dark mode. Persist only the theme preference in localStorage, validate the stored value, and fall back to System when storage is unavailable or invalid. This is UI preference storage; snippet data will later live in SQLite.

## Desktop lifecycle

Use the native Tauri tray API with **Open Texpio** and **Quit** menu actions. Open restores a minimized window, shows it, and focuses it. Closing the main window hides it while the process remains available in the tray. Provide visible explanatory text on the foundation screen stating that closing keeps Texpio running and that the tray menu's Quit action exits it.

Register the official single-instance plugin before other plugins. Launching a second instance restores, shows, and focuses the existing main window and does not leave a second background process. Quit exits the process; it must bypass close-to-tray behavior. Do not enable launch at login or register a global shortcut in this foundation.

## React and Rust boundary

Expose one read-only `get_app_info` command returning a Serde DTO serialized as `{ version: string, platform: string }`. Read the version from Cargo package metadata and return the actual compile target platform. Put frontend invocation and its result type in one module. Validate the small response shape at this boundary without adding a full forms dependency solely for this command.

Keep Rust command registration and tray/window lifecycle separate from metadata creation so the metadata contract can be unit tested without launching a WebView. Use local React state for this one request; introduce TanStack Query when library operations need caching and invalidation. No database, input hook, clipboard manipulation, network call, or account dependency belongs in this command.

The webview loads bundled assets in production. Keep Tauri capabilities limited to operations actually used, with no broad shell, filesystem, SQL, HTTP, or remote-page access. Remove the starter's unused example command and opener integration if the foundation screen has no need for them. Allow only the development origin needed by Vite and document the production content policy.

## Repository and tooling

Retain the official root layout and add focused frontend areas for `components/ui`, the desktop bridge, theme handling, and the foundation screen. Under `src-tauri/src`, separate the metadata command from desktop setup. Do not create empty engine, storage, or sync implementations.

Use strict TypeScript, the Vite Tailwind integration, checked-in shadcn configuration/components, pnpm, Vitest, React Testing Library, ESLint, and Prettier. Pin tested Node and Rust versions and the exact pnpm version. Commit both lockfiles. The selected storage, forms, query, and native-input dependencies are documented in the [stack record](../../tech-stack.md) and will be added with the features that use them.

Add GitHub Actions for frontend checks/build and Windows Rust formatting, linting, tests, and desktop compilation. Use frozen JavaScript dependencies and Cargo's locked dependency resolution. Build checks produce no public release, updater feed, or signed installer. Keep secrets and local runtime data outside version control.

## Verification and acceptance

The implementation is ready when all the following evidence is recorded:

1. A fresh dependency install follows the checked-in toolchain versions and lockfiles, with documented PowerShell development commands.
2. Type checking, frontend lint/format checks, Vitest, and the Vite production build pass. UI tests exercise loading, a typed bridge success response, malformed response handling, error/retry, valid/invalid stored theme preferences, unavailable localStorage, and live system-theme changes in System mode. Do not claim a browser test proves native IPC.
3. Rust formatting, Clippy, tests, and Windows desktop compilation pass. A serialization test verifies the command response keys and package version against the frontend contract.
4. On the Windows host, launch the desktop app and verify real Rust metadata, theme behavior, closing to tray, reopening, second-instance focus, and explicit Quit. Distinguish these manual observations from automated checks.
5. README instructions identify the prerequisites, commands, scope, and next P0 work. Existing planning files remain intact, and no credentials, runtime database, build output, or dependency directory enters Git.

When a prerequisite prevents a check, record the exact missing prerequisite and leave that acceptance item incomplete. A successful web build alone is insufficient to claim a working desktop foundation. Windows requires Rust, the Microsoft C++ build tools/SDK, and WebView2; see the [official prerequisites](https://v2.tauri.app/start/prerequisites/).

## Follow-on work

After the foundation, write the detailed P0 probe for Windows input observation, literal matching, guarded replacement, focus changes, and insertion strategy comparison. Its report selects the engine approach before the full library editor is built. Storage, editor/forms, picker, autostart, installers, updates, macOS, and paid sync retain their separate phase gates. This foundation does not complete P0 or implement the text expander.
