# Ampello source audit and lessons for Texpio

24 October 2026 · Source inspection, not upstream runtime qualification

## Finding and reproducibility

Ampello is a close product and technical reference: an offline snippet library built with Tauri 2, React/TypeScript, Rust and SQLite. It has real native Windows, macOS and X11 implementations. Its Linux adapter explicitly refuses Wayland. It offers useful insertion pacing and cancellation mechanisms, but its success condition is weaker than Texpio's exact observed-output contract.

The user requested this clone and analysis. The reference is at ignored `.superpowers/reference/ampello`, pinned to [`c6ba8ea4a227b063df945597d1223f43f6e2f074`](https://github.com/ampello/ampello/tree/c6ba8ea4a227b063df945597d1223f43f6e2f074), the remote HEAD observed during this inspection. The package version is 0.4.0. No upstream installer, application, build script or test was executed, and no upstream runtime code or dependencies were imported into Texpio. Read alongside the [Espanso/Snipt comparison](cross-platform-expansion-source-comparison.md).

## Architecture worth learning from

| Boundary | Ampello implementation | Application to Texpio |
| --- | --- | --- |
| Shared core | `ampello-core` owns matching, a bounded character-history buffer, storage and backups without Tauri/native dependencies | Preserve our independent matcher/render core and test it without installing hooks |
| Capture versus work | Windows hook translates input and queues an expansion; a separate worker loads content and performs effects | Keep blocking providers, database reads and injection outside callbacks |
| Platform boundary | A common `InputService` interface selects Windows, macOS, Linux or an unsupported-platform stub at compile time | Retain capability-specific native adapters; platform names alone do not establish support |
| UI/native boundary | Central TypeScript IPC module and Rust-owned persistence; events notify the interface | Retain generated typed bindings, native authority and TanStack Query; Ampello's Zustand data cache does not supersede our state rules |
| Lifecycle | Tauri setup starts the input service; exit requests shutdown; tray/settings expose enable/restart/status | Preserve Texpio's explicit Start and stopped launch. Lifecycle completion must be acknowledged, not inferred from a posted stop message |

Sources: [core manifest](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/crates/ampello-core/Cargo.toml), [platform selection](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/mod.rs), [Windows service](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/mod.rs#L68), [IPC facade](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src/lib/ipc.ts), [Tauri integration](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/lib.rs#L104).

The matcher indexes triggers by text and tries distinct character lengths longest first, checking the preceding word boundary. This is a useful compact alternative to a trie; its allocation and distinct-length cost should be measured against our workloads before choosing it. A source benchmark is not a whole-application latency result. [Matcher source](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/crates/ampello-core/src/engine/matcher.rs#L66).

## The Windows expansion path

Ampello uses a low-level keyboard hook and foreground-layout `ToUnicodeEx` translation with the no-dead-key-state-change flag. On a match it suppresses the terminating key, then queues the snippet ID and foreground HWND. Its worker loads the current snippet, deletes the trigger and delivers the replacement; it can subsequently forward the terminator. Texpio observes Space without suppressing it and verifies/deletes the trigger plus that space. These are different recognition/effect contracts. [Hook and translation](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/mod.rs#L293), [expansion worker](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/mod.rs#L521).

Two insertion choices matter:

- **Paste is the default.** The delivery path temporarily writes clipboard text, sends Ctrl+V and attempts restoration after fixed waits. This differs from Texpio's approved direct-Unicode backend and disabled automatic clipboard fallback. [Configuration](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/config.rs), [delivery](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/inject.rs#L475).
- **Type is paced.** Each Unicode scalar becomes one `SendInput` call containing its UTF-16 press/release pairs. Rates are 500/300/120 events per second, approximately 250/150/60 BMP characters per second; supplementary characters require more events. Backspaces are a separate effect. This gives a concrete comparison with our combined batch. [Typing loop](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/inject.rs#L409).

The Windows injector checks returned submission counts. Its hook counts marked events returning through the hook, and a drain meter limits outstanding events; if that meter times out, it switches to timed pacing. Long jobs also have a polling cancellation watcher, and the hook can discard remaining marked events. These are useful queue/cancellation mechanisms, but hook passage is not acknowledgement by the receiving application. The receiving control may still have pending work. [Meter and cancellation source](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/inject.rs#L86).

Its Windows guard compares foreground and focused HWNDs, not the caret offset inside a field. The inspected path has no UIA proof of the current suffix, selection, password state or exact final replacement. A changed-window preflight error can still send the suppressed terminator to the current foreground window. It also releases modifiers synthetically. These behaviors must not replace our refusal and no-repair rules. Mac/Linux guards inspect cancellation, but do not provide the Windows guard's HWND checks. [Windows guard](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/inject.rs#L243), [preflight/error path](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/mod.rs#L525).

## Clipboard and Unicode tradeoffs

The Windows clipboard implementation copies bounded global-memory formats and explicitly excludes handle-backed formats such as bitmap/metafile handles. Incomplete capture does not stop a paste. Restoration empties the clipboard and reinstates captured formats, with no clipboard sequence-number/ownership comparison in the inspected module. **Inference:** a concurrent user copy can be overwritten, and omitted formats can be lost. On macOS/Linux the snapshots distinguish text, empty and other content rather than retaining arbitrary formats. Preserve our automatic-clipboard acceptance gate. [Windows snapshot/restore](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/clipboard.rs#L68), [macOS snapshot](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/mac/clipboard.rs#L72), [X11 snapshot](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/linux/clipboard.rs#L79).

Windows Type emits Return for LF, ignores CR and uses a Unicode tab for body indentation; the forwarded terminator has separate key semantics. macOS follows similar newline handling. X11 can emit an actual Tab key, which can move focus. These choices demonstrate why literal text, line endings and control keys need per-control tests. Neither counting Unicode scalars nor UTF-16 units proves the number of Backspaces required for every grapheme/editor combination. Our initial ASCII-only trigger contract avoids that ambiguity for deletion; replacement Unicode still needs exact observation. [Windows character encoding](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/win/inject.rs#L375), [X11 typing](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/linux/mod.rs#L831).

## Actual platform boundaries

| Platform | Source mechanisms | Limits relevant to us |
| --- | --- | --- |
| Windows | WH_KEYBOARD_LL/WH_MOUSE_LL; foreground-layout translation; marked SendInput or clipboard paste | HWND checks are weaker than suffix/caret proof; no exact output acknowledgement |
| macOS | Quartz session event tap and CF run loop; posted marked Unicode/key events; permission prompting/retry | Requires real Mac permission/Secure Input/composition testing; cancellation guard does not establish target identity |
| Linux/X11 | X RECORD observation, XTEST insertion, keymap translation and temporary unused-keycode mapping; `xclip` clipboard | Layout/composition, global keymap changes, deletion and focus need qualification; X RECORD cannot suppress the terminator, so deletion includes it |
| Linux/Wayland | Explicit refusal when `XDG_SESSION_TYPE=wayland` | No native Wayland expansion implementation; an X11 binary is not universal Linux support |

Sources: [macOS capture](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/mac/mod.rs#L235), [macOS effects/guard](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/mac/mod.rs#L541), [Linux refusal](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/linux/mod.rs#L140), [X11 recording](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/linux/mod.rs#L426), [X11 injector](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/src/input/linux/mod.rs#L700).

The CI configuration builds/checks/tests the app on Windows, macOS and Linux, and tests the platform-free core separately. This is stronger configured coverage than Snipt's inspected Windows story. It is not an observed successful CI run or physical input campaign. The manual checklist describes real-app scenarios, but unchecked boxes and README compatibility claims are not reproducible qualification evidence. [CI configuration](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/.github/workflows/ci.yml), [manual testing](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/docs/TESTING.md).

## Decisions and next experiments

1. Use Ampello as the closest Tauri product reference, Espanso for broader platform architecture, and Snipt as a smaller dependency-level comparison. Keep our stack and native ownership rules.
2. Treat separated deletion and paced typing as **diagnostic variables**, not a product fix. Compare one factor at a time on a new disposable document, retain full preflight guards, record submitted and exact observed output, and stop after uncertainty. Do not replay into a failed fixture.
3. Keep a future multi-batch strategy explicit: it needs cancellation/target generations between effects, cumulative partial-submission reporting, bounded pending work and exact final verification. A delay or an empty hook queue alone cannot establish success.
4. Keep paste behind its own qualified strategy. Clipboard format/sequence preservation and paste acknowledgement are outstanding work; a default used by another product does not waive them.
5. Retain immutable rule snapshots and stop-before-library-change. Ampello reads the body from the current database after recognizing an ID; do not replace our snapshot consistency with that behavior.
6. Add manual coverage for cancellation queue tails, multiline/tab semantics, competing expanders, focus within one field, layout/IME and clipboard races. Continue separate Windows/macOS/X11/Wayland support matrices.

Texpio's current evidence remains: the original combined batch produced exact output in an owned Win32 EDIT receiver, while fresh Notepad fixtures failed even after TextExpander exited. Ampello's source gives a narrower experimental plan; it does not identify our root cause or qualify automatic expansion. See the [acceptance record](offline-mvp-acceptance.md).

Ampello declares **GPL-3.0-or-later** in its manifest/source headers and includes the GPL text. This inspection records independent design lessons and pinned source links; adopting its implementation would require a deliberate licensing/reuse decision. The approved independent-engine choice remains unchanged. [Manifest](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/src-tauri/Cargo.toml), [license](https://github.com/ampello/ampello/blob/c6ba8ea4a227b063df945597d1223f43f6e2f074/LICENSE).
