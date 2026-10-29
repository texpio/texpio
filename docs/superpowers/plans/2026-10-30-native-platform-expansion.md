# Native platform expansion implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add macOS, Linux X11 and Linux Wayland product engines, including a separate Wayland helper and distinct Linux release variants, on x64 and ARM64.

**Architecture:** Preserve the Rust product matcher, immutable rules and cancellation generations. A compile-time platform facade owns native runtimes; Linux X11 and Wayland are mutually exclusive variants. Native clipboard/target/permission checks stay outside the webview, and Wayland device access belongs to a separate helper.

**Tech Stack:** Tauri 2, Rust 1.99.1, Bun 1.4.4, Node 24.18.0, React, native Quartz/X11/evdev/uinput and GNOME integration.

**Spec:** [Approved native expansion design](../specs/2026-10-30-macos-linux-expansion-design.md).

## Global constraints

- Work in the approved current checkout. No global tooling changes, upstream runtime source copying, publication or automatic privileged helper installation.
- Keep the strict Windows P0 probe separate. Ordinary checks never arm global capture or send input.
- Preserve Windows behavior, native generations/target checks and clipboard ownership. Refuse unsupported operations before erasure.
- Linux builds select exactly one of `linux-x11` or `linux-wayland`; name artifacts and update manifests independently.
- New visible copy belongs in all three root locales. Native data uses typed commands and TanStack Query.
- macOS/Linux compile and native interaction evidence must be reported separately from Windows checks.

## Review focus

- A delivered completing key must not leave one character behind or erase a prefix.
- Stop, focus/layout change and clipboard ownership loss must cancel pending output.
- Passive raw-device capture must never authorize deletion before the editor committed the trigger.
- The helper must reject another session/peer, stop on lock/disconnect and never grant privileges to Tauri.
- Linux variant builds and updater manifests must not overwrite one another.

## Task 1: Shared input contract and runtime selection

**Files:** `src-tauri/src/product/{matcher,service,mod,effects}.rs`, `product/platform/mod.rs`, `src-tauri/Cargo.toml`.

**Interfaces:** Each native module exports `Runtime::launch(Arc<Shared>, Vec<ProductRule>, ProductSettings) -> Result<Self, ProductError>` and `Runtime::shutdown(&mut self) -> bool`. `TypedHistory::push_delivered(char)` and `backspace_delivered()` return `Option<Expansion>` for observation-only backends; existing `push`/`backspace` retain Windows semantics.

- [ ] Add failing tests for delivered immediate/delimiter/backspace deletion and refusal of already-delivered Tab/Return activation; run focused Rust tests.
- [ ] Implement explicit delivered-input methods and keep pending-input tests unchanged.
- [ ] Add compile-time native facade and route ProductEngine start/stop through it; make Linux feature combinations explicit.
- [ ] Add shared fake-runtime/cancellation tests; run Rust suite and preserve generation publication behavior.

## Task 2: macOS runtime and effects

**Files:** `product/platform/macos/`, platform-scoped manifest dependencies.

**Consumes:** Task 1 runtime signature and current `EffectBackend`, `ProductRule`, `ProductSettings`, `Shared`.

- [ ] Add policy tests before implementation for permission refusal, target changes, Unicode event boundaries, cancellation and clipboard ownership.
- [ ] Implement owned Quartz tap/runloop, bounded queue, marked events and acknowledged teardown; no callback permission prompts.
- [ ] Implement AX target/protected-context checks, marked native typing and NSPasteboard paste leasing; unsupported payloads refuse before deletion.
- [ ] Validate exact pinned bindings and compile on a macOS runner; record unavailable native-host checks honestly.

## Task 3: Linux X11 runtime and effects

**Files:** `product/platform/linux_x11/`, platform-scoped manifest dependencies.

**Consumes:** Task 1 runtime signature and delivered-input methods.

- [ ] Add policy tests for observed completion, focus/layout changes, self-generated input exclusion, keymap-representability and clipboard ownership.
- [ ] Implement X RECORD, XKB translation, XTEST effects and bounded start/stop.
- [ ] Implement bounded native selection transfer and ownership-safe snapshot restoration; no shell clipboard dependency or global keymap rewriting.
- [ ] Validate exact x11rb APIs and compile on matching Linux CI; keep physical acceptance separate.

## Task 4: Wayland helper and desktop bridge

**Files:** `product/platform/linux_wayland/`, `native/wayland-helper/`, `native/gnome-extension/`, `packaging/linux-wayland/`.

**Consumes:** Task 1 runtime signature and delivered-input methods.

- [ ] Add failing tests for protocol bounds/version, stale generations, peer/session authorization and committed suffix/caret evidence.
- [ ] Implement a separately packaged helper and authenticated bounded local IPC; use evdev observation/uinput without grabbing/replaying the physical keyboard.
- [ ] Implement active local unlocked session revocation, hotplug/resync and virtual-device exclusion; keep Tauri unprivileged.
- [ ] Implement the GNOME context/clipboard bridge and native AT-SPI evidence path. No missing-context fallback may permit erasure.
- [ ] Package explicit setup/uninstall instructions. Run helper protocol tests without device access; require GNOME native feasibility evidence before support claims.

## Task 5: Desktop settings and capability UI

**Files:** product desktop adapters/commands, `lib.rs`, bindings, `src/services/product.ts`, expansion Settings, root locales.

- [ ] Add meaningful tests for non-Windows native settings application and bounded capability parsing, including unknown/unsupported replies.
- [ ] Split portable shortcut/startup flow from Windows-native clipboard dispatch; map cancel keys per platform while retaining persisted settings strings.
- [ ] Add typed capability/permission/setup commands, regenerate bindings, and show actionable platform state without exposing native context.
- [ ] Keep unsupported payloads/actions explicit; verify all three locales and cleanup across StrictMode.

## Task 6: Separate Linux builds, review and acceptance

**Files:** `.github/workflows/{checks,release}.yml`, `src-tauri/tauri.linux-{x11,wayland}.conf.json`, build/release scripts and tests, developer/platform/security docs, `TASKS.md`.

- [ ] Add failing release-matrix tests for eight named OS/backend/architecture variants and independent Linux updater manifests.
- [ ] Implement explicit targets, Linux feature flags/config overlays and helper packaging without publication; preserve pinned tools and release gates.
- [ ] Run independent source review and resolve actionable findings.
- [ ] Run frozen install, bindings, `bun run check:all`, frontend build and locked Windows no-bundle build. Never replace a working executable after a failed build.
- [ ] Stop only the verified checkout executable, start through `scripts/launch-dev.ps1` outside the sandbox, and perform focused actual-app checks; leave preview open.
- [ ] Record remote/native checks that could not run. Update completion checkboxes only for observed results.

## Execution rulings

The user explicitly approved the design and requested immediate implementation;
the plan records that authorization and adds no second approval pause. Current
checkout execution remains the established choice. Independent platform modules
may be implemented concurrently after fixing their shared interface; only the
coordinator edits shared files/manifests and regenerates bindings. Platform
implementers do not arm input or control the desktop.

## Execution record

Shared/native source previews, typed Settings capabilities and eight build
variants are implemented. Independent source reviewers rechecked clipboard,
key-release, context, autostart and worker-attestation fixes. The final local
check suite and Windows build/preview passed; see the
[acceptance record](../../research/native-platform-acceptance.md) for exact counts.

The user explicitly retained strict verification with unsupported fields disabled
and deferred foreign native testing. Generic composition providers remain missing,
so the macOS/Linux qualification steps above are intentionally not marked complete.
No remote build, helper installation or release publication is claimed.
