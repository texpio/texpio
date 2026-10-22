# Texpio feature plan

Planning proposal · 23 October 2026 · Review pending

## Brief, audience, and success

**Confirmed:** the name is Texpio; the product is an open-source, local-first desktop text expander inspired by TextExpander, with an optional small-charge managed sync service later. Release order is **Windows first, then macOS**. The user selected the official Tauri starter with React/TypeScript and shadcn/ui; see the [technology stack](tech-stack.md). The native expansion engine remains subject to feasibility. Feature scope below remains proposed; runtime results and customer validation are future work.

**Proposed product promise:** create reusable text visually, expand it reliably in supported applications, and retain ownership of an unlimited local library without an account. Paid services must remain optional; subscription expiry must leave local creation, editing, expansion, and export working.

**Audience assumption:** individual knowledge workers, developers, and support professionals repeating addresses, code fragments, and replies. The initial reason to choose Texpio is an approachable visual editor plus dependable offline expansion and transparent compatibility. This positioning needs interviews and usage evidence. Teams, regulated workflows, and password management are outside the initial audience.

Success means a new user can create and test a snippet quickly, predict where expansion works, pause it immediately, recover their library, and understand what input access permits. “Works everywhere” is not a promise. Linux is a later feasibility investigation, including compositor-specific constraints.

## Product and engine approaches

| Approach | Benefit | Tradeoff and decision |
| --- | --- | --- |
| **Tauri editor with a separate Rust expansion engine — recommended** | A visual interface and independently testable matching/insertion components; future native adapters can share library semantics. | Texpio owns difficult input behavior, packaging, and maintenance. P0 must establish feasibility before committing to this architecture. |
| GUI around an existing engine, evaluating Espanso reuse or process integration | May reuse established expansion behavior and reduce some native-engine work. | Do not assume a supported embeddable API. Validate integration, distribution, update coupling, and license compatibility. Espanso identifies GPL-3 licensing and file-based configuration on its [official site](https://espanso.org/). |
| Picker-first text utility | Explicit insertion reduces dependence on continuous abbreviation recognition and can validate the editing experience. | Does not deliver the central automatic-expansion promise; retain as an experiment or compatibility fallback, not an equivalent MVP. |

Tauri's [global-shortcut plugin](https://v2.tauri.app/plugin/global-shortcut/) documents registered key combinations. **Planning inference:** it is useful for opening the picker, but does not establish ordinary typed-character recognition or replacement. Separate the UI, library/template rules, and native observation/insertion adapter; separation need not mean multiple processes. The frontend receives narrow commands and status, not the raw input stream. Apply least-privilege [Tauri capabilities](https://v2.tauri.app/security/capabilities/) and keep observation independent of editor work.

## Priorities by phase

M = required for that phase's gate; C = candidate requiring evidence. Subsequent phases retain earlier capabilities. These are scope proposals, not delivery dates.

| Capability | P0 feasibility | P1 Windows offline MVP | P2 workflows/macOS readiness | P3 personal sync | P4 teams |
| --- | --- | --- | --- | --- | --- |
| Native observation, insertion, focus safety | M: compare strategies | M: supported-app contract | M: broader coverage | Maintain | Maintain |
| Plain-text triggers and Unicode bodies | M: prove core cases | M: bounded literal matching | C: aliases and case rules | Retain | Retain |
| Visual create/edit/duplicate/delete | Test interface only | M | M: bulk operations | Retain | Retain |
| Organization and search | — | M: folders, title/trigger search | C: tags, favorites, body search | Retain | Shared libraries |
| Keyboard picker | Probe focus restoration | M: search, preview, insert/copy | M: workflow polish | Retain | Retain |
| Tray pause, app exclusions, health status | M: prove controls | M | M: per-app tuning | Retain | Retain |
| Local durability, backup, native JSON export | Probe failure modes | M | M: migration tools | Retain | Retain |
| Date/time, clipboard, cursor tokens | Probe insertion implications | M: minimal safe set | M: simple fill-ins | Retain | Retain |
| Rich text, images, nested templates | — | Excluded | C: separate compatibility gate | — | — |
| macOS | Document adapter risks | Excluded | M: feasibility and permission work; beta only after its gate | Supported devices only | Supported devices only |
| Encrypted managed sync and history | Boundary only | Excluded | Demand/recovery research | M, conditional on demand | Retain |
| Shared libraries, roles, administration | — | Excluded | Excluded | Excluded | C, after demand |

P0 answers engineering questions; it is not a consumer release. P2 does not automatically promise macOS general availability. Arbitrary scripts, shell execution, network tokens, an extension marketplace, AI generation, mobile expansion, and browser extensions are excluded from the MVP. No automatic launch-as-administrator workaround is proposed.

## Important user flows

1. **First snippet and trigger:** explain local input access, offer a test field, and create `;addr`. P1 proposes literal, case-sensitive triggers with a recommended prefix. **Space activates expansion; preserve that trailing space by default, with an explicit remove-space setting.** Reject duplicate active triggers and warn on overlaps. P2 may evaluate Tab/Enter only after preventing unintended navigation/submission. Matching failures or uncertainty leave the trigger and space intact; no deletion occurs. Never recursively expand generated text.
2. **Picker:** a configurable shortcut opens title/trigger search with a readable preview. Enter attempts insertion into the captured application only after checking focus; Escape cancels. An invalid target causes a non-destructive refusal and offers explicit Copy. Never paste into whichever unrelated window happens to gain focus.
3. **Editing:** save title, trigger, plain-text body, folder, and enabled state atomically. Validate tokens and collisions before save; preview uses sample values and does not consume live clipboard content without an explicit action. Duplicates receive a new identity and require a distinct trigger. Deleted items can be restored from local trash or backup.
4. **Exclusion and pause:** pause immediately from tray or shortcut; clearly show paused/active/error state. Exclude the current application using its identity, with a reviewable list and guidance for executables that change location. Clear recognition state on pause, resume, application switch, navigation/selection, layout change, lock, and uncertain composition state.
5. **Import/export and recovery:** P1 supports a documented, versioned Texpio JSON interchange format. Validate size, schema, tokens, and conflicts, show a dry-run summary, back up before importing, and commit all-or-nothing. Never execute imported content. P2 evaluates specific TextExpander/Espanso formats using fixtures and a conversion-loss report; compatibility is not assumed. Export is explicit and user-readable; restoring a verified backup must be possible without a service.

## Safe expansion and templating

Start with plain text and a small declarative token grammar: date/time using documented formats and timezone rules, explicit clipboard text, and one cursor position. No `eval`, shell, file reads, or network access. Unknown tokens remain visible validation errors; escape syntax allows literal token text. P2 simple fill-ins require preview, cancellation, and restoration of the original target. General scripting needs a separate permission and threat-model decision later.

P0 compares Unicode insertion with clipboard-assisted paste by application. Clipboard use requires preserving supported formats, detecting ownership changes, and restoring only when Texpio still owns the clipboard; a concurrent user copy must win. If safe preservation cannot be established, use a validated alternative or decline that insertion mode. Clipboard managers/history may retain temporary values, so clipboard-based insertion needs a clear explanation. Do not present snippet text as a safe secret store.

Offer immediate expansion undo only while the same target/caret context remains verifiably valid; invalidate it after other edits or navigation. Test both Texpio's reversal and native application undo. Never promise one undo gesture across all editors, and never issue blind backspaces to “recover” after lost focus or a partial insertion. Report uncertain outcomes and stop further automatic expansion until the context resets.

Skip recognized password/secure fields, exclude known sensitive applications, and explain manual pause. Password-field detection is **best effort**: custom controls, browser editors, and inaccessible surfaces can hide field semantics. If the chosen adapter cannot establish a safe supported context, automatic expansion should decline it; there is no universal password-field guarantee.

## Storage, privacy, and future sync boundary

**Proposal:** an application-owned SQLite library with transactions, explicit schema migrations, and recoverable local snapshots; JSON is the interchange format. Each snippet/folder has a stable random UUID, schema version, modification metadata, and local revision. Separate user content from device-specific shortcuts, exclusions, insertion preferences, and transient matching state. Do not sync a live database by placing it in a shared cloud folder.

Preserve IDs for native backup restoration; offer duplicate-as-new behavior for conflicting imports. P1 needs migration/rollback and restore tests, not an early distributed-sync implementation. P3 can add per-record revisions, device identity, a durable change queue, and tombstones under an explicit protocol. Concurrent edits should preserve both versions for review; specify deletion retention, restoration, offline device return, and history limits before implementation.

P3 proposes authenticated device enrollment and end-to-end encrypted content with explicit key recovery. Account reset must not imply recovery of lost content keys. Decide what metadata the service sees and which settings travel between devices. Local storage and exports are not automatically encrypted by remote encryption; document their protection separately. P1 remains usable in airplane mode, with no account or telemetry required. Collect no typed-text history; keep only a bounded in-memory matching suffix, clear it on context reset, and omit snippet/clipboard content from diagnostic logs. Diagnostics are locally reviewable and explicitly exported.

## Compatibility and proposed quality gates

Windows [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) is restricted to equal/lower-integrity targets, and pressed modifiers can interfere. Normally running Texpio must describe elevated applications as unsupported until separately validated. Microsoft also documents message-loop and timeout requirements for [low-level keyboard hooks](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc); a hook may be silently removed. P0 must evaluate recovery and honest health reporting without claiming guaranteed removal detection.

Observed key events are not equivalent to committed text. [Windows IME composition](https://learn.microsoft.com/en-us/windows/win32/intl/wm-ime-composition) requires separate testing from Unicode output. Publish exact tested OS/app versions, input controls, layouts, IMEs, and insertion modes. Candidate targets include Notepad, VS Code, browser plain text/contenteditable fields, Outlook, Word, and terminals; none is currently certified. Test dead keys, AltGr, emoji, combining marks, Hindi input, CJK composition, selections, remote sessions, and sleep/resume. Composition-based triggers remain unsupported until safe; Unicode snippet bodies alone do not prove IME recognition.

macOS work must test packaged-app grants/revocation and secure input; Apple documents user-controlled [Input Monitoring](https://support.apple.com/guide/mac-help/control-access-to-input-monitoring-on-mac-mchl4cedafb6/mac). The selected adapter may require additional permissions. Linux remains separate: the first-party [global-hotkey README](https://github.com/tauri-apps/global-hotkey/blob/dev/README.md) identifies Linux support as X11-only, which does not establish Wayland expansion feasibility.

All numbers below are **proposed targets, not measured benchmarks**. Freeze reference hardware, build, fixtures, and support matrix; publish per-case results. Eligibility requires a certified supported context. Unknown contexts are refused before deletion and counted separately. Refusals within eligible contexts count as unsuccessful completions, separately from incorrect output.

| Area | Acceptance definition |
| --- | --- |
| Expansion correctness | At least 10,000 eligible attempts per supported insertion strategy across the declared matrix; ≥99.9% exact expected text. Zero wrong-target insertions or unrelated-text deletions across positive, unsupported-context, modifier, and focus-race cases. |
| Responsiveness | With 10,000 snippets, p95 ≤50 ms from committed trigger recognition to insertion dispatch and ≤150 ms to observed completion for a 2,000-character plain-text fixture; picker search p95 ≤100 ms. Record each app separately. |
| Resource budget | Propose ≤1% average idle CPU over ten settled minutes and ≤150 MiB combined resident memory with editor hidden on a declared 4-core/8-GB reference machine; revise transparently after P0 evidence. |
| Durability | Zero loss of acknowledged saves under 100 forced-termination/restart cycles; import/export preserves supported fields and identities; restore succeeds after a deliberately corrupted working copy. |
| Accessibility | Every primary flow keyboard-operable; verify focus order, accessible names/status, Windows screen-reader use, high contrast, and 200% scaling. Status must not rely on color. macOS repeats with its accessibility tools. |

## Dependencies, release decisions, and gates

**P0 → P1:** demonstrate observation/insertion and safe failure on Windows, document unsupported contexts, choose the engine approach, and review dependency licenses. Failure means revise scope or architecture, not market an unproven system-wide engine.

**P1 → P2:** pass the declared offline correctness, durability, privacy, accessibility, and packaging gates; verify install/update/uninstall and backup recovery on supported Windows versions. Publish known limitations and obtain real beta feedback.

**P2 → P3:** preserve Windows quality while validating macOS independently; confirm recurring multi-device demand and willingness to pay. Sync requires reviewed conflict/key/recovery design, security review, service-cost evidence, and cross-device fault testing before release.

**P3 → P4:** demonstrate reliable personal sync and paid demand for sharing before committing to roles, ownership, offboarding, or administration.

Open decisions: supported Windows/macOS versions and architectures; license and contribution policy; exact undo contract; initial app/IME support matrix; backup retention and optional local encryption; migration formats; sync pricing, recovery model, and hosting jurisdiction. Resolve these through evidence and product review. This is a review proposal, not a detailed implementation plan.
