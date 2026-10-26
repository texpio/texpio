# Title bar and branding polish — 26 October 2026

Base: `63f7e4890621540cda3d9d393da4c2bcf20c9c69`. The user requested removing
the centered title-bar wordmark and sidebar wordmark, adding a small app icon on
the title bar's left, and conventional double-click maximize/restore behavior.

## Result

- The title bar has one 16 px existing app icon on the left, followed by the
  existing sidebar toggle. Its center is empty. The icon retains an accessible
  application title and does not start an image drag.
- The sidebar wordmark and its entire header row are removed; navigation starts
  directly below the title bar. Fonts, theme, brand assets and native icons remain.
- Windows and macOS title bars use Tauri's subtree drag region. The installed
  handler excludes interactive controls, preserving sidebar and window buttons.
  Linux continues to use its native window decorations.
- Main-window capabilities now allow `internal_toggle_maximize`, the command
  emitted by Tauri 2.12.5's native drag script on double-click. Existing
  `toggle_maximize` permission alone did not authorize that separate command.
  The picker receives no additional window permissions. No competing JavaScript
  double-click handler, Rust command or input-engine change was added.

## Verification

Resolving main-window permissions through the generated Tauri ACL failed before
the fix because `internal_toggle_maximize` was absent. It passes after the fix.

`bun run check:all` passed: 162 frontend tests, 188 Rust library tests, 12 tooling
tests, one fixture-server test and all TypeScript/lint/format/AST/Clippy gates.
`bun run build` passed. A read-only review of the five changed source/configuration
files reported no findings.

An isolated headless Chrome fixture loaded the production frontend and the exact
installed Tauri Windows drag script, using synthetic native replies. It verified
one maximize/restore command per empty-title-bar double-click, exclusion of the
sidebar button, one registry command for the maximize button, a 16 px left icon,
no rendered wordmarks and no blank sidebar header. Light/dark, 800×560 and Arabic
RTL checks passed without page-level horizontal overflow or page errors. An
initial harness selector used the wrong capitalization for Hide Left Sidebar;
correcting that selector required no product change. The browser/server ended.

The fixture never started expansion, sent native input or saved library content.
It verifies DOM, native-script dispatch and permission configuration; it does not
establish actual OS maximize/restore behavior. Local screenshots and the fixture
script are ignored under `.superpowers/sdd/2026-10-26-titlebar/`.

## Native build and preview

The user saved and quit the old preview, then acknowledged that it was closed.
`bun run tauri build --no-bundle -- --locked` passed; native release compilation
took 5m 12s. The generated release capabilities grant the double-click command
only to the main window. No installer or signed release was produced.

The normal optimized executable was reopened without remote-debugging arguments.
A read-only process/window check confirmed the expected executable path, title
Texpio, visible window and live process (PID 8344 at verification). Reobserve the
process before any future action and preserve drafts before replacing it.
Executable SHA-256: `EEA1A4BF52C5CA548EF308B15D6D2283222C60F2E17FC7327A3C934D6F9ADE4B`.
Actual mouse double-click maximize/restore in the live native window remains an
unverified acceptance check; the rendered fixture is not recorded as that check.
