---
name: cleanup
description: Use when the user requests static cleanup analysis, unused-code review or duplication recommendations.
---

# Cleanup

Use the [cleanup analyzer role](../../agents/cleanup-analyzer.md) in a read-only agent if available; otherwise perform the role directly. Run the configured Knip, jscpd and quality checks and investigate reported uses before recommending removal.

Return a report grouped into safe candidates, needs review, intentional infrastructure, duplication worth extracting and check failures. This analysis is report-only: do not edit application files or create task documents unless the user has already authorized that work. A report-only request needs no extra follow-up question. Existing authorization to fix or create tasks remains valid.

When requested, save concrete follow-up tasks in `docs/tasks-todo/` and link them from canonical [TASKS.md](../../../TASKS.md). See [task workflow](../../../docs/tasks.md).

Adapted from [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c), MIT; see [notice](../../../THIRD_PARTY_NOTICES.md).
