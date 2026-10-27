# Rich snippet editor

Use locally bundled, open-source Tiptap packages and include their and their
transitive components' license notices in the distributed application. Offer a
compact accessible toolbar for bold, italic, underline, links and lists using the
existing theme. Preserve existing editing, shortcuts, drafts, accessibility and
offline operation. Imported content and links must be handled as untrusted.

Old snippet bodies remain literal plain text, including tabs, line endings and
literal markup. Rich snippets store an explicit, versioned Tiptap document beside
their plain-text projection; never reinterpret old text as HTML. Backups, imports,
exports and duplicates preserve both. Validate bounded documents and verify their
plain-text projection matches the saved body so the editor and expansion cannot
diverge. Fail closed on unknown content and unsafe links.

Paste mode can supply sanitized/generated HTML and Unicode plain text together,
using the existing clipboard ownership and restoration safeguards. Type mode and
plain-text targets receive the plain projection. Preserve attachment order,
cancellation and pre-erase preparation. Native rich insertion is an input behavior
change: keep it gated until the applicable native compatibility check has passed;
do not infer cross-application or physical P0 qualification from synthetic tests.

Regression tests cover legacy records and backup migrations, document validation,
literal whitespace/Unicode, round trips, draft/undo/find behavior, safe links,
clipboard format encoding and ownership. Run pinned-toolchain quality checks,
production build, offline synthetic UI tests and bounded native verification as
authorized. Keep the current app and user data intact during development.
