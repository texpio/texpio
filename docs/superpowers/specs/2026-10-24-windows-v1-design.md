# Windows v1 prototype — Ampello feature parity, Texpio UI

24 October 2026 · Supersedes the limited offline-MVP product scope

## User direction and execution

The user installed Ampello and reports that it works on Windows. They explicitly requested every feature supported by that application as Texpio's first version, preserving Texpio's UI/UX and focusing on a working Windows prototype. macOS/Linux implementation and universal compatibility are not v1 work. Existing autonomous Native/current-checkout execution and commit/push authorization continue; do not repeat approval conversations already waived by the user.

Reference: Ampello 0.4.0 at `c6ba8ea4a227b063df945597d1223f43f6e2f074`, inspected in the [source audit](../../research/ampello-source-audit.md). Learn mechanisms and behavior; retain the independent-engine choice and existing tool versions. Do not copy GPL implementation into the project or select a product license incidentally.

## Approach

Keep Tauri/React/TypeScript/shadcn/Bun, existing layout/themes/locales, Rust SQLite and generated bindings. Add a practical Windows product engine separate from the strict Developer/P0 experiment. It recognizes typed history, verifies foreground/focus/layout identity around effects, and supports explicitly selected Paste (default) or paced Type. This supersedes the old product restriction to Notepad/UIA-proven suffixes and single-batch Unicode. The stricter probe and its evidence remain intact; product submission must not be reported as exact observed output without observation.

Alternatives considered: continue fixing only the narrow Notepad profile (does not deliver requested app features); replace/fork the whole Ampello application (changes UI/architecture and licensing choice); independent Windows engine plus incremental feature parity (selected implementation decision, preserves prior work). No invisible retry through a second insertion strategy after partial submission.

## Behavioral parity checklist

| Area | V1 behavior | Existing Texpio contribution |
| --- | --- | --- |
| Expansion | User triggers, word-start/anywhere matching, longest suffix, Space/punctuation/Tab/Enter terminators, optional terminator preservation, Backspace correction, reset on navigation/click/focus change | Retain strict probe separately; add product recognizer |
| Insertion | Paste/Type choice, Unicode and whitespace, large/empty bodies; fast/balanced/careful typing | Explicit Copy remains separate |
| Cancellation | Escape/Pause/ScrollLock/F1–F12 setting; bounded pending events; stop between effects and on target change | Keep cancellation generations and no replay |
| Clipboard | Restore option; temporary text/file payloads; independent clipboard Type/Paste shortcut with enabled/rebinding/conflict states | Native clipboard plugin already installed |
| Files | Ordered attachments, add/remove/reorder, text-first/files-first, grouped/strict ordering and configurable delay; missing-file state | Add native-owned content-addressed store and selected-file access |
| Library | Create/edit/delete, enable, favorites, collections, search, keyboard navigation, literal editor with find/replace and shortcuts | Keep titles, trash/recovery, explicit save, current editor layout |
| Dashboard | Total/enabled snippets, usage counts, recent/most-used entries | Adapt within current shell; no redesign |
| Data | Personal/default and optional selected shared local directory, switch without restart, external edit refresh, database diagnostics | Preserve revision conflicts, transactions and backups; do not move a live database |
| Interchange | YAML/JSON text backups and attachment archive; preview import, additive/replace choice with confirmation; legacy text backups | Retain current JSON IDs and schema migration compatibility |
| Recovery | Preserve unusable DB/WAL/SHM before explicit recovery; visible failure/recovery status | Never interpret permission/lock/migration errors as proof of corruption |
| Desktop | Tray Open/enable/disable/Quit, close-to-tray option, single instance, launch-at-login hidden, configurable expansion toggle shortcut | Retain draft Quit acknowledgement and quick picker shortcut |
| Preferences | Theme, insertion/boundaries/terminator/restore/speed/cancel/delay, shortcuts, data location, app/version/update/diagnostics | Rust-owned, all visible strings en/fr/ar |
| Privacy | Offline local library, no account/sync/telemetry; update check only by explicit action | Retain existing configured-update safeguards |
| Packaging | Windows executable/MSI/NSIS, tested prototype workflows | Signing/distribution is separate from local prototype success |

No feature is complete just because this table names it. The implementation plan and acceptance tracker record actual status and evidence. Existing accessibility, theme, error/empty/loading states and listener cleanup remain requirements.

## Native product engine

Use a small hook callback, bounded matching history and bounded job queue. Own hook thread/message loop and effect worker; Stop disables publication immediately, requests cancellation, and joins with bounded acknowledgement. Never run global hooks in ordinary tests. Only one product/probe observer may be armed. Reject unrelated injected events from matching, ignore our marked effects, and clear history on input/focus/layout discontinuities. Background rendering/database/file work stays outside the callback.

Prepare the replacement and attachment paths from an immutable library snapshot before deleting. Capture target identity at recognition. Do not write to Texpio's own windows. Preserve physically held modifiers rather than forcibly releasing them. If modifiers do not clear within a bounded wait, refuse. On target change/cancel/partial delivery, stop without compensation and report partial/cancelled status. Password/Secure Desktop/elevated contexts receive no compatibility claim; use positive native checks where available without making UIA text support a prerequisite for normal editors.

Paste snapshots supported clipboard formats with byte bounds, writes the chosen payload, sends Ctrl+V, and restores only while ownership/sequence still belongs to this operation. A newer user copy wins. Unsupported formats are disclosed when restoration is incomplete. Type sends scalar-aligned UTF-16 pairs with paced checks; newline and tab semantics follow the selected Windows strategy and need real-app evidence. Queue metering is not target acknowledgement. Final status distinguishes submission/cancellation/failure from the Developer probe's observed result.

Library saves rebuild product snapshots while clearing old matching history; do not require repeated manual Start after each save. Start/enable is explicit initially; persist preference only through user settings. Launch-at-login is opt-in. Recovery and replacing/importing a library stop active effects before mutation.

## Data and migration

Migrate existing SQLite records in place without losing IDs, bodies, titles, folders or trash. Add favorites, usage/timestamps, attachments and product settings with defaults. Retain old JSON import compatibility. Content bytes stay native-owned and outside UI layout state. File pickers grant only requested paths; IPC attachment identifiers never authorize arbitrary filesystem reads. Hashes address stored content; filenames are validated leaf names. Archives reject traversal, duplicate/colliding entries and decompression/size/count abuse; imports validate before committing.

Shared libraries are explicitly chosen local directories, not a sync backend. Switch the store and engine snapshot together after validating the destination. Preserve the personal library and rollback the selection on failure. Detect external commits using SQLite data version/revision and invalidate TanStack Query without discarding editor drafts.

## Acceptance and priorities

First prove a functioning Windows product expansion path on a disposable document, then complete storage/settings/files and UI parity, then full app acceptance/review/builds. Test core matching/effect cancellation and storage migration with meaningful RED/GREEN regressions. Run full checks and Windows builds at integration. Record actual normal typing separately from synthetic fixture submissions. The old 0/300 campaign is not a prerequisite for local prototype iteration and is not silently counted as completed.

Acceptance includes short/long/multiline/Unicode/empty snippets, boundaries/terminators, mode changes, cancel/focus changes, clipboard preservation/concurrent copy, attachments/order, shortcuts, shared/personal switch, import/export/recovery, persistence, and tray/quit. Test only owned synthetic content. Keep desktop automation short and disconnect it before code/build work; do not change the user's installed Ampello settings or data without a task-specific reason.
