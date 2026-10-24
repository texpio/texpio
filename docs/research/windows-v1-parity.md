# Windows v1 prototype parity tracker

Reference: [design](../superpowers/specs/2026-10-24-windows-v1-design.md), Ampello source `c6ba8ea4a227b063df945597d1223f43f6e2f074`. User reports installed Ampello works on this PC. Texpio keeps its own UI and independent implementation.

| Feature | State | Evidence / remaining work |
| --- | --- | --- |
| Windows practical Paste/Type expansion | Partial | Separate product engine integrated. Fresh Notepad Paste fixture: 18/18 events, exact output observed; Type: 52/52 but mismatched. Physical capture acceptance pending |
| Word/anywhere boundaries and terminators | Implemented core | Longest match, colon/semicolon/Unicode, punctuation/Tab/Enter and editing tests pass; product hook awaits physical trials |
| Pace/cancel/target changes | Partial | Effect failure/cancellation and generation tests pass; real-app Start/Pause/Resume/Stop passed. Type Notepad remains unqualified |
| Clipboard shortcut and restoration | Partial | Sequence-aware native restoration implemented; explicit Copy and synthetic Paste pass. Shortcut and clipboard-race acceptance pending |
| Attachments/order/delay | Implemented, acceptance pending | Native content-addressed store, selected/drop/pasted-file intake, ordered CF_HDROP delivery and editor controls; storage/payload tests pass. Native target delivery still needs acceptance |
| Create/edit/search/folders/disable/trash | Implemented | Existing offline MVP tests and actual-app evidence |
| Favorites/usage/dashboard | Implemented, acceptance pending | Favorite filter and editor toggle, active/recent/most-used summaries; metadata updates preserve drafts and favorite writes are revision-fenced |
| Editor keyboard navigation/find/replace | Implemented, acceptance pending | Registered Ctrl+N/S/F/H, existing Ctrl+K palette, list navigation/Delete/Escape, literal Unicode find/replace and undo button; focused UI tests pass |
| JSON backup/import | Implemented | Transaction/round-trip tests; selected-file native checks pending |
| YAML/archive/import replace/recovery | Implemented, acceptance pending | JSON/YAML text+metadata, ZIP attachment bytes, one-shot preview tokens and explicit merge/replace. Corrupt DB/WAL/SHM preservation tests pass; actual-app dialogs still need acceptance |
| Shared/personal switch/external refresh | Implemented, acceptance pending | Local-drive selection, atomic persisted routing, stale-save/attachment fencing and SQLite data-version watch tested; actual-app switch check pending |
| Tray/close/single instance | Partial | Hide/restore passed; configurable behavior/toggle pending |
| Autostart hidden/expansion hotkey | Planned | Existing quick-picker shortcut retained |
| Product preferences | Partial | Insertion/boundary/terminator/restore/speed/cancel/delay controls implemented in existing dialog; persistence/failure tests pass. Hotkey/startup/data settings pending |
| Explicit Copy picker | Implemented | Actual clipboard/focus/Escape checks passed |
| Manual update/diagnostics | Partial | Existing configured updater and redacted diagnostics retained |
| Windows packaging | Implemented baseline | Executable/MSI/NSIS build at 84fefd6; rebuild after implementation |

This tracker records v1 delivery, independently of strict Developer/P0 physical qualification and macOS/Linux research.

Current checkpoint verification: `bun run check:all` passed **148 frontend tests, 149 Rust tests**, plus tooling/static checks after task 7. Matching/effects/settings and migration tests demonstrated failing-then-passing behavior. Native app lifecycle acceptance used an isolated identifier and no typed input; all owned connections/processes ended afterward. No claim that all rows are complete.
