# Task workflow

Root [TASKS.md](../TASKS.md) is canonical; approved plans remain in `docs/superpowers/`. Optional follow-ups can live in `docs/tasks-todo/`, linked from the backlog.

`bun run task:complete UNIQUE_NAME` archives one uniquely selected Markdown task into `docs/tasks-done/` with a UTC date prefix. Exact filenames take priority. Ambiguity, traversal, redirected paths and destination collisions refuse. Unrelated tasks are preserved. Existing completed tasks are not bulk-renamed implicitly.

Cleanup is report-only unless edits/tasks are authorized. [Release preparation](developer/releases.md) changes local versions only.

Adapted from [tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../THIRD_PARTY_NOTICES.md).

For an explicitly requested existing archive normalization, run `bun run task:rename-done --dry-run` first, then omit `--dry-run` to apply. It dates undated files from their UTC modification date, skips already dated files and preflights collisions before moving any task.
