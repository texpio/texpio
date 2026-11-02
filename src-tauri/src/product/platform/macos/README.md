# macOS product adapter

This module independently implements Quartz capture, AX context and AppKit paste.
It does not incorporate Ampello or Espanso runtime source. Its framework ABI is
in `ffi.rs`, linked only on macOS; no new Cargo dependency is required.

The active session tap suppresses a completing event only after a bounded job is
accepted. The callback uses a recent native context snapshot and performs no AX
IPC, permission request, clipboard access or injection. Separate owned threads
refresh context and serialize effects. Generated events carry an owned marker.
Generations, focus/source changes, navigation, permissions and Secure Input guard
recognition and each effect. Jobs expire after 30 seconds; a context polling gap
over 250 ms invalidates history after suspension or delayed scheduling.

Candidate modes are Type, and Paste with restoration explicitly disabled.
Paste with restoreClipboard=true is unavailable: runtime startup rejects it with
`invalidSettings` before creating a tap, recognition refuses it before job
admission/suppression, and per-job preparation and the clipboard adapter enforce
the same refusal. The settings store also clears restoration whenever insertion is
Paste on macOS, so a stored pair from another platform or an earlier build cannot
leave every start failing. Public NSPasteboard APIs provide no atomic
compare-and-clear operation. A changeCount
and marker check cannot guarantee restoration will preserve a copy made between
that check and clearContents. The unsafe restoration/snapshot/fallback-writing
paths were removed, rather than claiming that a narrower race is safe.

Non-restoring Paste deliberately replaces the clipboard and uses Cmd+V. A UUID
format and changeCount identify the current lease; detected ownership changes
cancel following effects. This is not an atomic clipboard-replacement guarantee.
The 150 ms cancellable settling interval is not paste acknowledgement. Neither
submitted events nor a completed effect job prove document commitment. Partial
effects are never retried.

Initial recognition requires ASCII triggers, an active input source that is a
keyboard layout (`TISTypeKeyboardLayout`) whose `kTISPropertyInputSourceIsASCIICapable`
flag is true, positively settable AXValue, collapsed numeric selection metadata,
an editable `AXTextField`/`AXTextArea` role, a subrole that is not
`AXSecureTextField`, and an explicitly returned empty `AXMarkedText` string.
Input methods (Pinyin, Kotoeri, transliteration) are refused: they compose marked
text that a keydown cannot describe, so per-key deletion is unsafe. A layout only
has to be ASCII-capable, so regional English layouts such as
`com.apple.keylayout.ABC-India` are accepted instead of a US/ABC-only allowlist.
Missing attributes, failed queries, secure subroles, unknown value types and
nonempty marked text refuse.

Characters are derived with `UCKeyTranslate` using the layout's
`kTISPropertyUnicodeKeyLayoutData` and persistent dead-key state. A keydown that
starts a dead-key composition, or the keystroke that completes one, delivers fewer
characters than keys, so both reset history and refuse rather than desynchronizing
the deletion count. Many normal TextField/TextArea providers still omit the
positive context attributes; that remains a functional gap, and the runtime does
not infer this evidence from missing attributes.

Option/Command/Control combinations, unknown translations, non-ASCII trigger
events, Return/Tab activation, rich bodies and attachments refuse. Unicode Type
events retain whole UTF-16 scalar encodings. The payload's multiline eligibility
and configured mode are checked before queue admission, so deterministic refusals
leave the original completing event delivered. Preparation repeats these checks
before any erasure. Writable AX TextArea newline semantics still need native
qualification before extending the currently recognized subrole profile.

Shutdown acknowledges only after the tap/source and context/effect threads exit.
It returns false after two seconds if they cannot finish; process-wide ownership
prevents a replacement tap until they exit. Each AX query has a checked 25 ms
messaging timeout. AppKit calls are synchronous and do not expose per-call
provider deadlines; a hung native service remains a qualification/teardown gap.

Host verification (macOS 13.7.8, x86_64, 1 November 2026): policy tests, rustfmt,
strict Clippy and the Rust suite pass on the macOS host; `bun run tauri build
--bundles app -- --locked --no-default-features` produces and launches
`Texpio.app`. The active input source on this host is
`com.apple.keylayout.ABC-India` (keyboard layout, ASCII-capable), which the
previous US/ABC allowlist refused with `unavailable`; the current gate accepts it,
and the app now reports `permissionRequired` instead. The in-app permission prompt
and Settings card were observed with the missing grant listed, and the deep link
opened System Settings directly on the Accessibility pane.

Permission model: Accessibility is the only grant readiness requires. On this host
the Input Monitoring pane listed **no items at all**, while a process trusted for
Accessibility reported `CGPreflightListenEventAccess()` and
`CGPreflightPostEventAccess()` as true, created both a HID and a session event tap,
and received injected keydowns. The same probe built into a fresh, untrusted bundle
reported both preflights false and had both tap creations denied. macOS therefore
uses Accessibility trust for input control and keeps Input Monitoring for processes
it does not trust; the reference app Fast Text asks for Accessibility only and uses
an `NSEvent` global monitor. `listen` and `post` are still reported, and the Input
Monitoring row appears only when macOS withholds listening from an
Accessibility-trusted process. The tap-creation failure path still returns
`inputBlocked` if a future macOS release separates the grants.

Still unverified natively: actual recognition and insertion with Accessibility
enabled, `UCKeyTranslate` output and dead-key refusal against real typing, a usable
positive AX context provider in ordinary editors, software-event provenance,
Unicode/newline delivery, non-restoring clipboard races, shutdown timing, and
ARM64. Ordinary tests install no tap. No end-to-end macOS expansion result is
claimed.

Primary ABI/behavior references inspected:

- [Apple CGEventTapCreate](https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate(tap:place:options:eventsofinterest:callback:userinfo:))
- [Apple AXUIElementSetMessagingTimeout](https://developer.apple.com/documentation/applicationservices/1459345-axuielementsetmessagingtimeout)
- [Apple AXValueType.cfRange](https://developer.apple.com/documentation/applicationservices/axvaluetype/cfrange)
- [Apple AXValue attribute](https://developer.apple.com/documentation/applicationservices/kaxvalueattribute)
- [Apple NSPasteboard.changeCount](https://developer.apple.com/documentation/appkit/nspasteboard/changecount)
- Cached objc2-core-graphics / objc2-core-foundation 0.3.2 generated declarations
  for Quartz fields, Unicode counts, permission calls and Core Foundation.

The ignored host fixture is `.check/harness.rs`. Reproduce source metadata and
policy checks with:

`pwsh -NoProfile -File src-tauri/src/product/platform/macos/.check/check.ps1`

It refreshes native source copies before checking metadata and running policies.
