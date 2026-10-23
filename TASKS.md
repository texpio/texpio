# Texpio tasks

Updated: 23 October 2026 (Asia/Kolkata)

## Confirmed brief

- **Name:** Texpio, selected by the user.
- **Repository:** [texpio/texpio](https://github.com/texpio/texpio), created by the user.
- **Platform order:** Windows first, then macOS, confirmed by the user.
- **Product:** open-source, local-first desktop text expander; optional paid managed sync later.
- **Selected stack:** official Tauri 2 starter, React + TypeScript + Vite, shadcn/ui; separate Rust expansion engine subject to feasibility.
- **Future service:** reuse the user's existing Laravel + PostgreSQL project for accounts, billing, and later sync; no backend is being built in this repository now.

The requested planning documents are complete for review. Their feature details, license, prices, quotas, budgets, targets, and audience remain proposals. The desktop foundation code is committed; its final lifecycle acceptance and the P0 native probe remain in progress. Runtime benchmarking and customer validation remain future work.

## 1. Naming research and selection

- [x] Generate a short, brandable shortlist and screen existing products.
- [x] Search public GitHub repository names and exact public handles.
- [x] Record the user's selection of Texpio.
- [x] Recheck Texpio after selection: public account API 404, public repository-name search zero results, and public nikitech0103/texpio lookup 404.
- [x] Record the initial verification limit: GitHub's signup validator returned 403, so the assistant could initially confirm only public-name absence.
- [x] Record the user's confirmation that GitHub showed texpio as available, followed by their creation of https://github.com/texpio.
- [x] Verify the created identity through GitHub's public API: HTTP 200, login texpio, type Organization, and zero public repositories at verification.

Deliverable: [Naming record](docs/naming-research.md). Texpio is selected and its GitHub organization is created and verified. Naming is complete.

## 2. Feature plan

- [x] Separate user-confirmed requirements from audience/positioning assumptions.
- [x] Compare product/engine approaches and recommend a direction to validate.
- [x] Prioritize features by phase and define MVP exclusions.
- [x] Describe trigger, picker, editor, exclusions, import/export, and recovery flows.
- [x] Address native input, Unicode/IME, clipboard, focus, password-field limits, permissions, and OS constraints.
- [x] Propose storage/sync boundaries and measurable quality/accessibility/privacy gates.

Deliverable: [Feature plan](docs/feature-plan.md). These are planned requirements; runtime and customer validation are future work.

## 3. Business plan

- [x] Research official competitor offerings/pricing and explain positioning hypotheses.
- [x] Define proposed free/open-source and optional paid-service boundaries.
- [x] Compare license candidates without applying a license.
- [x] Propose pricing, support, quotas, cancellation, and recovery experiments.
- [x] Calculate illustrative contribution and break-even scenarios with explicit assumptions/exclusions.
- [x] Define acquisition, demand, cohort, cost, and stop/re-scope experiments.

Deliverable: [Business plan](docs/business-plan.md). Prices and economics are assumptions, not published offers, quotations, or forecasts. Experiments have not been conducted.

## 4. Phased PRD summaries

- [x] P0: summarize the native Windows engine feasibility question and evidence gate.
- [x] P1: summarize the Windows offline MVP and release criteria.
- [x] P2: summarize workflow improvements and independent macOS validation.
- [x] P3: summarize conditional paid personal sync, integrity/recovery requirements, and demand gate.
- [x] P4: summarize conditional team libraries and access/ownership requirements.
- [x] Include users/problems, scope, exclusions, dependencies, risks, acceptance evidence, and phase decisions.

Deliverable: [Phased PRD summaries](docs/phased-prds.md). Writing a phase summary does not mean the phase has been implemented or validated.

## 5. Competitor research — initial capture complete

- [x] Save the five user-supplied competitor links and identify the Microsoft Store product.
- [x] Record dated public-page observations, primary sources, evidence limits, and hypotheses about discovery and adoption.
- [x] Save future SEO, product, Store, distribution, and growth research tasks as CR-01 through CR-10.
- [ ] Run the detailed competitor research passes and validate the ranking/success hypotheses when this work is prioritized.

Deliverable: [Competitor analysis and research backlog](docs/competitor-analysis.md). Rankings, traffic, conversion, revenue, and success causation have not been established.

## Review and evidence

- [x] Complete an independent review of scope, cross-document consistency, proof claims, and economic arithmetic.
- [x] Align the retention definition across the business plan and PRDs.
- [x] Verify all local document links, inspect conflict markers, and independently recompute contribution/break-even figures before handoff.

Reference sources: [Planning evidence](docs/planning-evidence.md). Document index: [README](README.md).

## 6. Desktop foundation

- [x] Record the user's choice of the official Tauri starter and the agreed desktop stack.
- [x] Write the foundation design covering scope, lifecycle, IPC, tooling, and acceptance checks.
- [x] Audit local prerequisites and record missing standard Rust/MSVC/SDK installations.
- [x] Complete written-spec review with the user.
- [x] Write the foundation implementation plan and perform its self-review.
- [x] Complete the user's implementation-plan review and choose its execution method: Native, current checkout.
- [x] Generate and integrate the official starter, pin tools, and install Windows build prerequisites.
- [ ] Add and verify the foundation features/checks.
- [ ] Verify native Windows build and runtime behavior; record any missing prerequisites.
- [ ] Commit and push the verified implementation.

Deliverables: [Technology stack](docs/tech-stack.md), approved [foundation design](docs/superpowers/specs/2026-10-23-desktop-foundation-design.md), and approved [implementation plan](docs/superpowers/plans/2026-10-23-desktop-foundation.md). Implementation is in progress.

The starter, themed screen/metadata, and tray/single-instance changes are committed through `22fadcb`. The Windows preview rendered Connected with version `0.1.0` and platform `windows`. Remaining lifecycle and remote CI evidence still needs recording; the unchecked foundation gates above remain open.

## 7. Windows expansion feasibility

- [x] Start P0 and record the user's selection of a native Rust engine with Windows APIs.
- [x] Research official Windows observation, insertion, accessibility, and clipboard constraints.
- [x] Capture ideas from the eight user-supplied TextExpander screenshots in the competitor analysis, mapped to later phases.
- [x] Draft the detailed [Windows expansion probe design](docs/superpowers/specs/2026-10-23-windows-expansion-probe-design.md).
- [x] Complete the design's independent builder review and resolve its questions about physical trials, key-release ordering, and picker edits.
- [x] Complete written-design approval with the user.
- [x] Write and self-review the [implementation plan](docs/superpowers/plans/2026-10-23-windows-expansion-probe.md) for approved P0 scope.
- [x] Complete implementation-plan review; preserve Native execution in the current checkout.
- [ ] Implement and verify the native probe, controls, and separate insertion/picker experiments.
- [ ] Run and report the native campaigns, strategy comparison, and support decision.

P0 is paused at the user's request for the template migration below. Probe controls are implemented and the initial Stopped screen was confirmed by the user. No input strategy is qualified; the English (India) layout investigation and physical trials remain open. See the [evidence report](docs/research/windows-expansion-probe.md).

## 8. Full template adoption and Bun

- [x] Inspect `dannysmith/tauri-template` at pinned revision `1295f691bbdf373896e7c0f1cc969e2040fbf42c`, including README, implementation, docs, skills and license.
- [x] Record the user's choice: full template shell, preserve the existing engine in a Developer view.
- [x] Inventory every runtime/development feature and all four skills/two role prompts; distinguish implementation from README claims.
- [x] Verify the installed Bun version (`1.3.13`) and research migration/Windows compatibility.
- [x] Write the migration design and complete its independent builder check.
- [ ] Obtain approval of the written [migration design](docs/superpowers/specs/2026-10-23-template-adoption-design.md).
- [ ] Prepare the implementation plan, preserving Native/current-checkout execution.
- [ ] Import/adapt the shell, typed bridge, preferences, quick pane, supporting plugins, UI library and development tooling.
- [ ] Migrate package management/scripts/CI/docs to a validated pinned Bun version and text lockfile.
- [ ] Port the four skills and two role prompts into `.agents`; merge useful upstream `AGENTS.md` guidance while preserving computer-availability rules.
- [ ] Verify the feature parity table with code/test/native evidence and explicit external/platform limits.
- [ ] Commit/push verified changes and resume P0 afterward.

Deliverables: [template source audit and parity tracker](docs/research/tauri-template-parity.md), [migration design](docs/superpowers/specs/2026-10-23-template-adoption-design.md). No template runtime code or project skill has been imported yet.

## Future decisions and execution work

- [ ] Check desired domains/app-store names and brand clearance before publishing.
- [ ] Review the feature/business/phase proposals and decide the initial audience, license, supported Windows versions, and app matrix.
- [x] Approve the detailed P0 probe/spec before implementation.
- [ ] Run P0 and measure feasibility; revise proposed targets using evidence.
- [ ] Conduct interviews, beta cohorts, willingness-to-pay tests, and real cost estimates at the stated gates.
- [ ] Produce implementation plans for reviewed scope, starting with the desktop foundation above and then the P0 probe.
