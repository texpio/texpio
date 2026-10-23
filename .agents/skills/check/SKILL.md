---
name: check
description: Use when checking a completed feature or bug fix against Texpio architecture and quality gates.
---

# Check

Read the changed files and [architecture guide](../../../docs/developer/architecture-guide.md). Check typed IPC, state ownership, accessible UI, translated labels and privacy boundaries. Remove only leftovers introduced by the authorized change; keep reusable UI, Radix dependencies and generated bindings.

Run `bun run check:all` with the pinned Bun on PATH. Inspect actual results and fix failures within scope. Run `bun run build` for frontend changes and the relevant native build when Rust/configuration changes. Ordinary checks never install keyboard hooks or send input. Report checks that could not run as unverified.

Summarize the resulting behavior, validation and material limits; suggest a concise commit message. Commit/push only within the user's existing authorization. Native GUI checks follow [computer availability rules](../../../AGENTS.md).

Adapted from [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c), MIT; see [notice](../../../THIRD_PARTY_NOTICES.md).
