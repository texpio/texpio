# Library workspace and Preferences page

## Intent

The user requested removing the global right sidebar/toggle and Developer page,
making Preferences a dedicated page, and improving snippet creation/management.
They selected a combined snippet list and editor instead of a separate editor
page. Existing autonomous current-checkout execution continues.

## Layout and workflow

- One left navigation contains Snippets, Favorites, Trash, Unfiled and folders,
  with New folder and Preferences. The native title bar retains its left toggle.
- The workspace has a compact heading/toolbar, an expansion status strip, a
  searchable snippet list and a larger editor. Pane widths are bounded and
  independently scrollable; at narrow widths navigation can collapse.
- New snippet opens an empty editor, focuses Title, inherits the selected folder
  and makes Save prominent. Existing snippets show their title, abbreviation,
  folder, enabled state, favorite and copy/duplicate/trash actions. A roomy body
  editor, preview and attachments are distinct editing sections.
- Explicit Save, duplicate validation, stale revisions, attachment semantics,
  keyboard shortcuts and unsaved-draft confirmation remain. Changing collection
  filters does not discard a draft. Visiting Preferences keeps the editor mounted
  so returning preserves the unsaved draft; hidden library shortcuts are disabled.
- The idle editor gives create/select guidance and a compact library overview.
  Statistics no longer push the snippet list below the fold.

## Preferences

Preferences is a main view with General, Expansion, Shortcuts and Data sections.
General groups appearance/language and launch/close behavior. Expansion groups
activation, insertion, matching, cancellation and compatibility. Shortcuts groups
the quick picker, expansion toggle and clipboard insertion. Data contains library
location and backup/recovery entry points. Preference controls preserve Rust-owned
immediate persistence and visible failures. Destructive/import confirmations may
still use dialogs; the Preferences screen itself does not.

## Implementation boundaries

The existing Radix/shadcn neutral theme, pinned dependencies, Tauri commands and
Rust engine remain. UI store owns navigation, section and filter IDs and only
persists sanitized layout numbers/visibility. Drafts, snippets and clipboard text
are not placed in shared persisted state. Remove obsolete right-layout keys,
right-sidebar commands/titlebar button, Developer navigation/menu commands and
unneeded last-quick-entry retention. Preserve the strict probe source/qualification
infrastructure outside the product navigation.

## Approaches and decisions

The user chose list/editor together over separate editor navigation. The agent
places folders in the existing left navigation to avoid a fourth column, keeps
explicit Save rather than introducing autosave semantics, and reuses installed
ToggleGroup primitives for section choices instead of adding dependencies.
These implementation choices support the requested workflow and can be revised
from the actual preview.

## Verification

Test removal from shell/menu/commands, old-layout migration, Preferences navigation
and draft retention, native setting persistence, folder/collection filters,
create/edit/save/duplicate/trash/restore, field validation, keyboard navigation,
and stale/recovery guards. Run full checks and frontend/native builds. Inspect
rendered light/dark and compact desktop layouts using isolated fixtures; native
checks never arm global input. Keep the user's preview available and ask them to
save/Quit before replacing the running executable. Record any unverified checks.
