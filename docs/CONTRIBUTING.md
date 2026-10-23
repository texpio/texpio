# Contributing

Source and proposed changes belong in [texpio/texpio](https://github.com/texpio/texpio). Read [setup](development.md), [architecture](developer/architecture-guide.md), [testing](developer/testing.md) and [agent instructions](../AGENTS.md).

Keep changes focused on actual problems. Preserve pinned tools, frozen/locked installs, typed IPC, accessibility, locales and privacy. Run `bun run check:all`, `bun run build` and relevant native no-bundle builds. Describe behavior, reason, validation and pending native checks in reviews.

Ordinary tests never arm hooks/send input. Physical qualification is a separate campaign. Keep reusable UI/plugin infrastructure unless removal is authorized and caller evidence supports it.

Texpio's own license remains undecided; upstream MIT notices do not establish the license of all original work. No support response-time commitment is made.

Adapted from [tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../THIRD_PARTY_NOTICES.md).
