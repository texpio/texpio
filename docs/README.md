# Texpio documentation

Continuing development on another PC: read the [24 October MVP handoff](handoffs/2026-10-24-offline-mvp.md), including the failed native insertion trial and remaining review/acceptance work.

Start with [development setup](development.md), [developer guides](developer/README.md), [user guide](userguide/userguide.md), [contributing](CONTRIBUTING.md), [security](SECURITY.md) and [task workflow](tasks.md).

Product research and approved plans remain in the [repository index](../README.md). [Template parity](research/tauri-template-parity.md) tracks implementation and acceptance separately. [TASKS.md](../TASKS.md) is canonical. [Template use notes](USING_THIS_TEMPLATE.md) explain adoption boundaries.

The Windows offline MVP opens a local snippet library on Home, with explicit save, folders, search, trash/restore, additive JSON import/export and a searchable Copy picker. It requires no account or backend; sync is not implemented. Read the [MVP design](superpowers/specs/2026-10-24-offline-mvp-design.md), [implementation plan](superpowers/plans/2026-10-24-offline-mvp.md) and [acceptance record](research/offline-mvp-acceptance.md). Native Notepad insertion and lifecycle compatibility remain unqualified; the acceptance record separates automated checks from physical trials.

The [Espanso source audit](research/espanso-source-audit.md) records native platform mechanisms, implementation tradeoffs and proposed engine options. The [Windows probe evidence](research/windows-expansion-probe.md) retains observed results and outstanding qualification gates.

Adapted from [tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../THIRD_PARTY_NOTICES.md).
