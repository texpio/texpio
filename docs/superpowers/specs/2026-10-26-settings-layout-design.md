# Settings layout

Use the user's Multica screenshot as the layout reference: keep the application
sidebar, add a separate settings sidebar, and show a spacious section heading and
bordered groups of rows. Labels/help sit on the left and controls on the right;
rows stack when their own content width is too narrow. Preserve Texpio's neutral
theme, Poppins typography and existing General/Expansion/Shortcuts/Data content.

Settings search filters the section menu using translated section names and the
labels of settings inside each section. It selects the first match when the
current section is excluded, shows a no-match message, and never writes settings.
Clearing search restores all sections without resetting the selected section.

The selected approach composes installed Field, ToggleGroup and InputGroup
primitives. Merely restyling the horizontal selector would miss the reference's
navigation structure; replacing the application sidebar would remove the context
shown in the reference. No dependencies, Rust commands or persistence change.
Keep one mounted product-settings mutation owner across all navigation and search.
Retain errors, pending-save disabling, native ownership, draft guards and all three
locales. Native preview replacement requires the user's save-and-Quit reply.
