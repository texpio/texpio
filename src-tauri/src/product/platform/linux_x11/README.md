# X11 adapter status

This is an **unqualified adapter**, not enabled native expansion support. Both
`readiness()` and `Runtime::launch()` refuse with `InputBlocked` because no
qualified direct-input provider currently establishes positive composition-idle
evidence. AT-SPI Text does not supply that contract. `context::provider_ready`
is an explicit functional gate; do not remove it merely to run a typing demo.

The prepared runtime uses separate RECORD, control and effect connections,
Xlib's XKB lookup, and XTEST without changing the global keyboard mapping. It
requires a native X11 session and refuses XWayland/unknown sessions. Its initial
trigger-deletion profile is ASCII. It refuses rich payloads and unrepresentable
Type payloads before deletion. A Type payload is fully mapped before erasure;
it never falls back to another insertion mode.

Generated events are attributed to the exact effect connection through recorded
FakeInput requests, rather than excluding a key solely because it resembles a
planned event. Foreign XTEST injection stops the runtime: delayed foreign input
cannot safely be attributed to hardware. Physical input while an effect is busy
cancels its generation. This conservative behavior still needs native RECORD
ordering qualification, including capture backlog and repeat behavior.

Field checks use the shared AT-SPI tracker on one dedicated context thread.
Password, unknown-role, readonly, ambiguous-focus, selected-text, unknown
composition and unknown/locked-session evidence all refuse. Capture/effects
share the tracker's strongly referenced field token. Before erasure the worker
requires the committed ASCII suffix and caret captured at admission. X focus,
PID, active XKB group and keyboard map are checked between effects. Native
field queries, socket interruption and acknowledged stop need Linux validation.

Clipboard paste materializes a bounded set of prior targets before erasure.
INCR, incomplete, unresponsive or excessively large formats refuse restoration.
The native selection source serves captured formats and remains alive for the
runtime lifetime. Restoration changes that owned source under a server grab,
after checking XFixes owner/timestamp; it never reclaims ownership after another
client copies. Its 200 ms settling window means input submission, not editor
acknowledgement. Clipboard-manager behavior, source retention across Stop/exit,
lazy providers, owner exit and concurrent copies remain native qualification
items. Restored source persistence after runtime destruction is not established.

The implementation uses local x11rb 0.13.2 protocol bindings and independently
written Xlib/AT-SPI integration. No Ampello/Espanso implementation was copied.
Native links require libX11 and the helper crate's libatspi, GLib/GObject and
systemd development libraries. The helper library reuse does not require
installing or starting the privileged Wayland input daemon for X11.

Acquisition also retains the old owner/timestamp token throughout materialization,
including empty and self-owned selections. Immediately before taking ownership,
it compares that token under the same server grab as SetSelectionOwner. A copy
between snapshot/preflight and publication refuses before erasure. Possibly held
synthetic keys are tracked before submission; failed down/up requests trigger
best-effort release and a sticky runtime fault that prevents subsequent jobs.

## Observed verification on Windows

- Eleven policy tests pass via standalone `rustc --test policy.rs`, covering
  session selection, clipboard bounds/epoch, RECORD request attribution and
  framing, acquisition races, sparse keymaps, uncertain key releases and
  whole-payload Type representability. No test connects to X or
  sends input.
- A temporary Cargo harness compiled this adapter against exact cached x11rb
  0.13.2 APIs plus the actual shared matcher, settings and effects. Its 36 tests
  passed initially; the review-fix rerun passed 41 tests. Windows compilation excludes native Linux FFI, so this is not a Linux
  compile or native interaction result.
- No native Linux compilation, active capture, physical typing, clipboard-owner
  campaign or app qualification has been performed here.

Primary API references:
[XKB lookup](https://www.x.org/releases/current/doc/libX11/XKB/xkblib.html),
[x11rb 0.13.2](https://docs.rs/x11rb/0.13.2/x11rb/),
[AT-SPI roles](https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/enum.Role.html),
[AT-SPI timeout](https://docs.gtk.org/atspi2/func.set_timeout.html).
