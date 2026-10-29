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
Paste with restoreClipboard=true is unavailable: runtime startup rejects it before
creating a tap, recognition refuses it before job admission/suppression, and
per-job preparation and the clipboard adapter enforce the same refusal. Public
NSPasteboard APIs provide no atomic compare-and-clear operation. A changeCount
and marker check cannot guarantee restoration will preserve a copy made between
that check and clearContents. The unsafe restoration/snapshot/fallback-writing
paths were removed, rather than claiming that a narrower race is safe.

Non-restoring Paste deliberately replaces the clipboard and uses Cmd+V. A UUID
format and changeCount identify the current lease; detected ownership changes
cancel following effects. This is not an atomic clipboard-replacement guarantee.
The 150 ms cancellable settling interval is not paste acknowledgement. Neither
submitted events nor a completed effect job prove document commitment. Partial
effects are never retried.

Initial recognition requires ASCII triggers, US/ABC input sources, positively
settable AXValue, collapsed numeric selection metadata, an explicitly recognized
nonprotected AXSubrole (currently AXSearchField), and an explicitly returned empty
AXMarkedText string. Missing attributes, failed queries, unknown/secure subroles,
unknown value types and nonempty marked text refuse. US/ABC alone does not prove
composition is idle. Many normal TextField/TextArea providers omit this context;
that is a functional gap until a qualified native provider or bounded bridge
supplies positive protected/composition evidence. No general editor support is
claimed. The runtime does not infer this evidence from missing attributes.

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

Windows-host verification: nine pure policy tests observed red then green,
rustfmt, and Rust source metadata checking against a small shared-contract harness
with framework linking removed. This checks source shapes only. macOS framework
linking, permission flows, actual input delivery, a usable positive AX context
provider, software-event provenance, Unicode/newlines, non-restoring clipboard
races and shutdown require native testing on x64 and ARM64. Ordinary tests install
no tap. No macOS native build or interaction result is claimed.

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
