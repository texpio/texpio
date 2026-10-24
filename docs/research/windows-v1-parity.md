# Windows v1 prototype parity tracker

Reference: [design](../superpowers/specs/2026-10-24-windows-v1-design.md), Ampello source `c6ba8ea4a227b063df945597d1223f43f6e2f074`. User reports installed Ampello works on this PC. Texpio keeps its own UI and independent implementation.

| Feature | State | Evidence / remaining work |
| --- | --- | --- |
| Windows practical Paste/Type expansion | Planned | Old strict Notepad pipeline fails; add separate product path |
| Word/anywhere boundaries and terminators | Planned | Existing strict recognizer supports only semicolon/lowercase + Space |
| Pace/cancel/target changes | Planned | Strict probe cancellation generations exist; product path pending |
| Clipboard shortcut and restoration | Planned | Explicit Copy passed; automatic delivery not implemented |
| Attachments/order/delay | Planned | Native store, clipboard file delivery and UI pending |
| Create/edit/search/folders/disable/trash | Implemented | Existing offline MVP tests and actual-app evidence |
| Favorites/usage/dashboard | Planned | Schema/UI pending |
| Editor keyboard navigation/find/replace | Partial | Existing accessible editor and Save; richer shortcuts pending |
| JSON backup/import | Implemented | Transaction/round-trip tests; selected-file native checks pending |
| YAML/archive/import replace/recovery | Planned | Preserve existing JSON compatibility |
| Shared/personal switch/external refresh | Planned | No backend/sync |
| Tray/close/single instance | Partial | Hide/restore passed; configurable behavior/toggle pending |
| Autostart hidden/expansion hotkey | Planned | Existing quick-picker shortcut retained |
| Product preferences | Planned | Existing theme/language/shortcut preferences retained |
| Explicit Copy picker | Implemented | Actual clipboard/focus/Escape checks passed |
| Manual update/diagnostics | Partial | Existing configured updater and redacted diagnostics retained |
| Windows packaging | Implemented baseline | Executable/MSI/NSIS build at 84fefd6; rebuild after implementation |

This tracker records v1 delivery, independently of strict Developer/P0 physical qualification and macOS/Linux research.
