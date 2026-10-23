# UI patterns

Compose existing shadcn/Radix primitives, React 19 ref props, Tailwind 4 semantic OKLCH tokens and the canonical theme provider. Preserve focus, keyboard navigation, dialog titles/descriptions and disabled/busy states.

Use `cn()` and existing variants. Native appearance or live OS theme controls dark mode. Logical CSS supports Arabic. Keep titlebar interactive children outside drag regions.

Main has left/content/right resizable panels; restore widths only after actual layout and constrain them. Home/Developer are real views; palette/Preferences/update confirmation are accessible dialogs. Busy install blocks dismissal; cancellation releases update resources.

Use `bun x --no-install shadcn` and installed docs when adding a primitive. Do not silently reinitialize theme, upgrade dependencies or delete reusable UI.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
