# Native platform implementation and acceptance

30 October 2026 · Incomplete native previews

The user approved the [native design](../superpowers/specs/2026-10-30-macos-linux-expansion-design.md)
and requested separate X11/Wayland downloads. Source previews and build workflows
are implemented; working macOS/Linux expansion is not complete. The user selected
CI preparation with native interactive testing deferred.

## Platform status

| Backend | Implemented source | Remaining functional prerequisites |
| --- | --- | --- |
| Windows | Existing product engine behind the new runtime facade; pending-event matching preserved | Existing native qualification remains separate |
| macOS | Quartz tap/runloop, bounded jobs, AX context, Unicode events, non-restoring native paste, permissions | A usable positively qualified field/composition provider; restore-required paste is refused because public pasteboard APIs offer no atomic compare-and-clear |
| Linux X11 | RECORD/XKB/XTEST, native selection leases, shared AT-SPI context, bounded lifecycle | Positive composition evidence; readiness refuses until this provider exists; restored selection lifetime after Stop needs qualification |
| Linux Wayland | Separate broker/worker, bounded session protocol, evdev/uinput source, GNOME 46 bridge, root-owned worker attestation and Debian assembly | Positive per-field composition provider; native authentication/integration still needs Linux/GNOME execution evidence |

The missing generic composition providers are functional gaps, not just missing
test results. The user explicitly chose to retain strict checks and leave
unsupported fields disabled. Increasing
privileges or setting an environment flag must not bypass them. Installation of
the helper does not establish working Wayland expansion. See its
[feasibility record](../../native/wayland-helper/FEASIBILITY.md) and
[setup/uninstall guide](../../packaging/linux-wayland/README.md).

## Separate release variants

The shared [target matrix](../../scripts/platform-builds.js) defines eight jobs:
Windows and macOS x64/ARM64, Linux X11 x64/ARM64, and Linux Wayland x64/ARM64.
Both Linux editions retain the same product version and library identity, with
distinct download names and update feeds. They are alternatives for a desktop
session; do not run both against one library concurrently.

Linux overlays use `Texpio X11` / `Texpio Wayland` product names and select
`linux-x11` / `linux-wayland` Rust features respectively. Exactly one feature is
required on Linux. The Tauri CLI receives `--no-default-features` after its `--`
separator, because this pinned CLI forwards that Cargo option to the runner.

Updater configuration rejects a feed for the other Linux edition. Draft release
assembly produces `latest-linux-x11.json` and `latest-linux-wayland.json`, each
with x64 and ARM64 assets/signatures. No private signing key or active default
update configuration was added. The manual/environment-gated workflow still
creates drafts only; no release has been published by this implementation.

Native checks build explicit target triples without installing global observers
or sending input. The separate helper package is built only for Wayland jobs;
package installation and socket activation remain explicit user actions. CI
source has been prepared, but these new remote jobs have not been executed here.

## Review corrections

Independent source reviews identified and corrected completing-key deletion
accounting, known payload refusal before suppression, uncertain key-release
cleanup, clipboard acquisition/restoration races, sparse X11 keymap lookup,
accessibility registry-root traversal, physical input during Wayland effects,
AppImage autostart paths and unsupported macOS cancel settings.

The macOS restoration race was resolved by refusing restore-required paste,
not by claiming a closer change-count check was atomic. Missing AX protection or
composition evidence now refuses. GNOME authentication uses bounded root-owned
records containing the forked worker's actual PID, session UID and kernel start
time. The bridge checks the D-Bus daemon's peer identity and process lifetime
without making the worker dumpable. Publication ordering and cleanup were
independently reviewed; no remaining actionable authentication defect was found.
Native execution remains unverified.

## Local verification evidence

No ordinary check opened an input device, installed a hook or sent native keys.
Pure macOS and X11 policy modules are included in the Windows Rust suite. The
standalone helper tests cover framing, authorization, epochs, deadlines, unsafe
fields, UTF-8 boundaries, physical action queues and injected transport failures.

The frozen Bun install passed without dependency changes. Two existing settings
tests failed when attempting atomic writes in the sandbox's default temp area;
they pass with isolated writable fixtures under `.superpowers/`. A full frontend
run exposed a probe mock returning Stopped after acknowledging Running. Its
fixture now preserves command-acknowledged status for subsequent polls; no probe
runtime behavior or timeout was changed.

The final `bun run check:all` passed outside the sandbox with disposable workspace
temp fixtures: **236 frontend tests, 256 Rust tests, 14 helper tests, 18 tooling
and authentication tests, one loopback fixture test and six AST groups**.
TypeScript, ESLint, Prettier, rustfmt and strict Clippy also passed. No test was
skipped to obtain this result. The separate sandbox junction restriction required
an outside-sandbox run; the original filesystem test remained enabled.

The pinned `bun run tauri build --no-bundle -- --locked` passed, including both
frontend entries and optimized Windows compilation. The native release compile
took 6 minutes 42 seconds. The exact checkout executable was started through
`scripts/launch-dev.ps1` using the same-session Explorer parent. Read-only process
verification confirmed PID 23492, the expected executable path, a visible `Texpio`
main window and `Responding = True`. The preview was left open. No keyboard/mouse
automation or physical input campaign ran.

Windows executable SHA-256:
`5CBB66AE7CD1A6C6651EBBDE462BE255F2980F8C5686574430F484FEEE6D3B88`.
The macOS configuration explicitly declares minimum system version 14.0; that
JSON value was checked locally, not established by a macOS build.

macOS framework metadata checking and the Windows X11 API harness are source-shape
checks only; neither establishes foreign linking or native behavior.

## Required foreign-host acceptance

macOS and Linux linking, installation, permissions, native input ordering,
protected controls, composition, Unicode/newlines, clipboard races, lock/suspend,
hotplug, teardown and x64/ARM64 interaction remain unverified. Resolve the named
functional providers before attempting a supported-release qualification campaign.
Use disposable synthetic content and the readiness rules in AGENTS.md. Existing
P0 physical qualification is not completed by this work.
