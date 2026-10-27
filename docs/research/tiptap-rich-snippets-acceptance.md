# Tiptap rich snippets — work in progress, 27 October 2026

The user chose Tiptap and asked that formatting survive expansion where the target
supports it. Tiptap 3.32.0 and its dependencies are pinned and their upstream
copyright and license notices are bundled under `public/licenses/`.

The editor supports bold, italic, underline, lists, links, preview and structured
find/replace. Rich documents and plain-text fallback are saved together, validated
against one another, and included in versioned JSON/YAML/archive exports. Existing
plain records remain literal. Snippets with carriage-return line endings remain
read-only until the user explicitly confirms conversion to LF; the draft can be
discarded before saving. Plain-text paste preserves blank lines and list markers;
pasting CR requires explicit conversion. Import validation rejects duplicate keys,
unsafe links and unsupported structures. User-facing labels cover all three locales.

SQLite v1–v3 databases receive a pre-v4 snapshot before migration; attachments stay
in the original attachments directory. Restoring that snapshot requires the original
blobs and discards changes made after migration. Older releases reject v4 databases
and v2 rich exports; do not silently downgrade them.

## Outstanding acceptance gate

**Formatted expansion is not enabled.** The native clipboard encoder and delivery
path are prepared and covered by synthetic tests, but the production gate is false:
snippets expand as plain text and the editor says so. A safe owned-target native
check must demonstrate actual formatted insertion, exact plain fallback, clipboard
ownership/restoration and no unintended input before the gate can be enabled.
Current `SendInput` delivery has a focus-change race, and the existing Notepad
fixture cannot independently verify rich formatting. No safe target-directed rich
fixture has yet been established. No physical P0 qualification is claimed.

## Verification so far

The frozen Bun installation and `bun run check:all` passed: 201 frontend tests,
223 Rust library tests, 12 tooling tests, one fixture-server test and all static
gates. `bun run build` passed; Vite reported a large frontend chunk warning.
An isolated headless production-frontend fixture verified formatting, saving,
preview, reload, bundled license notices, light/dark and compact layout. It used
synthetic native replies, then closed its own browser and loopback server.
Read-only reviews identified and drove fixes for untrusted documents, paste
normalization, mixed text direction, SQLite migration recovery and legacy line
endings. These checks do not qualify native formatted insertion or native IME.

The user saved drafts, quit and explicitly requested the preview. The locked
optimized Windows build passed; the normal preview reopened (observed PID 26068).
A bounded read-only check confirmed the visible main window and that the library
loaded after migration. The app remains open. The editor's rendered formatting
was checked in the isolated fixture, not through native user interaction.
Executable SHA-256:
`4E64A09CCF2B872031E3D160FC71420AEACC457A1051AC9D89CAA9A3A97D661A`.
No user's clipboard, snippets or desktop were used in the synthetic tests. See the
design and plan under `docs/superpowers/`.
