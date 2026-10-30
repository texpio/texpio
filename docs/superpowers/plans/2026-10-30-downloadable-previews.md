# Downloadable Previews Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans for this small CI task.

**Goal:** Produce eight downloadable test packages from the native CI matrix.

**Architecture:** Reuse the existing target matrix and native bundlers. Validate
and stage an allowlist of package files before uploading a separate artifact for
each variant.

**Tech Stack:** Pinned Bun/Node/Rust, Tauri CLI, GitHub Actions, native tar/dpkg.

**Spec:** [Approved task scope and packaging design](../specs/2026-10-30-downloadable-previews-design.md).

## Global constraints

- Keep all eight variants and independent Linux feature selections.
- Use ad-hoc macOS previews and existing unsigned Windows/Linux package support.
- Never run the helper or app input hooks in automated checks.
- Preserve release signing/publication gates and pinned tooling.
- Use the user's already authorized current checkout and push to main.

## Review focus

Missing package formats must fail rather than create documentation-only downloads.
Staging must exclude caches/debug files. macOS archives must preserve the app's
structure and executable bits. Wayland archives must include the correct helper
architecture and readiness notes. Download links must identify the uploaded
artifact in the current run.

## Task 1: Prepare and publish preview artifacts

- [x] Add failing tooling fixtures for all eight output selections, incomplete or
  ambiguous package rejection, and macOS tar archive contents/permissions.
- [x] Add preview build arguments to `scripts/platform-builds.js` and a package
  staging script with build identity, instructions and checksums.
- [x] Update `.github/workflows/checks.yml` to bundle apps, build/package the
  Wayland helper, upload 14-day artifacts and expose download links.
- [x] Add download/test instructions and review the diff independently.
- [ ] Run `bun run check:all`, the pinned Windows development build/restart and
  focused staging checks. Commit and push within the user's authorization.
- [ ] Confirm all eight CI jobs and artifacts, download/inspect the archives, and
  return the actual successful-run/artifact links.
