# Texpio

A local-first desktop text expander in development, targeting Windows first and macOS next. Optional paid managed sync is planned later.

The desktop shell builds on the official Tauri 2 starter and the pinned dannysmith template, using React/TypeScript, shadcn, Zustand and TanStack Query with Bun 1.4.4. Home, Developer, Preferences, command palette and quick entry are implemented. The Rust expansion probe remains experimental with physical qualification incomplete. See [setup](docs/development.md), [user guide](docs/userguide/userguide.md) and [feature parity](docs/research/tauri-template-parity.md) for verification limits.

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
| [Windows expansion probe plan](docs/superpowers/plans/2026-10-23-windows-expansion-probe.md)          | Approved implementation tasks and physical/native acceptance campaigns; currently paused for template migration |
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
