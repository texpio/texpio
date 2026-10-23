# Menus and context menus

`src/lib/menu.ts` creates translated native menus: an application menu on macOS, main-window menu on Windows/Linux. Native Edit roles retain OS undo/redo/clipboard/select semantics. Implemented app actions dispatch shared command IDs; no fake document actions exist.

Generation-unique IDs prevent stale callback routing. Cancelled delayed construction releases resources; language updates mutate text rather than creating duplicate ownership.

`src/lib/context-menu.ts` retains popup resources until disposal: popup resolution means shown, not dismissed. Availability/labels refresh when shown, and hooks release listeners/resources on teardown. Only targeted editable context menus are intercepted.

Register commands before adding items. Mocked dispatch tests do not establish native menu layout or platform role acceptance.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
