# Library workspace acceptance — 25 October 2026

Development base: `17c01c3f944f99ac80ed1ece4bcb57308304492e`.
The user requested removing the right sidebar/toggle and Developer page, a
dedicated Preferences page, and improved snippet creation/management. They
explicitly selected a combined list and editor. Current-checkout execution and
commit/push authorization continue from the handoff.

## Result

- The global right sidebar, its title-bar button, command and native menu entry
  are removed. Old persisted right-pane keys are ignored during normalization.
- The left navigation contains All snippets, Favorites, Trash, Unfiled and
  folders, with folder creation/management and Preferences. Pane widths remain
  bounded and resizable; only sanitized layout values are persisted.
- The workspace gives the snippet list and editor independent scroll areas,
  keeps search above the list and gives New snippet a clear primary action.
  Import/export/reload are in Library actions; backups moved into Preferences.
- The editor focuses Title for new records, keeps Save in its header, groups
  trigger/folder/enabled controls, and separates Content, Preview and Attachments.
  Favorite, Copy, Duplicate, Trash/Restore and literal Find/Replace remain.
- New snippets clear search and leave Favorites/Trash so saved records remain
  visible, while inheriting the chosen folder. Duplicates preserve their source
  folder and start disabled. Invalid disappeared folder filters fall back to All.
- Preferences is a dedicated page with General, Expansion, Shortcuts and Data
  sections. Controllers remain mounted across page changes so unsaved snippet
  drafts, pending settings saves and failure state survive navigation. One
  product-settings mutation owner serves all Preferences sections.
- Developer is removed from product navigation and command registration. Strict
  probe source, native guards and qualification records remain available in the
  repository. The obsolete last-quick-entry retention/listener is removed.
- Existing neutral shadcn/Radix styling and three root locales remain; Texpio's
  title is consistent across languages and snippet fields use automatic text
  direction for mixed-language content.

## Verification

`bun run check:all` passed: **162 frontend tests in 32 files, 188 Rust library
tests, 12 tooling tests and one fixture-server test**, plus TypeScript, ESLint,
AST checks, Prettier, rustfmt and all-target Clippy. No dependency, lockfile,
Rust command or input-engine change was required.

Meaningful regressions were observed failing before their fixes: modal Preferences
and removed native menu entries, initial new-snippet focus, a disappeared folder,
creating a snippet while search/Favorites hides it, a delayed settings save across
sections, and body validation hidden in Preview. Existing Unicode saves,
duplicate-trigger validation, stale revisions, metadata refresh, native Quit draft
confirmation, literal search/replace, folder removal and Trash/Restore tests pass.

The introduction of resizable panes exposed a jsdom limitation: its zero-sized
handle rectangles intercepted every simulated click. Test-only realistic panel
geometry fixed the environment; no product input behavior was changed to work
around it. Rendered browser checks subsequently exercised the real DOM geometry.

## Independent review

The fresh read-only whole-change reviewer reported no Critical issues, one
Important issue and one Minor issue:

1. Changing Preferences sections remounted product-settings mutations, allowing a
   second stale full-record save while the first response was delayed. A test
   reproduced enabled controls during the pending save. One controller now stays
   mounted across sections and main-page navigation; subsequent saves retain the
   acknowledged prior change. Failed-save visibility also has a regression test.
2. Body validation could be hidden when Save was pressed in Preview/Attachments.
   A control-character reproducer failed before a persistent visible error was
   added outside Content.

Both were fixed and the whole suite rerun. This records the original review and
the regression-tested fix pass, not a second reviewer approval.

## Rendered checks

An isolated, headless installed Chrome instance served the production frontend
over loopback with synthetic in-memory native replies. It checked:

- List/editor navigation, an edited Unicode body, Preview and returning from
  Preferences without losing the draft or saving it implicitly.
- Appearance changes, categorized Preferences and a delayed product save across
  both section switches and leaving/reopening Preferences.
- Light/dark layouts, Windows title-bar rendering, compact 800×560 layout and
  Arabic right-to-left layout without page-level horizontal overflow.
- No page errors, Developer/right-toggle controls, `product_start`, `probe_start`
  or library-save calls during the visual campaign.

All assertions passed. Screenshots and full logs are ignored local artifacts under
`.superpowers/sdd/2026-10-25-library-workspace/`. The headless browser and fixture
server ended after every run. Edge was absent; the installed Chrome executable
was used without installing a browser or touching an existing browser profile.

These checks establish rendered UI behavior, not delivery of native OS shortcuts,
file dialogs or text insertion. The UI campaign never armed global input. Prior
native expansion evidence and the unchanged strict P0 qualification limits are
recorded separately in [immediate expansion acceptance](immediate-expansion-acceptance.md).

## Windows packages and preview

The user saved and quit the previous preview before rebuilding.
`bun run tauri build -- --locked` passed, producing the optimized executable,
MSI and NSIS packages. Native compilation took 5m 10s. No installer was run and
no signed/public release was published.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `src-tauri/target/release/texpio.exe` | 12045312 | `D17C9F7F1343AD9CDC2EFEBB600DBA74585BCFA74414C60EF2AD4313AFFC05C9` |
| `src-tauri/target/release/bundle/msi/Texpio_0.1.0_x64_en-US.msi` | 5820416 | `592E927E0E0B349392718FC00A03E5C577B5498AE4C8D729A2ABC7C6CA8298FB` |
| `src-tauri/target/release/bundle/nsis/Texpio_0.1.0_x64-setup.exe` | 4243113 | `09FABBFD7D7ED7DC840418C919828C6A8EF1EDFE7CE2937E1A21482D6F45BAA7` |

The normal preview was reopened from the release path (observed PID 24792, title
Texpio) without remote-debugging arguments. Its library and saved preferences were
not edited by the UI fixture. Reobserve process identity before future actions;
ask the user to save and Quit before replacing this executable again. The
headless browser/server have ended and no test-port listener remains.

See the [design](../superpowers/specs/2026-10-25-library-workspace-design.md),
[plan](../superpowers/plans/2026-10-25-library-workspace.md) and [TASKS](../../TASKS.md).
