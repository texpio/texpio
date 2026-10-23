# Texpio Windows expansion probe design

23 October 2026 · Native Rust approach selected by the user · Written design awaiting approval

## Purpose

Establish whether Texpio can safely replace an abbreviation in other Windows applications before building the persistent snippet manager. This is the P0 engineering milestone in the [phased PRDs](../../phased-prds.md), not a public text-expander release. The maintainer needs a runnable probe, repeatable cases, and a report that can support or reject the engine approach.

The user selected **a native Rust engine with Windows APIs** for this probe. Retain Tauri 2, React/TypeScript, shadcn, the pinned toolchain, and execution in this checkout. No backend enters this phase. The TextExpander screenshots supplied on 23 October inform the later library interface; their feature mapping belongs in [competitor analysis](../../competitor-analysis.md).

## Starting state and prerequisites

The clean starting checkout is commit `22fadcb`: foundation screen, typed metadata, themes, tray, single instance, checks workflow. The native preview has shown Connected, version `0.1.0`, platform `windows`. Tray lifecycle and remote CI still need their recorded acceptance evidence; do not infer those results from the commit or mark them complete as part of this design.

Before arming the input probe, finish the foundation's pending native acceptance checks and record the current local checks and remote CI result or exact limitation. Freeze the probe build, Windows build, app builds, control types, layout, and configuration in its report. Administrative OS/app inventory commands were denied in the sandbox during design research; this is an inventory limitation, not a missing compiler or runtime.

## Approaches

| Approach | Benefit | Tradeoff and disposition |
| --- | --- | --- |
| Native Rust engine through Microsoft's `windows` bindings | Own matching semantics, target checks, diagnostics, and lifecycle within Tauri; test pure engine logic independently | Native input complexity remains ours. **Selected by the user for P0**, with adoption still conditional on evidence. |
| Integrate an existing engine such as Espanso | Potential reuse of established expansion behavior | Integration API, process/configuration ownership, packaging, updates, and licensing need evaluation. Retain as a fallback; no bundled engine or copied code in this probe. |
| Picker-only insertion | Useful to examine target restoration and deliberate insertion | Cannot demonstrate automatic expansion. Include a small diagnostic experiment, not an alternative definition of P0 success. |

Microsoft's bindings use MIT/Apache-2.0 licenses. Espanso identifies GPL-3.0 licensing and YAML configuration. Record a source/API/license comparison, without claiming a supported embedding API or settling the project's own license. A runnable Espanso integration requires a subsequent scope decision. See [windows-rs](https://github.com/microsoft/windows-rs), [Espanso](https://github.com/espanso/espanso), and its [configuration documentation](https://espanso.org/docs/configuration/basics/).

## Probe behavior and limits

Start **Stopped** on every launch. The maintainer explicitly starts a test session; starting it does not enable automatic expansion in every application. Permit only the selected reference app/control profiles and synthetic test documents. Unknown profiles are refused. Pause is immediately available in the window and tray, and Quit stops all native workers.

Use a fixed in-memory fixture library rather than a database or editor. Proposed builder choices are six fixtures covering a short greeting, a synthetic address, accented Latin text, Devanagari text, an emoji plus combining sequence, and multiline plain text. Include a generated 2,000-character fixture for timing. Use synthetic content only, with stable fixture IDs for diagnostics.

Recognition is literal and case-sensitive. Start with US English keyboard layout `00000409`, ASCII triggers matching `;` followed by lowercase letters, at most 32 characters. **Space activates a complete trigger and one trailing space remains.** Start-of-field or preceding whitespace is required, verified from the supported control where possible. `;hello` followed by Space becomes `Hello from Texpio! `; `;Hello`, an incomplete trigger, and a trigger embedded in a word do not expand.

A matching Space key-down creates a pending candidate. Submit it to the insertion worker only after the corresponding Space key-up, with no intervening key-down, repeated Space, or unrelated input. Earlier trigger-key releases may update tracked state without adding text; require all trigger keys released before publishing the request. Any unexpected release or ambiguous pairing cancels the candidate. After publication, new physical input invalidates it. This explicit release ordering avoids treating the activating key-up as a new edit or injecting while a trigger key remains held; recognition timing begins when the completed request is published, and the report also records delimiter key-down to completion.

Backspace, Delete, navigation, selection, mouse activity, modifier shortcuts, focus changes, unsupported input, and layout changes discard the candidate; editing a partially typed trigger is deliberately unsupported initially. Cap the transient matching suffix at 33 ASCII characters including the delimiter. Generated output never enters matching.

No full library, SQLite, autosave, folders, rich text, images, fill-ins, date/math/keyboard tokens, script execution, production undo, import/export, accounts, sync, autostart, installers, or macOS adapter is included. Later UI work can adopt the useful organization/preview ideas from the screenshots using Texpio's own design.

## Initial compatibility candidates

These are test candidates, not supported-product claims. Exact app/control versions must be recorded before testing. Windows 11 x64 on the current development host is the initial environment; Windows 10, ARM64, remote desktops, and other layouts remain unvalidated.

| Reference | Positive candidate | Required negative cases |
| --- | --- | --- |
| Windows Notepad | Disposable plain-text document | Another document/tab, selection, caret movement, focus switch |
| Visual Studio Code | Untitled plain-text editor with unrelated extensions disabled in a disposable profile | Search/input panels, terminal, command palette, document switch |
| Google Chrome | Local offline fixture page with ordinary textarea and single-line text input | Password input, readonly/disabled controls, another field/tab/window; contenteditable exploratory only |

The browser page includes synthetic sentinel text around the insertion point and a second field for race tests. No remote website, account, or real customer content is used. The local fixture server must bind to loopback only.

P0 must establish whether each control exposes sufficient trustworthy accessibility/caret evidence. A process allowlist alone is insufficient. If a candidate lacks the evidence, decline it and report the missing capability. Do not loosen checks just to reach the three-app count; select and disclose another reference or leave the gate unmet.

## Architecture and input lifecycle

Keep a platform-independent matching/state module and Windows-specific adapter under Rust. React receives fixture definitions, aggregate counters, and bounded status DTOs through narrow commands; it never receives the keyboard stream, matching suffix, arbitrary target text, clipboard data, or full process paths.

A dedicated native thread owns low-level hooks and its message loop. A separate worker handles context/accessibility checks and insertion. UI Automation COM objects stay on their worker. Do not perform COM, clipboard, logging, disk/network operations, or blocking waits inside hook callbacks. Use bounded nonblocking communication; overflow invalidates the session's context and prevents pending mutation. Stop closes the input gate first, cancels queued work, unhooks, and joins workers without holding Tauri UI-thread locks.

Track a monotonically changing context generation and input generation. Each request carries its original target identity and generations. Physical input after a candidate, focus/control/caret changes, mouse/navigation activity, pause, layout changes, lock/suspend/resume, hook faults, and queue overflow invalidate queued work. Resume starts clean and requires explicit rearming after lock/suspend or a fault.

Skip all injected keyboard events and tag Texpio-generated events with `dwExtraInfo`; expected self-generated output must not create a new candidate or cancel its own insertion bookkeeping. Unexpected injected events invalidate context. This excludes many software keyboards and accessibility-generated keystrokes in the initial physical-input path. The tag is bookkeeping, not authentication. See [KBDLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct).

Initial US-layout recognition may use explicit key mapping rather than generic layout translation. Track physical modifier transitions, and refuse Ctrl/Alt/Windows/AltGr, Caps Lock, composition, and stateful layouts. Do not infer committed text solely from those transitions. If translation is needed, follow the documented non-mutating `ToUnicodeEx` behavior and reject dead-key/multi-character ambiguity; see [ToUnicodeEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex) and [IME composition](https://learn.microsoft.com/en-us/windows/win32/intl/wm-ime-composition).

## Eligibility and replacement contract

The physical delimiter initially passes through normally. The worker waits only within a bounded, measured deadline for the supported control to expose the committed trigger and space immediately before a collapsed caret. Capture the foreground window, process identity, focused control/accessibility identity, layout, selection/caret evidence, and generations. A missing/ambiguous result is a refusal before mutation. The implementer selects concrete supported UI Automation text-range/control adapters; it must not fall back to deleting a remembered number of characters without checking the actual suffix.

Require an editable, enabled, non-password control in a permitted profile, an unchanged target/caret, the exact expected trigger suffix and boundary, and no held modifiers or unsupported composition. Avoid reading an entire document to perform these checks; inspect only the bounded range needed for the candidate and caret. Target text is transient and must never enter diagnostic output.

Password detection must distinguish explicit false from an unsupported property. Use `GetCurrentPropertyValueEx` with `ignoreDefaultValue=TRUE`, recognize the NotSupported sentinel, and refuse unsupported/error results. Provider-reported false is not a universal secure-field guarantee. Also reject recognized secure controls, Texpio's management UI, excluded applications, elevated targets, and unknown integrity/focus/caret states. See [UI Automation property retrieval](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-propertiesforclients).

Immediately before mutation, recheck the target, context/input generations, exact suffix/caret, and modifier state. Construct one bounded input batch containing only deletion of the verified ASCII trigger plus delimiter, followed by UTF-16 output and one trailing space. Unicode output uses key-up/down pairs, with special attention to surrogate pairs, combining sequences, and line endings. See [KEYBDINPUT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput).

`SendInput` is foreground-directed, has no target-window argument, and is restricted by integrity level. A successful submission count is not proof that the expected control committed the text. Record dispatch separately from observed completion. A focus check and injection are not atomic; the race campaign below determines whether this approach can advance. See [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) and [GetGUIThreadInfo](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getguithreadinfo).

Partial submission, unexpected text, provider timeout, or changed focus after dispatch yields **Outcome uncertain** and stops automatic expansion until reset/rearmed. Never issue compensating backspaces, replay the batch, or claim the trigger was preserved after mutation has begun. A refusal before dispatch must leave the user's trigger and delimiter intact.

## Direct and clipboard insertion experiments

Direct Unicode insertion is the first implemented strategy. Test UTF-16 output independently of trigger recognition; Unicode bodies do not establish Hindi/CJK/IME trigger support. Compare actual text, including trailing space, line endings, emoji, and combining sequences without silently normalizing mismatches away.

Clipboard-assisted insertion is a separate, explicitly selected experiment after direct insertion works. It is disabled by default. Before changing clipboard or target text, enumerate formats and refuse anything the probe cannot preserve; never replace an image, file list, private format, or delayed-rendering data with a text-only backup. The initial implementation may support an empty clipboard and a fully snapshottable plain-text format set only, declaring other states unsupported.

Use clipboard ownership and sequence checks around publishing/pasting/restoring. Verify paste consumption through the supported target's text evidence rather than a fixed sleep. Restore only if Texpio still owns the expected clipboard version; a concurrent user copy wins. Clipboard locks, changed ownership, unknown formats, and timeout cases must be reproducible negative tests. Content changes between checking and pasting remain a race to measure, not a solved property. If this cannot be made reliable, document that clipboard insertion does not qualify for P1; do not ship it as an invisible fallback. Clipboard history/managers may retain temporary values. See [clipboard operations](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-operations) and [sequence numbers](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboardsequencenumber).

## Maintainer controls and picker experiment

Extend the foundation with a compact **Windows expansion probe** panel: Stopped/Running/Paused/Fault status, Start/Stop, Pause/Resume, fixture list, insertion strategy, current refusal reason, and aggregate submitted/observed/refused/uncertain counts. Describe Running as an installed observation session, not proof of continuous hook health. Show latest event activity without displaying keys or captured text. Low-level hooks can be silently removed on timeout; an idle heartbeat cannot prove health. Offer Stop/Start recovery and report the limitation. See [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc).

Tray controls mirror Pause/Resume and Stop while retaining Open and Quit. Session-only app exclusions and explicit reference-profile selection suffice. Quitting or stopping must leave no hooks, pending insertions, or clipboard restoration job that could later overwrite the user's copy. Repeated command requests are idempotent or return a stable busy/stopped error.

For the picker experiment, register a configurable test shortcut only during an armed session, initially Ctrl+Shift+Space if available. Capture the original target before showing a small fixture chooser. Escape cancels with no insertion. Selecting a fixture attempts restoration of that captured target and repeats the permitted-profile, editability, password, integrity, modifier, and target/caret checks; it must not paste into whichever window is currently foreground. A stale or destroyed target is refused. Record shortcut conflicts, restore behavior, and whether target/caret evidence survives opening the chooser. No full search/library workflow is required.

Picker selection inserts **only the fixture body at the original collapsed caret**, deletes no text, and adds no automatic trailing space. It does not require an abbreviation suffix. Store a separate picker target token containing the original control/caret and a bounded adjacent-text anchor; expected focus transitions through the chooser do not preserve or reuse the automatic recognizer's generation. After restoring the target, require the original caret/anchor evidence to match and invalidate the token on any intervening target edit, uncertain observation, or unexpected focus transition. Refuse if this cannot be established; never use blind caret-coordinate restoration. These picker semantics are a builder call for the experiment, not the final P1 picker contract.

Use shadcn components and the existing themes; keep buttons keyboard-accessible, states textual, and errors concise. Browser-only preview shows native controls unavailable instead of simulating working expansion. Do not introduce a competitor-style management interface or decorative nonfunctional controls in P0.

## Verification and evidence gate

The probe is complete only when its report provides the evidence below, including failures. Unit tests and a successful build do not replace native acceptance.

1. **Pure engine and boundary checks:** matching, delimiter/boundary/case behavior, buffer limits, state resets, injected-event handling, stale generations, pause/exclusions, UTF-16 generation, unsupported context and partial-submission behavior; validate narrow IPC contracts and controls. No native hook or input injection runs in ordinary unit tests or CI.
2. **Foundation regression:** native metadata/themes, close-to-tray, hidden/minimized restoration, single instance, Quit/relaunch; existing frontend/Rust/build checks and CI diagnostics. Record each check against the actual build.
3. **Positive campaign:** at least 300 eligible **physical-input automatic end-to-end** direct-insertion attempts, at least 100 per reference application, distributed across the fixture bodies and typing speeds. Synthetic trials are additional and do not contribute to this count. Check exact output plus unrelated sentinel text and target identity. Report each application's exact completion rate; eligible refusals count as unsuccessful completions. Report clipboard trials separately with their own denominator and supported format states; an unqualified strategy does not inherit direct-mode results.
4. **Separate negative campaign:** focus changes at pre-check/pre-dispatch/post-dispatch boundaries, same-process field/tab switches, caret moves/selections, mouse input, modifiers, pause/stop races, excluded/password/unknown/elevated controls, layout/IME changes, queue overflow, provider failure, partial insertion, clipboard contention/concurrent copy, repeated Start/Stop, Quit, and suspend/resume. Preserve surrounding sentinel text in every case.
5. **Picker experiment:** correct captured-target restoration, cancel, shortcut conflict, changed/destroyed targets and lost caret evidence; never count picker insertions toward automatic expansion attempts.
6. **Timing/resources:** record recognition-to-dispatch and recognition-to-observed-completion separately, with p50/p95/max and per-app sample counts; record idle CPU and combined process memory on the actual host. Include the 2,000-character fixture. Future P1 reference-machine budgets remain proposed, not achieved by this probe.

Input generated by UI automation commonly has the injected flag and is intentionally ignored by the physical-input path. Keep pure-event tests, synthetic native insertion trials, and physical typing end-to-end trials as separate evidence classes. Any harness-only acceptance of tagged synthetic input must be explicitly enabled in a test build/session and absent from the normal path. The report must state which path each attempt exercised; synthetic trials alone do not validate physical observation or satisfy the automatic-expansion gate. If physical trials or a reference application cannot be exercised, leave the corresponding gate pending.

Require **zero observed wrong-target insertions and zero unrelated-text deletion** across all positive and negative cases. Any such incident stops the campaign, preserves a synthetic reproduction, and blocks advancement until the defect is fixed and affected campaigns repeated. This is a measured gate, not a guarantee of universal Windows safety. Hook installation does not imply committed-text recognition, and queued input does not imply a successful expansion.

## Report and phase decision

Write `docs/research/windows-expansion-probe.md` during execution with the tested commit/build, environment and controls, fixture IDs, method and evidence class, per-app counts/results, timings, resource observations, refused contexts, reproducible failures, dependency/license review, and native-versus-existing-engine assessment. Store only synthetic test evidence; no captured typing history or personal clipboard content. Distinguish observations from architectural inferences.

Choose one explicit outcome: **advance with the validated native adapter/strategy and declared limits**, **continue the probe with named unresolved cases**, or **revise the architecture/scope**. P1 planning starts only after the P0 evidence gate and support contract are reviewed. Feature-rich screenshots do not change that ordering.

## Calls left to the implementer

Select the current compatible stable `windows` crate version and narrow API features, then lock dependencies. Choose module/type names, bounded queue sizes, measured deadlines, COM worker mechanics, supported control-range adapters, and synthetic test harness details. Keep the gates and refusal semantics above intact. Do not relax target evidence or expand platform/feature support as an incidental implementation choice.

This design uses maintainer test sessions, fixed fixtures, US-layout ASCII recognition, direct Unicode first, a separate clipboard trial, and a small picker restoration experiment as proposed scope decisions. Written-design approval covers these choices; implementation planning follows that approval.
