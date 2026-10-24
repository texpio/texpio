# Texpio

A local Windows desktop text expander in development. The offline MVP includes a snippet library with explicit save, folders, search, duplicate, trash/restore, JSON import preview/export and a searchable Copy picker. Home opens the library; no account or backend is required, and sync is not implemented.

The desktop shell builds on the official Tauri 2 starter and the pinned dannysmith template, using React/TypeScript, shadcn, Zustand and TanStack Query with Bun 1.4.4. Rust owns the local SQLite library and native Start/Pause/Stop controls. Automatic expansion uses literal plain-text bodies, semicolon-prefixed triggers and direct Unicode input in strictly identified Windows Notepad editor controls. It starts stopped; saving or importing stops it and requires an explicit restart. Native insertion and lifecycle compatibility remain unqualified, with physical automatic trials at **0/300**. See [setup](docs/development.md), the [user guide](docs/userguide/userguide.md) and the [MVP acceptance record](docs/research/offline-mvp-acceptance.md) for usage and verification limits.

## Planning documents

| Document                                                                                              | Purpose                                                                                                         |
| ----------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| [Feature plan](docs/feature-plan.md)                                                                  | Audience, priorities, workflows, architecture options, compatibility, and quality targets                       |
| [Business plan](docs/business-plan.md)                                                                | Positioning, open-source model, paid-sync experiments, unit economics, and validation milestones                |
| [Competitor analysis](docs/competitor-analysis.md)                                                    | Saved competitors, initial product/SEO observations, and future research ideas                                  |
| [Phased PRD summaries](docs/phased-prds.md)                                                           | P0 feasibility → P1 Windows MVP → P2 workflows/macOS → P3 personal sync → conditional P4 teams                  |
| [Technology stack](docs/tech-stack.md)                                                                | Selected desktop tooling and reuse of the existing Laravel/PostgreSQL service later                             |
| [Desktop foundation design](docs/superpowers/specs/2026-10-23-desktop-foundation-design.md)           | Approved starter scope, desktop behavior, architecture, and acceptance checks                                   |
| [Foundation implementation plan](docs/superpowers/plans/2026-10-23-desktop-foundation.md)             | Approved tasks for prerequisite setup, scaffolding, desktop behavior, and verification                          |
| [Windows expansion probe design](docs/superpowers/specs/2026-10-23-windows-expansion-probe-design.md) | Approved P0 native Rust scope, guarded replacement, test controls, and evidence gate                            |
| [Windows expansion probe plan](docs/superpowers/plans/2026-10-23-windows-expansion-probe.md)          | Approved implementation tasks and native campaigns; Notepad read-only check passed, physical qualification pending |
| [Windows offline MVP design](docs/superpowers/specs/2026-10-24-offline-mvp-design.md)                 | Local library, additive JSON interchange, Copy picker and guarded Notepad expansion scope                       |
| [Windows offline MVP plan](docs/superpowers/plans/2026-10-24-offline-mvp.md)                           | Storage, UI, engine integration and acceptance work                                                              |
| [Windows offline MVP acceptance](docs/research/offline-mvp-acceptance.md)                            | Implemented behavior, actual verification evidence and remaining native/release gates                           |
| [Espanso source audit](docs/research/espanso-source-audit.md)                                         | Pinned native-engine research, four platform backends, architecture/reuse options and acceptance implications |
| [Template audit and parity](docs/research/tauri-template-parity.md)                                   | Pinned upstream feature inventory, portable agent mapping and migration evidence tracker                        |
| [Template adoption design](docs/superpowers/specs/2026-10-23-template-adoption-design.md)             | Approved full template shell with preserved engine, Bun and universal agent materials                           |
| [Template adoption plan](docs/superpowers/plans/2026-10-23-template-adoption.md)                      | Eight implementation tasks, parity checks and Windows acceptance; approved; implementation/evidence tracked in parity            |
| [Tasks](TASKS.md)                                                                                     | Planning progress, decisions, and future validation work                                                        |
| [Planning evidence](docs/planning-evidence.md)                                                        | Official sources and the limits of the research                                                                 |
| [Naming record](docs/naming-research.md)                                                              | Selection and GitHub checks                                                                                     |

The product plans are reviewable proposals. Windows first, then macOS and the desktop stack are confirmed; audience, feature details, license, price, quotas, targets, and native engine approach require review and validation. No performance or customer-demand results have been measured.

## GitHub identity

The user created the [Texpio GitHub organization](https://github.com/texpio) and the canonical [texpio/texpio repository](https://github.com/texpio/texpio). The organization was verified through GitHub's public API on 23 October 2026. Domains and brand clearance remain separate launch work.

## Contributor guides

See the [documentation index](docs/README.md), [portable skills and roles](.agents/README.md) and [third-party notices](THIRD_PARTY_NOTICES.md). Native acceptance and real signed update delivery are tracked separately from automated checks.
