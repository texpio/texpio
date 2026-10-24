# Texpio preview guide

Texpio is a Windows development MVP. Home opens your local snippet library. You can create and organize snippets, save them on this computer, import/export JSON and copy saved snippets through the quick picker. No account or backend is required; sync is not implemented.

Native insertion and lifecycle compatibility remain unqualified. Physical automatic trials remain **0/300**; see the [MVP acceptance record](../research/offline-mvp-acceptance.md) for current evidence and pending checks. Browser preview cannot use native storage, clipboard or engine services.

## Create and organize snippets

1. On Home, choose **New snippet**. Enter a title, trigger and body, and optionally choose a folder.
2. Use `;` followed by 1–31 lowercase ASCII letters for the trigger, such as `;hello`. Enabled snippets outside Trash must have unique triggers.
3. Enter a literal plain-text body, then choose **Save snippet**. Tabs, line breaks and Unicode text are supported, up to 16,384 Unicode characters. Date/time, clipboard and cursor tokens remain literal text.

Select a snippet to edit it and save explicitly. The preview shows the body as text. Search matches titles, triggers and bodies; the folder filter narrows the results. **New folder** creates a folder. Select a folder and use **Manage folder** to rename or delete it; deleting a folder leaves its snippets unfiled.

**Duplicate** creates an unsaved, disabled copy. Give it a unique trigger before enabling and saving it. **Move to Trash** removes a saved snippet from expansion and the picker. Open **Trash**, select it and choose **Restore snippet** to recover it; an active trigger conflict must be resolved first.

Selecting another snippet, creating a snippet, importing or reloading prompts before discarding an unsaved draft. If another window changes the library, your draft is kept and saving is refused. Copy any edits you need to retain before choosing **Reload library**, which discards the draft after confirmation.

## Copy from the picker

Press **Control+Shift+Period** on Windows to open the quick picker, or use **Open Quick Entry** in the app menu or command palette. Rebind or reset the shortcut in Preferences; a conflicting chord preserves the previous binding and shows an error.

Search by title, trigger or body. The picker shows saved, enabled snippets outside Trash. Choose a result's **Copy** action, or press **Enter** to copy the first result. Copy replaces the clipboard text and closes the picker; paste into your destination yourself. **Escape**, Close or clicking away dismisses the picker and clears its search. Opening or closing it does not insert text into another app. The library editor also has a **Copy text** button for its current body.

## Import, export and local backups

Choose **Export JSON** to save the current saved library, including folders and snippet identities, to a file. Unsaved editor changes are not included. Choose **Import JSON**, select a Texpio JSON export and review the counts before confirming.

Import is additive. Identical records are skipped; differing records with an existing identity or a conflicting active trigger refuse the entire import without changing the library. Import does not overwrite your existing library and is not a destructive backup-restore tool.

Local data is unencrypted in Texpio's app-data directory: `library.sqlite3` and `backups/library-r<revision>.json`. A changed save or import creates a snapshot of the previous saved library; if the backup fails, the change is refused. Exports and backups contain snippet bodies. Keep the live database outside cloud-sync folders. This preview has no UI for replacing a corrupt database from a backup.

## Start, pause and stop expansion

The native engine starts **Stopped**. Save your draft and enable at least one snippet, then choose **Start expansion**. **Pause** suspends observation; **Resume** continues it. **Stop expansion** ends the session. Library saves and confirmed imports stop the engine, including paused sessions; choose Start again when ready. A Developer probe session must be stopped before starting library expansion.

Automatic expansion is restricted to strictly identified Windows Notepad editor controls on US or English (India) keyboard layouts. Type a trigger such as `;hello`, followed by **Space**; the intended replacement is the literal body followed by one retained space. The engine uses direct Unicode input. It refuses unsupported or uncertain contexts, including password fields, selected text, elevated targets, active composition and changed targets. It has no automatic clipboard fallback.

These controls are implemented, but native insertion, picker behavior and the full background lifecycle still need interactive qualification. Other Windows applications, macOS and Linux are separate compatibility work. Developer retains the experimental fixed-fixture probe and its separate acceptance campaign.

## Preferences and background behavior

Open the command palette from the app menu or its displayed shortcut to find actions, toggle/resize sidebars or switch Home/Developer. Layout saves locally. Preferences offers System/Light/Dark and English/French/Arabic. System follows OS appearance; Arabic changes direction. Preferences apply to the quick pane. Failed loading offers Retry without discarding saved settings.

Closing the main window is designed to hide it. Tray **Open Texpio** restores it; **Quit** exits. Launching again restores the existing instance. Tray controls can pause/stop observation while hidden. Restart/install requires confirmed engine shutdown. These lifecycle paths remain part of the pending native qualification.

Developer can request a test notification; OS settings affect delivery. A rendering failure offers redacted diagnostic saving and safe restart, reporting actual save success/failure. **Check for Updates** is manual. This preview has no configured feed/key and reports setup unavailable.

Adapted from [tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md).
