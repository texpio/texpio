# Portable agent materials

Canonical repository instructions live in [AGENTS.md](../AGENTS.md). These files use plain Markdown and runtime-neutral instructions. Storing a role here does not automatically register a role, slash command or permissions in every assistant; load it explicitly when your runtime has no discovery support. No canonical `.claude` copy is maintained.

| Upstream path | Canonical destination |
| --- | --- |
| `.claude/skills/check/SKILL.md` | [skills/check/SKILL.md](skills/check/SKILL.md) |
| `.claude/skills/init/SKILL.md` | [skills/init/SKILL.md](skills/init/SKILL.md) |
| `.claude/skills/cleanup/SKILL.md` | [skills/cleanup/SKILL.md](skills/cleanup/SKILL.md) |
| `.claude/skills/change-package-manager/SKILL.md` | [skills/change-package-manager/SKILL.md](skills/change-package-manager/SKILL.md) |
| `.claude/agents/cleanup-analyzer.md` | [agents/cleanup-analyzer.md](agents/cleanup-analyzer.md) |
| `.claude/agents/userguide-reviewer.md` | [agents/userguide-reviewer.md](agents/userguide-reviewer.md) |

All six files are adapted from revision `1295f691bbdf373896e7c0f1cc969e2040fbf42c` under MIT. [Attribution](../THIRD_PARTY_NOTICES.md). Runtime-specific tool allowlists, argument variables and agent colors are intentionally omitted. Existing-project initialization, report-only cleanup and preservation-first package migration are exercised with read-only pressure cases; local mapping/link checks run with `bun run test:tooling`.
