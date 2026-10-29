# Native expansion on macOS and Linux

30 October 2026 · Design for review

Independent builder review is complete. User approval of this written design is
pending; no native implementation or platform qualification is claimed.

Texpio will retain its independent Rust product engine and add macOS, Linux X11
and Linux Wayland adapters. The user requested both platforms, source comparisons
with Ampello and Espanso, and x64/ARM64 releases. The user explicitly selected an
optional, separately installed Wayland input helper with one-time administrator
setup. The Tauri app will continue to run as the normal desktop user.

## Intent and scope

Make the existing snippet library useful for native expansion on the three desktop
operating systems. Preserve Windows behavior, the current library and editor,
immediate and delimiter activation, correction with Backspace, Pause/Stop,
cancellation, configured insertion modes and clipboard ownership safeguards.

There are six binary targets, but four input backends: Windows, macOS, X11 and
Wayland. Linux distributions and applications are qualified individually. Building
an ARM64 binary does not establish native interaction acceptance on that hardware.

The initial foreign-platform qualification scope is plain-text expansion and the
existing explicit Copy picker. Rich clipboard formats, file attachments, Unicode
trigger deletion and clipboard hotkey delivery require their own capability rows;
unsupported operations must refuse before deleting a trigger. Existing Windows
capabilities remain available. Rich snippet editing and portable library storage
remain shared regardless of the native delivery capability.

No app-store submission, signing purchase, publication, upstream engine embedding,
global-tool upgrade or automated physical typing campaign is part of this design.
The stricter Windows Developer/P0 probe stays separate and Windows-specific.

## Existing implementation

- `product/service.rs` owns publication tickets, generations, serialized effects,
  pause/stop and immutable rule sessions. Its runtime is Windows-only; other OSes
  currently return `ProductError::Unavailable`.
- `product/matcher.rs` and `product/effects.rs` already separate matching from
  ordered native effects. The current matcher assumes the completing key can be
  suppressed: immediate activation erases trigger length minus one, and a pending
  Backspace that reveals a match uses trigger length plus one.
- `product/hooks.rs` suppresses that key only after accepting its exact bounded
  job. `product/windows.rs` rechecks target, cancellation, generation and modifiers
  between effects. The product engine uses remembered typing; it does not prove
  the current document suffix or final output through UIA. Those stronger checks
  belong to the independent probe.
- Product clipboard paste is already enabled. `product/clipboard.rs` uses owner
  and sequence checks before restoration. Older audit/security prose describing
  disabled automatic clipboard insertion needs to distinguish the probe from the
  current product.
- Shortcut/autostart initialization and settings application are gated to Windows
  in `lib.rs` and `product/commands.rs`. Shared `cancel_vk()` validation currently
  embeds Windows key codes. The macOS picker already has an NSPanel implementation.

## Approaches

| Approach | Benefits | Costs and decision |
| --- | --- | --- |
| Independent native adapters and a separate Wayland helper | Preserves Texpio's library, matcher, settings and failure boundaries; per-platform capabilities | Selected direction, consistent with the established independent-engine choice and the user's helper selection; native implementation and qualification remain ours |
| Reuse Ampello or Espanso implementation modules | Existing capture and delivery code | Both sources declare GPL terms; reuse would require a deliberate distribution/license decision and adapting their target, cancellation, clipboard and shutdown behavior |
| Run Espanso as an engine sidecar | Existing multi-platform engine | Adds another lifecycle, configuration format, UI/IPC surface and packaging model; no stable embedding SDK was established; does not preserve Texpio's product behavior directly |

Learn the OS mechanisms and callback/worker separation from both references.
Do not copy runtime source into Texpio. Keep the inspected upstream checkouts
unchanged and record the revisions used for each implementation decision.

## Shared engine changes

Introduce a small platform facade under `product/platform/`. Windows initially
wraps its current runtime and effects to minimize changes. Each adapter supplies
runtime start/acknowledged stop, opaque native target identity, keyboard/layout
state, capture disposition, native effects, clipboard leasing and startup support.
Keep settings, rules, payload preparation, library ownership and effect ordering
in the existing shared modules; avoid a replacement engine framework.

The normalized input contract must explicitly describe whether the triggering
event is pending and suppressible or has already been allowed through. An
observation means the event was delivered into the desktop input path, not proof
that an editor committed text. Match candidates describe the logical trigger;
the effect plan derives deletion from that disposition. Failed queue admission
must leave a pending event untouched and reset ambiguous history.

Examples for an ASCII `;hello` trigger:

| Event state | Required erasure |
| --- | --- |
| Final `o` pending and successfully suppressed | Five characters |
| Final `o` already allowed through | Six characters |
| Space delimiter pending and suppressed | Six characters, then optionally forward Space |
| Space delimiter already allowed through | Seven characters, then optionally reinsert Space |

Tab/Return delimiters can move focus before an observation-only worker starts.
They must not be handled like a harmless printable delimiter: require a proven
same-field capability or refuse/reset without deleting. Backspace correction
needs equivalent pending/delivered cases. Start with qualified ASCII triggers on
new backends; do not equate Unicode scalars with editor deletion units or change
the stored trigger schema to hide an unsupported native operation.

Retain one bounded expansion job, preparation before deletion, target/generation
checks between effects, no forced modifier release and no automatic retry after
partial or uncertain output. Track refused, requested/submitted and uncertain
states distinctly; never describe posting input as observed document success.
Reset history on navigation, pointer action, focus/layout change, composition,
unknown translation, input loss, suspend/lock and device reconnect. evdev queue
overflow (`SYN_DROPPED`) invalidates recognition until key/modifier state is
resynchronized; hotplugged keyboards start with a fresh epoch.

Translate the persisted cancel-key strings into a platform-neutral enum and map
them in each adapter. Preserve old settings files. Split portable Tauri shortcut
registration from Windows registry startup and native clipboard dispatch; retain
the existing transactional controller and rollback behavior.

## macOS adapter

Use an owned Quartz session event tap on a dedicated CFRunLoop, as illustrated by
Ampello, with callback work restricted to bounded recognition and queue admission.
An active tap can suppress the completing event; retain exactly-on-accepted-job
semantics. Mark generated events to prevent recursive recognition. Own and release
the tap, run-loop source, callback state and worker with acknowledged shutdown.
Tap disable/timeouts invalidate history; permission revocation stops expansion.

Expose explicit Accessibility and input-listening/posting permission states.
Status reads and background startup must not prompt repeatedly. The user opens
the permission flow from Settings, then explicitly retries readiness. Preserve
fresh-launch automatic expansion only when prerequisites are already satisfied.

Prepare target context outside the tap callback: frontmost process plus focused
AX element when available, with bounded native queries. Recheck it before effects.
Secure Input, protected fields, composition and unsupported input-source states
block recognition/insertion; do not infer permission or text commitment from
key-code translation. Initial supported layouts and controls are positive test
profiles, not a claim of every IME or application.

Use marked Quartz Unicode/key events for Type, with qualified newline semantics
and pacing. Implement paste through native NSPasteboard and Cmd+V, not shell
commands. Snapshot bounded materialized formats and compare `changeCount` before
restoring. An incomplete snapshot with restoration requested refuses before
erasure; a concurrent user copy is never overwritten. Fixed delays are bounded
settling periods, not paste acknowledgement.

## Linux X11 adapter

Use X RECORD observation, XKB-aware layout translation and XTEST input. Maintain
separate recording/control connections, query required extensions at startup and
report unavailable capabilities. Observe focus/window and layout changes. The
observation-only completion path uses the delivered-event accounting above.

Use a native selection owner for clipboard paste. Before taking ownership,
materialize the supported prior MIME targets within bounded transfers and
timeouts; if restoration is requested and any required format cannot be captured,
refuse before erasing. Saving a previous owner's window ID cannot restore that
owner: taking the selection sends SelectionClear and may terminate its provider.
Restore captured data through Texpio's own selection source, only while the
XFixes owner/timestamp lease is still current. A concurrent user copy wins.
Ownership comparison and restored-snapshot publication must be atomic against
another client's ownership change; query-then-unconditional-set is insufficient.
Qualify this behavior under clipboard managers, lazy/large formats and owner exit.
Do not promise arbitrary-format restoration or depend on `xclip` processes.

Prefer paste for Unicode payloads when its lease can be prepared. Type mode only
uses keys representable by the qualified active keymap; prepare the entire body
before erasing. Do not silently use Ctrl+Shift+U sequences, change the global X
keymap or switch modes after deletion. Unsupported Type payloads produce a clear
reason and leave the trigger intact.

An X11 connection under a Wayland desktop is not a native Wayland capability.
Session selection must detect this and never call an XWayland-only observer
universal Linux support.

## Linux Wayland helper

The selected helper route uses evdev for observation and uinput for a virtual
keyboard. It must not exclusively grab and replay the user's entire keyboard.
Physical input continues to the compositor; match planning uses the delivered
event contract. evdev observation can precede compositor delivery, so it is not
permission to erase immediately. Initial Wayland profiles must establish the
trigger's actual committed suffix, caret and empty selection through bounded
AT-SPI text evidence on the same focused field before preparing destructive
effects. Check context/layout/input generations again at that boundary. A sleep,
uinput write or receipt of a physical key is not commitment acknowledgement.
Unsupported controls or expired evidence refuse without deletion. Exclude
Texpio's virtual device from capture and release every synthetic held key on
cancellation/teardown.

Package a separate native helper, never setuid or capability-enable the Tauri
executable. A minimal system broker owns privileged device discovery and creates
one session worker. The worker drops UID/GID/groups and unnecessary capabilities
after opening its approved seat devices; it retains only the needed descriptors.
The broker supervises/revokes the worker. No descriptor is handed to the webview
or a general user process. No raw input, snippet or clipboard content is logged.

Authorize only the active local graphical user's session. Tie the worker and
authenticated local IPC peer to UID, PID and logind session/seat. Refuse greeter,
remote, inactive or ambiguous sessions. Require an unlocked session and revoke
devices/worker on lock, session change, logout, app disconnect or heartbeat loss.
Active-session state alone is not sufficient proof of an unlocked screen.
If a desktop cannot supply trustworthy lock and context state, automatic input
is unavailable there. The broker must monitor revocation independently of the UI.

Expose a bounded, versioned protocol for status, normalized input and native
effect jobs with generation/deadline/cancellation. Reject unbounded frames,
unexpected file descriptors, arbitrary file paths and shell commands. Serialize
effects per session and fail closed on protocol/version mismatch. A reconnect
starts a new generation, without resuming any prior queued expansion.

The helper solves device access, not text semantics. XKB needs the actual active
layout/group; focused-app/field and protected/composition state require desktop
context integration. The initial proposed desktop is Ubuntu 24.04 GNOME 46.
Include an explicitly enabled GNOME Shell companion extension that provides a
bounded bridge for frontmost window/process, current input-source/group, lock and
overview transitions, and background clipboard selection operations. Use native
AT-SPI for qualified focused-field/protected-input evidence. A global compositor
window identity does not itself establish focused-field or composition safety.
Missing/ambiguous required context refuses automatic expansion.

Before building the complete helper integration, a feasibility task must prove
that bridge against the actual GNOME 46 APIs: focus/layout epochs, per-field
committed text/caret, protected/composition state, bounded clipboard transfer and
owner-change detection, concurrent-copy preservation and lock revocation.
Exercise physical-event/uinput ordering explicitly; the AT-SPI evidence gate must
prevent erasing a trigger before its final key has reached the editor. Current
Mutter documentation provides selection transfer,
set-owner and owner-changed primitives, but does not qualify their GNOME 46
runtime behavior. If the bridge cannot establish a required capability, stop that
backend at an honest unavailable state and report the precise blocker; changing
the initial compositor or weakening the contract requires review.

KDE and other compositors require their own bridge and acceptance rows; they
cannot inherit GNOME qualification. Native Wayland and XWayland client
applications also have separate rows. Do not infer layout from a hard-coded US
map or silently disable exclusions. The additional Shell extension is an agent
technical proposal; the user's confirmed decision was the separate input helper.

For Unicode paste use the GNOME bridge's selection operations with an ownership
lease; unavailable clipboard access is an explicit capability refusal. The bridge
must not expose a general command/evaluation interface or log clipboard content.
Authenticate its local caller and bound all IPC payloads. If clipboard restoration
is requested, materialize supported prior formats before erasure; refuse an
incomplete snapshot. Restoration checks the lease epoch, never overwriting a
concurrent copy. A non-GNOME backend can use an advertised data-control protocol
only after it independently passes the same ownership/format tests.
uinput Type only supports a prevalidated keymap-representable payload. General
Unicode text injection and IME surrounding-text access are not supplied by uinput.
Portal RemoteDesktop/libei injection alone does not replace passive trigger
observation; InputCapture is compositor-activated and diverts input.

The initial installer target for the helper is a native Debian package on
logind/systemd systems. AppImage remains a desktop UI download and can detect a
separately installed helper. No automatic privilege installation on app startup,
no blanket membership in the `input` group and no silent root fallback. Include
status, explicit setup instructions and a complete uninstall path.

## UI and native capability reporting

Add a typed, bounded capability report: backend, readiness, permission/setup
reason, supported input modes/payloads and available desktop actions. Keep native
tokens, events, clipboard values, device paths and accessibility objects in Rust.
Use the existing TanStack Query service and payload-free status invalidations;
do not put diagnostic/input text in Zustand.

Settings explains the next actionable step: grant macOS permission, install the
Linux helper, choose a supported session, or change an unsupported insertion mode.
Disable only unsupported actions; library editing and explicit Copy remain
available. A persisted unsupported option cannot display success as though it
had been applied. Every new visible string goes into all three root locales.
Preserve StrictMode/multiple-window cleanup and the picker capability boundary.

## Build and verification

Add non-publishing native CI for explicit Windows/macOS/Linux x64 and ARM64 Rust
targets. Keep pinned Bun 1.4.4, Node 24.18.0, Rust 1.99.1 and locked installs.
Prefer existing lockfile-qualified bindings, including objc2 0.6.5, its framework
crates 0.3.2 and x11rb 0.13.2 where suitable; add only required platform-scoped
dependencies/features after checking their exact APIs. The helper has Linux-only
build/package jobs. Existing release publication/signing gates remain separate.

Agent-selected starting test baselines: macOS 14+; Ubuntu 24.04 GNOME X11 and
Wayland on both architectures. Use the oldest compatible Linux packaging image
that meets native dependency requirements. Minimum supported OS versions remain
provisional until packaged tests pass; other distributions/compositors are
unsupported until added to the matrix.

Ordinary checks use fake adapters and never install global observers or inject
keys. Cover pending/delivered completion, correction, focus/layout/composition
invalidation, failed admission, self-event exclusion, queue overflow, permission
loss, Stop/start races, clipboard races, cancellation and partial effects.
Helper tests cover peer/session authorization, frame limits, malformed input,
revocation, disconnect, generation expiry and virtual-device exclusion.

On this Windows host run `bun run check:all`, regenerated bindings, the frontend
build and required locked Windows native build. Replace/restart only the exact
development process using the authorized Explorer-parent launcher, then perform
focused actual-app checks while leaving the preview open. Do not arm global input
in those ordinary checks. Build/check macOS and Linux on matching hosts/CI; mark
unavailable host checks as unverified rather than substituting compile guesses.

Native qualification uses disposable synthetic content and explicit readiness
for any user typing. Each OS/session/architecture needs installation, permissions,
actual expansion, pause/stop, restart, Unicode/newlines, focus changes, secure
fields, layout/composition, clipboard races and disconnect/suspend/lock tests.
Preserve existing P0 evidence as a separate gate. No zero-error or broad platform
support claim follows from unit tests or successful builds.

## Delivery order and completion

1. Shared event/effect accounting and capability boundary, with Windows regression
   checks; split portable desktop settings from Windows-native operations.
2. macOS runtime, permissions, native effects/clipboard and native CI.
3. Linux X11 runtime, layout/selection handling and native CI.
4. Prove the GNOME context/clipboard bridge, then implement the Wayland
   broker/worker, session integration and packaging. The bridge feasibility result
   is a prerequisite, not an assumption of native support.
5. Per-platform native qualification, six-target build evidence and documentation.

Stages may produce previews while remaining native checks are open. Completion
requires working native expansion on the declared profiles, documented refusals,
real platform test evidence and preserved Windows behavior. An unavailable helper
or desktop context may not be replaced with unguarded injection to pass a demo.

## Sources and implementation lessons

References were inspected as source, without executing upstream engines:

- Ampello at [`c6ba8ea4`](https://github.com/ampello/ampello/tree/c6ba8ea4a227b063df945597d1223f43f6e2f074):
  [Quartz tap and worker](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/mac/mod.rs#L235),
  [X11 RECORD/XTEST and Wayland refusal](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/linux/mod.rs#L139).
  Learn capture/effect separation and pacing; do not adopt its cancellation-only
  Mac/Linux guard, text-only clipboard restoration or unacknowledged shutdown.
- Espanso at [`a87d1723`](https://github.com/espanso/espanso/tree/a87d1723f47a59e38d370e2f0c636d66d568b8f6):
  [macOS observation](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/mac/native.mm#L35),
  [evdev observation](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/evdev/device.rs#L60),
  [uinput delivery](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-inject/src/evdev/mod.rs#L227).
  Learn platform separation and device handling; keep privilege away from Tauri.
- [Kernel uinput documentation](https://www.kernel.org/doc/html/latest/input/uinput.html)
  describes virtual input devices, not arbitrary Unicode or foreground context.
- [logind session APIs](https://www.freedesktop.org/software/systemd/man/latest/sd_session_is_active.html)
  distinguish active local sessions; screen lock requires additional integration.
- [InputCapture portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html)
  and [RemoteDesktop portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html)
  distinguish capture activation from consented injection.
- [Mutter selection API](https://gnome.pages.gitlab.gnome.org/mutter/meta/class.Selection.html)
  provides compositor selection transfer/ownership primitives. Its current API
  reference does not replace checking the targeted GNOME 46 implementation.
- Existing [Ampello audit](../../research/ampello-source-audit.md),
  [Espanso audit](../../research/espanso-source-audit.md) and
  [source comparison](../../research/cross-platform-expansion-source-comparison.md)
  provide detailed pinned references; their older descriptions of Texpio must be
  read against the current product/probe distinction above.
