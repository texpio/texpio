# Wayland / GNOME 46 feasibility record

Status: **incomplete native feasibility preview; ordinary application expansion
is unavailable**. No device capture, uinput injection, root service installation,
or desktop automation ran on the Windows development host.

The implementation deliberately returns `CompositionUnknown` when AT-SPI text,
caret and focused-field evidence cannot be paired with positive per-field
composition-idle evidence. The runtime does not report Running in this state.
This is a concrete unresolved capability, not a successful native implementation
or an installation problem the user can solve by granting broader permissions.
There is no environment flag or settings toggle to bypass it.

The earlier process-symlink authentication blocker now has an implementation:
the root broker publishes `/run/texpio-attest/PID.json` for its actual forked
worker, binding version, session UID, PID and kernel process start time. The child
waits at most 500 ms for publication before dropping privileges and using the
bridge. Root-owned immutable directories, a nonsymlink create-new 0444 file,
bounded parsing, DBus-daemon peer identity and current kernel start time establish
the authentication boundary; stale files cannot authenticate a reused PID. A
broker-owned guard removes the record on worker exit or revocation, and systemd
removes the separate runtime directory on service exit. The worker remains
nondumpable. GNOME authentication errors remain explicit refusals.

This authentication path is **implemented but native-unverified**. Linux process,
GIO metadata, systemd runtime-directory and GNOME DBus behavior still require
native qualification; pure policy tests do not establish those runtime results.

## Source evidence and decision

The following platform API definitions were inspected; no upstream runtime
implementation was copied into Texpio:

- [GNOME Shell 46 InputSourceManager](https://github.com/GNOME/gnome-shell/blob/46.0/js/ui/status/keyboard.js)
  supplies `currentSource.type`, `xkbId` and source-change notifications. These
  select the XKB layout, but do not prove client composition state.
- [GNOME Shell 46 InputMethod](https://github.com/GNOME/gnome-shell/blob/46.0/js/misc/inputMethod.js)
  creates an IBus input context named `gnome-shell`. Its preedit and surrounding
  text belong to that context. Treating its idle state as proof for the focused
  application would be incorrect. Native Wayland clients may use their own
  toolkit input contexts; XWayland requires its own qualification too.
- [AT-SPI 2.52 accessible API](https://github.com/GNOME/at-spi2-core/blob/AT_SPI2_CORE_2_52_0/atspi/atspi-accessible.h)
  and [Text API](https://github.com/GNOME/at-spi2-core/blob/AT_SPI2_CORE_2_52_0/atspi/atspi-text.h)
  provide focused/editable state, process identity, text, selections and caret.
  They do not expose a universal per-field preedit-idle property. Unknown state
  remains unknown. Protected and unrecognized roles are rejected before text is
  read. Native accessibility queries have bounded timeouts and traversal limits.
- [Mutter 46 selection API](https://github.com/GNOME/mutter/blob/46.0/src/meta/meta-selection.h)
  and [memory selection source](https://github.com/GNOME/mutter/blob/46.0/src/meta/meta-selection-source-memory.h)
  expose transfer, ownership and single-MIME sources. The companion implements
  bounded text-only snapshot leases and rejects multi-MIME restoration before
  erasure. No claim that this preserves arbitrary clipboard formats is made.
- [logind session API](https://www.freedesktop.org/software/systemd/man/latest/sd_session_is_active.html)
  distinguishes active/local user sessions. The broker additionally reads
  `org.freedesktop.login1.Session.LockedHint`; the worker independently checks
  Shell screen-shield/session-mode state. Active alone is never treated as unlocked.
- [Kernel uinput API](https://www.kernel.org/doc/html/latest/input/uinput.html)
  supports a marked virtual keyboard. Device discovery excludes virtual sysfs
  devices. No physical keyboard grab or replay is used.

## Implemented source boundaries

- Separately built socket-activated root broker: SO_PEERCRED, PID-bound logind
  session/seat, pidfd lifetime, lock/inactive revocation, one supervised worker.
- Privilege drop: UID, GID, supplementary groups, capability bounding set and
  no-new-privileges. Only the worker retains open evdev/uinput descriptors.
- Versioned 64 KiB frames, bounded body, unknown-field rejection, no paths/commands,
  rejected and closed SCM_RIGHTS descriptors, frame deadline, lease heartbeat,
  generation and operation deadline, client disconnect cancellation.
- Passive device observation, XKB translation from bridge layout/options,
  nonexclusive uinput, virtual-device exclusion, hotplug/overflow revocation
  requiring a new handshake and released-key resynchronization. Locked keyboard
  modifiers/LED states outside the prepared keymap profile are refused.
- AT-SPI committed suffix and caret gate before erasure; each submitted Backspace
  needs observed caret/suffix movement before the next one. Unsupported payloads
  are prepared/refused before erasure. Partial output is never retried.
- GNOME bridge authentication to the packaged native helper process; bounded
  context and clipboard operations only; no evaluator or general command API.
  Concurrent clipboard ownership changes invalidate restoration.
- Native Tauri client, delivered-event matching, bounded serialized jobs,
  generation/Stop lifecycle and explicit missing-helper/bridge/field readiness.
- Debian package assembly for amd64/arm64, explicit opt-in setup and uninstall;
  no changes to Tauri privileges or blanket `input` group membership.

## Evidence recorded on Windows

`cargo test --manifest-path native/wayland-helper/Cargo.toml --offline --locked`
passes 14 device-free policy tests: exact peer/session authorization, protocol
bounds/schema/version, generations/heartbeat/deadlines/revocation, committed
suffix/caret, unsafe fields/composition, UTF-8 boundary refusal, and physical
versus virtual/other-seat device selection. Additional regressions cover the
nonzero-PID accessibility registry root, queued physical actions after keys have
been released, and injected transport failure at key-down/sync/key-up/sync. The
transport attempts releases, permanently rejects later jobs, and the worker
closes its virtual device after any uncertain output. A regression test first reproduced
an invalid UTF-8 slice in the evidence predicate; the checked-slice fix passes.

Three additional pure Rust authentication tests cover root ownership/modes,
symlinks, bounded and malformed records, daemon UID/PID mismatch, kernel stat
parsing, and stale/reused PID start time. Matching three pure JavaScript tests
pass with `node --test native/gnome-extension/auth-policy.test.mjs`.

`cargo fmt --manifest-path native/wayland-helper/Cargo.toml` and
`node --check native/gnome-extension/extension.js` run successfully. These checks
are not Linux compilation or GNOME execution evidence.

## Required next native evidence

1. Compile/link on Ubuntu 24.04 amd64 and arm64 against systemd, AT-SPI, GLib and
   xkbcommon. Build/install/remove the generated Debian packages on disposable
   hosts. Confirm all systemd restrictions permit the intended native APIs.
2. Establish an actual per-field composition provider and narrowly defined
   positive toolkit/input-source profile. If this cannot be proved, review the
   backend design; do not replace Unknown with Idle based on a keyboard layout,
   a delay, two matching text samples, or Shell's own input-method state.
3. Qualify broker attestation publication/cleanup, startup acknowledgement,
   nondumpable `/proc/PID/stat` reads, root filesystem metadata, PID reuse and
   systemd runtime-directory behavior. Then qualify Shell context epochs across
   focus, overview, layout, session switches,
   lock, suspend, hotplug and extension reload; verify authenticated caller and
   revocation timing. Check every input loss and pending-output case.
4. Qualify AT-SPI traversal, object identity, protected fields, selections,
   committed suffix/caret and physical-event/uinput ordering in native Wayland
   and XWayland applications separately. No edit may precede final-key commit.
5. Exercise single-MIME clipboard snapshot/restore, concurrent copy, owner
   changes during deletion/paste, stalled transfers, disconnect and lock. Confirm
   owner-changed ordering, transfer size semantics and GJS introspected calls on
   GNOME 46. Multi-format restoration is intentionally unsupported.
6. Exercise expansion, immediate/delimiter/backspace correction, modifiers,
   cancellation between effects, Stop/start generations, queued physical input,
   partial output, and helper process death using disposable content and explicit
   user readiness. Type mode requires prevalidated keymap-representable payloads;
   general Unicode/newlines and arbitrary IMEs are not provided by uinput.

Task 4 and native platform acceptance remain open until these prerequisites are
satisfied. Do not publish a functioning-Wayland support claim from this source
preview, unit-test result, or a successful cross-platform build.
