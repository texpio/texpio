# Immediate expansion and correction

The user wants `;hello` to expand on its final character, without Space, and
` ;helllo` corrected with Backspace and retyping to work. Windows remains the
target and the existing UI, insertion safeguards and strict P0 engine remain.

## Approach

Use the existing bounded Rust typing history, matching after each printable
character and plain Backspace. Default to immediate activation; retain an
explicit Space/punctuation activation choice for users with overlapping triggers.
The first complete abbreviation wins in immediate mode (`;hi` expands before
someone can finish `;high`). Waiting for a delimiter remains an explicit option.
Reading arbitrary editor text or guessing cursor moves adds privacy and target
ambiguity, so neither is part of this change. Arrow keys, mouse clicks, shortcuts,
focus changes and uncertain translation invalidate history.

Retain 512 recent Unicode scalars to tolerate typos and subsequent corrections.
Remember when left context was truncated so erasing back to the buffer boundary
cannot invent a word boundary. Plain Backspace updates and rechecks the history;
Ctrl/Alt/Win Backspace resets it because it is not one-character deletion.

A matching key is suppressed only after its job is accepted. The expansion carries
the exact number of already-visible characters to erase: trigger length minus one
for the suppressed final character, trigger length plus one for a suppressed
Backspace which reveals a match, and full trigger length for a suppressed delimiter.
Only delimiter activation may append a terminating character. Own generated input,
target checks, cancellation and clipboard ownership remain unchanged.

## Acceptance

Automated tests cover immediate typing, typo correction before/after Space,
deletion revealing a match, prefixes, Unicode, boundaries after truncation,
shortcut resets, matching-key suppression and exact deletion plans. Existing
delimiter behavior remains covered. Settings round-trip with a missing activation
field defaulting to immediate; Preferences exposes both choices in all locales.
Run full checks and build. Native confirmation is separate and waits for a fresh
readiness reply; it must not be reported as passed from unit tests.

Implementation choices above are agent decisions within the user's existing
autonomous development authorization, not additional user requirements.

The user subsequently confirmed that correction used Backspace at the end.

## Reference behavior

[TextExpander expansion](https://textexpander.com/learn/using/snippets/expanding-snippets)
documents immediate activation and optional delimiters. Its
[prevention guide](https://textexpander.com/learn/using/snippets/expanding-snippets/preventing-expansion)
documents that arrows, mouse clicks and app changes interrupt recognition.
[Espanso's matcher middleware](https://github.com/espanso/espanso/blob/a87d1723f47a59e38d370e2f0c636d66d568b8f6/espanso-engine/src/process/middleware/matcher.rs)
provides a comparable bounded Backspace-history model. No upstream runtime code
was copied into Texpio.
