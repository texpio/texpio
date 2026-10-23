---
name: init
description: Use when explicitly initializing a new project from this template or deliberately renaming its product identity.
---

# Init

Texpio is already initialized. An unrelated feature request does not trigger initialization. Reuse identity, stack and execution choices already supplied by the user. Ask only for missing identity details needed for the requested initialization.

Inspect `package.json`, root `index.html`, `quick-pane.html`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, platform overrides, documentation and workflows. Update product name, identifier, descriptions, Rust names/imports and titles consistently. A Git display name is not a GitHub login: use an authenticated account lookup or the verified repository remote, and retain an existing valid organization/identifier.

Use the [package-manager skill](../change-package-manager/SKILL.md) only when a switch is requested. Preserve repository history, existing plans and application data. Do not invent security contacts, copyright ownership, update endpoints or signing keys. Release setup is a separate task, documented in [releases](../../../docs/developer/releases.md).

Verify frozen dependencies, `bun run check:all`, `bun run build` and the relevant native build. Offer a runnable preview within existing authorization and follow [computer availability rules](../../../AGENTS.md).

Adapted from [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c), MIT; see [notice](../../../THIRD_PARTY_NOTICES.md).
