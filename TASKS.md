# Texpio tasks

Updated: 23 October 2026 (Asia/Kolkata)

## Confirmed brief

- **Name:** Texpio, selected by the user.
- **Repository:** [texpio/texpio](https://github.com/texpio/texpio), created by the user.
- **Platform order:** Windows first, then macOS, confirmed by the user.
- **Product:** open-source, local-first desktop text expander; optional paid managed sync later.
- **Proposed stack:** Tauri desktop shell and separate Rust expansion engine, subject to feasibility.

The requested planning documents are complete for review. Their feature details, license, prices, quotas, budgets, targets, and audience remain proposals. The user created the Texpio GitHub organization; app implementation, runtime benchmarking, and customer validation remain future work.

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

## Future decisions and execution work

- [ ] Check desired domains/app-store names and brand clearance before publishing.
- [ ] Review the feature/business/phase proposals and decide the initial audience, license, supported Windows versions, and app matrix.
- [ ] Write and review the detailed P0 probe/spec before implementation.
- [ ] Run P0 and measure feasibility; revise proposed targets using evidence.
- [ ] Conduct interviews, beta cohorts, willingness-to-pay tests, and real cost estimates at the stated gates.
- [ ] Produce implementation plans only for approved phase scope; no application scaffolding is part of this planning task.
