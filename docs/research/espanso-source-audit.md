# Espanso source audit and Texpio engine options

24 October 2026 · Static source research; architecture recommendations remain proposals

## Conclusion

Espanso is useful engineering prior art for Texpio. Its strongest transferable design is a shared Rust event pipeline with independent native detection, application context, injection and clipboard backends. This supports keeping our Tauri/React/shadcn management UI while designing the expansion engine for Windows, macOS, Linux/X11 and Linux/Wayland separately.

Espanso obtains broad application compatibility through keyboard-derived text and configurable insertion strategies. It does not require a UI Automation provider to prove the committed abbreviation and caret before each Windows expansion. That differs from Texpio's approved P0 contract. Adopting its architecture does not establish identical safety guarantees, or prove that our implementation works across platforms.

Recommended direction: model those boundaries and compatibility controls in our own Rust engine first. Evaluate Raw Input as a Windows observer experiment, retain measured target/cancellation/outcome gates, and make any broader compatibility mode an explicit product/design decision. Direct source reuse or an Espanso-backed worker remains a separate option requiring a licensing and integration decision.

## Source and evidence

- Latest stable release returned by GitHub's API: [v2.4.1](https://github.com/espanso/espanso/releases/tag/v2.4.1), published 2 September 2026.
- Audited tag resolved on 24 October to [`a87d1723f47a59e38d370e2f0c636d66d568b8f6`](https://github.com/espanso/espanso/tree/a87d1723f47a59e38d370e2f0c636d66d568b8f6). The release API's `target_commitish` is `e6c3736675048026a057f1c4803d59242482fa7b`; we pinned the actual tag checkout rather than treating that metadata field as its resolved SHA. Binary-to-source correspondence was not verified.
- Development HEAD was [`fb90347ec0bdfceb0ea5cf1a1b36280fa2961463`](https://github.com/espanso/espanso/tree/fb90347ec0bdfceb0ea5cf1a1b36280fa2961463). Within the selected detection/injection/clipboard/engine/secure-input paths, stable-to-dev changes affect three X11 detection files. The newer change collects asynchronous X11 hotkey-grab errors and retains detection with a warning when a grab conflicts; the other application can still respond to the shortcut. It is not evidence of a new universal platform backend.
- Reference checkout: `.superpowers/reference/espanso-v2.4.1`, ignored by this repository. We inspected source, configuration, workflows and tests; we did not build/install/run Espanso, execute its scripts, arm input, or access the clipboard.
- Sources below link to the audited commit. Code presence and configured CI are distinct from observed runtime acceptance or passing remote jobs. No popularity ranking, zero-defect claim, or all-platform qualification follows from this audit.

## How the engine is organized

The core [engine loop](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/lib.rs#L34) composes a funnel, processor and dispatcher. Ordered [middleware](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/process/default.rs#L86) handles matching, rendering and effects; [events](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/event/mod.rs#L26) carry source/causal identifiers. These are useful boundaries for deterministic replay and cancellation tests.

| Boundary | Espanso implementation | Texpio lesson |
| --- | --- | --- |
| Detection | `espanso-detect` normalizes platform key/mouse events behind a source interface | Keep native capture and layout translation outside React and the pure matcher |
| Matching | Bounded rolling/history state; reset and Backspace behavior; literal/regex matchers | Use bounded state and explicit invalidation; add capabilities only at their reviewed phase |
| Rendering | Separate `Success`, `Aborted` and `Error` results; variable dependency resolution ([result contract](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-render/src/lib.rs#L26)) | Render/validate before destructive effects; cancellation preserves the original trigger |
| Effects | Dispatchers select key, clipboard, image, UI and other executors | Separate strategy policy from native execution; infrastructure does not imply every strategy is enabled |
| App configuration | The first matching app-specific configuration selects backend, exclusions, paste shortcut and timing; custom files are loaded without explicit sorting | Make supported profiles and deterministic precedence explicit; qualify per-app behavior ([upstream selection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-config/src/config/store.rs#L38)) |
| Lifecycle | Daemon supervises a worker; worker owns tray/UI loop and engine/detection threads; forms/search may use additional children | Give native resource ownership and shutdown a clear boundary; process separation is not itself bounded recovery |

The worker owns more than the engine: its [startup](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso/src/cli/worker/mod.rs#L111) also runs the tray/UI event loop. The [daemon](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso/src/cli/daemon/mod.rs#L140) handles configuration/layout restart paths; an unexpected worker exit can terminate the daemon rather than trigger an unlimited respawn loop. Espanso's crates and [IPC commands](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso/src/ipc.rs#L29) are useful integration evidence, not a verified stable embedding SDK.

## Native platform mechanisms

| Backend | Confirmed source behavior | Implications for Texpio |
| --- | --- | --- |
| Windows | A hidden window receives keyboard/mouse Raw Input. Foreground HKL, keyboard state and `ToUnicodeEx` derive characters. Orphan-device filtering plus expansion state suppresses many synthetic events. Injection uses UTF-16 `SendInput` events or clipboard paste. | Evaluate Raw Input alongside our hook observer. Test layout switches, AltGr, dead keys, software keyboards, event provenance and IME commits separately. Predicted characters are not proof of committed target text. |
| macOS | AppKit `NSEvent` global monitoring supplies key-derived characters; setup is dispatched to the main queue. Quartz posts generated key/Unicode events with a marker filtered by detection. Accessibility trust and Secure Input have separate flows. | Build a run-loop-aware adapter with permission states, synthetic-event handling and explicit composition/secure-context limits. It is not the same mechanism as a `CGEventTap` capture hook. |
| Linux/X11 | X RECORD/XKB capture and mapping updates; native injection uses character maps, `XSendEvent` or XTEST, with an xdotool alternative. | Qualify X-server extensions, layout handling and each insertion backend; a fallback is a separate tested capability. |
| Linux/Wayland | A Wayland build uses evdev capture and uinput virtual-keyboard injection. Character output depends on an XKB keymap. Layout and foreground-app discovery depend on compositor support. | Treat Wayland as a separate product/platform gate. Device permissions, representable characters, modifiers, compositor identity and app exclusions require independent qualification. |

Primary implementation references: [Windows detection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/win32/native.cpp#L130), [Raw Input registration](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/win32/native.cpp#L283), [macOS detection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/mac/native.mm#L35), [macOS injection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-inject/src/mac/native.mm#L26), [X11 detection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/x11/native.cpp#L96), [X11 injection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-inject/src/x11/default/mod.rs#L379), [evdev detection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/evdev/device.rs#L60), [uinput injection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-inject/src/evdev/mod.rs#L227).

Important platform qualifications:

- Windows layout caching defaults to two seconds. That is a compatibility/performance choice, not an authorization boundary; Texpio should invalidate queued work on a layout epoch change. See [refresh configuration](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-config/src/config/resolve.rs#L353).
- macOS native injection is [scheduled asynchronously](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-inject/src/mac/native.mm#L35); the [Rust return](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-inject/src/mac/mod.rs#L63) does not observe the target result. Its [Secure Input monitor](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso/src/cli/worker/secure_input.rs#L36) is a polled session status with uncertain attribution, not a password-field classifier.
- This tree's automatic Wayland [layout selection](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/layout/mod.rs#L38) uses the GNOME implementation. Other compositor integrations can provide [app information](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-info/src/lib.rs#L67), but unsupported environments can return no title/executable/class. App exclusions cannot be presumed enforceable there.
- Wayland initial [modifier synchronization](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-detect/src/evdev/sync/wayland.rs#L60) creates an XDG window and waits for a modifiers event without an explicit overall deadline. Texpio must retain bounded startup/stop behavior.
- When evdev is selected and the capability is available, Espanso temporarily raises [CAP_DAC_OVERRIDE](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso/src/cli/worker/engine/mod.rs#L321) while initializing native resources, then clears capabilities. Device-access permissions and privilege separation need their own design; this capability should not be added to the Tauri/webview executable by default.

## Behaviors we should preserve or improve

These differences explain the limits of source reuse. They are observed implementation choices, not claims that every Espanso user encounters a failure.

| Area | Audited behavior | Texpio disposition |
| --- | --- | --- |
| Target identity and deletion | Matching uses remembered input; trigger removal sends counted Backspaces without an exact current suffix/caret read | Preserve original target/generation checks. Any compatibility path without those observations needs a revised, explicit guarantee rather than silently inheriting P0's claims |
| Backend choice | Auto mode selects keys/clipboard by length; Linux also considers ASCII. Windows default threshold is 100 Unicode scalars | Separate configured selection from retry. Keep clipboard opt-in while preservation is unqualified |
| Submission result | The Windows native FFI ignores `SendInput`'s returned event count; the Rust wrapper returns success | Keep requested/submitted/observed outcomes distinct; partial submission stops the session without blind repair |
| Clipboard restoration | If text backup succeeds, waits then restores that text without owner/sequence comparison; otherwise skips restoration. Publishing empties the clipboard, so non-text formats cannot be restored and a newer copy can be overwritten | Retain our disabled state until supported-format preservation and concurrent-copy ownership are qualified |
| Sensitive context | Windows/Linux Secure Input monitoring is a no-op in this path | Preserve explicit password/composition/integrity checks where enforceable; report unknown capability honestly |
| Modifier timeout | The three-second release wait can expire and allow injection to continue | Refuse or cancel on timeout; do not continue with uncertain held modifiers |
| Rendering and cancellation | Trigger compensation is scheduled before rendering; an aborted render returns NOOP without undoing already-scheduled compensation, while render failure can insert error text | Prepare/render first, then revalidate before deletion. Cancel preserves input; errors stay out of the target document |
| Success and undo | Injected/undo state can be produced from rendering before native acknowledgement | Arm success-dependent state only from an appropriate completion result; qualify emoji/combining deletion units |

Sources: [trigger compensation and state](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/process/middleware/action.rs#L61), [backend policy/executor](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/dispatch/executor/text_inject.rs#L66), [native submission](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-inject/src/win32/native.cpp#L42), [clipboard restore guard](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso/src/cli/worker/engine/dispatch/executor/clipboard_injector.rs#L181), [Windows clipboard formats](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-clipboard/src/win32/native.cpp#L45), [secure-input platform branches](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso/src/cli/worker/secure_input.rs#L29), [modifier delay](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/process/middleware/delay_modifiers.rs#L54), [render outcomes](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/process/middleware/render.rs#L88), [undo state](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/process/middleware/undo.rs#L52).

Trigger-compensation ordering is shown in [cause processing](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/process/middleware/cause.rs#L47) and [Backspace generation](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/process/middleware/action.rs#L118).

## Proposed Texpio model

```mermaid
flowchart LR
    UI[Tauri management UI] <-->|Closed commands and coarse status| S[Native supervisor]
    A[Platform observer and layout adapter] --> N[Normalized native events]
    N --> M[Bounded matcher]
    M --> R[Render and prepare]
    R --> G[Target and cancellation gate]
    G --> I[Selected native insertion backend]
    I --> O[Submitted and observed outcomes]
    O --> S
    S --> A
```

This is a proposal, not an approved replacement spec. The native engine owns input, matching buffers, target tokens and OS resources. React manages snippets/settings/status through the existing typed boundary. Tauri's cross-platform window shell does not supply cross-platform text expansion.

| Component | Responsibility |
| --- | --- |
| Shared core | Bounded recognition, explicit reset/epoch rules, candidate IDs, rendering and deterministic policy; no OS handles |
| Platform adapter | Capture, layout/composition evidence, foreground context, native permissions and insertion; four implementations rather than one generic Linux adapter |
| Capability report | Per-capability available, permission required, unsupported, temporarily blocked or unknown; include reason and backend identity |
| Expansion job | Original target/context/input/layout generations, rendered payload and deadline; cancellation before destructive effects |
| Effect result | Distinguish prepared, requested, submitted, refused, partial/uncertain and observed completion; no fabricated success |
| Compatibility profile | Tested app/control/backend combinations, exclusions and timing; deterministic priority and explicit fallback consent |
| Supervisor | Bounded start/stop, fault acknowledgement, resource ownership, explicit rearm; worker process isolation where justified |

The current P0 recognizer, cancellation gate, redacted IPC and original-target checks are useful starting material. The Notepad read-only result remains evidence for that exact fixture. VS Code and Chrome are not newly qualified by reading Espanso. Our current strict UIA contract and a broader keyboard-history compatibility mode are different product promises; choosing between or combining them requires a reviewed amendment.

## Reuse choices

| Approach | Benefit | Cost and unresolved decision |
| --- | --- | --- |
| Independent Texpio core modeled on the architecture — recommended starting point | Retain Tauri integration, existing guard/outcome semantics and product control; independently implement general design patterns | We own native implementation and qualification; broad compatibility is not immediate |
| Reuse selected Espanso crates/native code | Potentially reuse substantial platform and matcher work | GPL licensing/distribution decision, native C/C++/Objective-C build ownership, internal API maintenance, and adaptation of failure semantics |
| Espanso-backed worker/sidecar with Tauri management UI | Fastest candidate for a comparative compatibility spike using an existing engine | Internal CLI/IPC and configuration are coupled to Espanso's lifecycle/UI; no drop-in SDK is established. It does not satisfy our stronger P0 contract unchanged; licensing and packaging remain decisions |

The [package](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso/Cargo.toml#L5) declares GPL-3.0; audited [source headers](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/lib.rs#L1) specify version 3 or later, with the complete [license](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/LICENSE) in the root. Direct reuse needs a deliberate licensing/distribution review before code is ported. Texpio's original product license remains undecided. This research adds descriptions and references, not copied Espanso runtime code.

## Suggested sequence and acceptance

1. Choose the reuse direction and explicitly settle the strict-versus-compatibility contract. Then write the revised engine design/implementation plan; retain the approved Windows-first, macOS-next delivery order.
2. Windows: compare Raw Input with the existing observer on fixed fixtures; test layout/AltGr/IME, software input, focus/control changes, cancellation, modifier release, full/partial submission, exact output and teardown. Keep clipboard and captured-target picker as separate experiments.
3. macOS: add its adapter and user-triggered permission flow; qualify native app controls, Secure Input, layout/composition, event filtering, asynchronous completion and stop/restart on actual Macs.
4. Linux/X11: qualify server/extension availability and insertion alternatives on declared distributions/apps. Linux/Wayland: first choose device-access/privilege boundaries and compositor capabilities, then test layout, foreground exclusions, representability, device reconnects and modifier synchronization.
5. Publish a measured compatibility matrix and known refusals by OS/backend/app/control. Packaging, permissions, native interaction, physical trials and CI have separate evidence rows.

Espanso itself documents [platform tiers](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/docs/src/ch02-01-platform-tiers.md) and a non-exhaustive [manual testing checklist](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/docs/src/ch05-01-manual-testing.md). Its configured PR CI differs by platform: Linux tests exclude modulo/IPC, macOS runs workspace tests, Windows PR jobs build/lint; Windows release workflows also run tests with exclusions. See [PR CI](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/.github/workflows/ci.yml#L61), [ARM CI](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/.github/workflows/ci-arm.yml#L60), and [release tests](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/.github/workflows/create-release-draft.yml#L86). These are inspected configurations, not verified recent run results.

P0 remains **0/300 physical automatic trials**, with no qualified insertion strategy. This audit informs the next design decision; it does not complete that gate. Existing [P0 evidence](windows-expansion-probe.md), [feature plan](../feature-plan.md) and [phase summaries](../phased-prds.md) remain the product record.
