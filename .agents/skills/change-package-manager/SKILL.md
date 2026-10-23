---
name: change-package-manager
description: Use when the user explicitly requests migrating this repository between Bun, pnpm or npm.
---

# Change Package Manager

Read the requested target from the user's message. If no target can be inferred, ask which manager. If already configured, verify the actual pins and explain that no migration is needed.

1. Record current direct dependency resolutions, tool versions and lockfile integrity. Preserve the original lockfile until replacement resolution is proven; use a staging checkout or reversible backup. No dependency upgrades are implied.
2. Qualify the target version against the existing lock format. Generate its lock while the original evidence remains available. Compare all original direct resolved versions, explain differences and validate a clean frozen install before retiring the superseded lockfile. Keep Node for Node-based tools.
3. Update package scripts, Tauri before-build/dev commands, toolchain pins, workflows, current docs and agent instructions contextually. Preserve historical command evidence and intentional artifact ignores. Pin Actions by immutable SHA and retain explicit lockfile-existence checks.
4. Use local binaries through the chosen manager. For this checkout use Bun 1.4.4, `bun run test` for Vitest and `bun x --no-install TOOL`; a global `bunx` shim may select a different Bun. Do not silently alter global tooling or trust new lifecycle scripts.
5. Verify frozen installation, unchanged intended resolutions, full checks and frontend/native builds. Search remaining old-manager references and classify intentional historical hits.

Migration does not authorize publishing packages, releases, tags, signing keys or changing unrelated product identity. [Release preparation](../../../scripts/prepare-release.js) has no publishing API. See [development setup](../../../docs/development.md).

Adapted from [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c), MIT; see [notice](../../../THIRD_PARTY_NOTICES.md).
