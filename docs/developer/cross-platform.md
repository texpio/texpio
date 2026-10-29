# Cross-platform development

Windows retains its current product engine. macOS, Linux X11 and Linux Wayland
have native source previews and explicit build targets; functional context and
authentication prerequisites still prevent a supported-release claim. See the
[native acceptance record](../research/native-platform-acceptance.md).
Platform adapters choose native behavior; a browser preview reports unknown and
must not pretend to be macOS.

Linux ships separate X11 and Wayland editions for x64 and ARM64. Select exactly
one Rust feature and its configuration overlay:

```sh
bun run tauri build --features linux-x11 --config src-tauri/tauri.linux-x11.conf.json -- --locked --no-default-features
bun run tauri build --features linux-wayland --config src-tauri/tauri.linux-wayland.conf.json -- --locked --no-default-features
```

The editions share library identity but have distinct product/download names and
update feeds. Choose the matching session edition, rather than running both.
The Wayland helper and GNOME integration are a separate explicit installation;
Tauri must remain unprivileged. X11 uses the helper's field-query library without
requiring its privileged daemon. Unknown composition remains a functional refusal
on both Linux backends. Mac restore-required paste also refuses until a valid
ownership contract exists.

Windows/macOS custom titlebars and Linux native decorations use separate Tauri platform configurations. Main close hides to tray; second launch restores/unminimizes/focuses it. macOS Dock reopen also restores main. Window-state excludes visibility and quick-pane state.

Use logical CSS for Arabic and native path APIs in Rust. Never accept arbitrary frontend filesystem paths. Windows/Linux quick panes are floating windows; macOS uses nonactivating NSPanel. Monitor/DPI behavior, menus and shortcut conflicts need native acceptance per target. A Windows build does not establish macOS/Linux runtime correctness.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
