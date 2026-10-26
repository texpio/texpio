# Texpio tasks

Updated: 24 October 2026 (Asia/Kolkata)

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
- [x] Add the foundation features and verify automated checks.
- [ ] Verify native Windows build and runtime behavior; record any missing prerequisites.
- [x] Commit and push the verified implementation.

Deliverables: [Technology stack](docs/tech-stack.md), approved [foundation design](docs/superpowers/specs/2026-10-23-desktop-foundation-design.md), and approved [implementation plan](docs/superpowers/plans/2026-10-23-desktop-foundation.md). Implementation is in progress.

The official-starter history is preserved. The adopted shell, typed services, review fixes and automated checks are committed through `1aae8e1` and pushed. The user confirmed tray Open/Quit on the earlier preview and basic opening of the migrated shell and launchers. Detailed acceptance on the final migrated binary remains pending. See template parity for exact evidence.

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

P0 resumed on 24 October after the template migration. Probe controls are implemented and the initial Stopped screen was confirmed by the user. The India refusal was isolated to the broad IME guard; positive UIA composition evidence now replaces foreign-thread IMM queries. The user-assisted Notepad read-only preparation/revalidation passed. No input strategy is qualified; other provider checks and physical trials remain open. See the [evidence report](docs/research/windows-expansion-probe.md).

## 8. Full template adoption and Bun

- [x] Inspect `dannysmith/tauri-template` at pinned revision `1295f691bbdf373896e7c0f1cc969e2040fbf42c`, including README, implementation, docs, skills and license.
- [x] Record the user's choice: full template shell, preserve the existing engine in a Developer view.
- [x] Inventory every runtime/development feature and all four skills/two role prompts; distinguish implementation from README claims.
- [x] Verify the installed Bun version (`1.3.13`) and research migration/Windows compatibility.
- [x] Write the migration design and complete its independent builder check.
- [x] Obtain approval of the written [migration design](docs/superpowers/specs/2026-10-23-template-adoption-design.md).
- [x] Prepare and self-review the [implementation plan](docs/superpowers/plans/2026-10-23-template-adoption.md), preserving Native/current-checkout execution.
- [x] Complete implementation-plan review with the user before importing the template; approved on 23 October 2026.
- [x] Import/adapt the shell, typed bridge, preferences, quick pane, supporting plugins, UI library and development tooling.
- [x] Migrate package management/scripts/CI/docs to validated Bun 1.4.4 and text lockfile; retain Node/Rust pins.
- [x] Port all four skills/two roles into `.agents`; merge architecture guidance and preserve computer-availability rules.
- [x] Record all 43 feature rows with code/test evidence and explicit native/platform/external limits.
- [ ] Complete new-binary Windows native interaction acceptance, packaged notifications and multi-monitor/DPI checks.
- [x] Commit/push reviewed migration changes; remote SHA matches and the CI connector's 404 access limitation is recorded.
- [x] Resume P0 layout/provider diagnosis after migration handoff.

Deliverables: [template source audit and parity tracker](docs/research/tauri-template-parity.md), [migration design](docs/superpowers/specs/2026-10-23-template-adoption-design.md). All template runtime infrastructure and six portable skill/role files are adapted. Review fixes, automated checks and the optimized Windows no-bundle build pass; the debug preview and basic opening were confirmed. Detailed native acceptance remains open.

## 9. Espanso source and engine architecture research

- [x] Pin Espanso's stable source tag and development HEAD; record release/source evidence separately.
- [x] Audit Windows detection/layout/injection/clipboard and compare them with Texpio's P0 guarantees.
- [x] Audit macOS, Linux/X11 and Linux/Wayland mechanisms, permissions and lifecycle limits.
- [x] Inspect the shared matcher/render/effect pipeline, app configuration, manual testing and configured CI.
- [x] Document independent modeling, selected-source reuse and sidecar integration options, including the pending GPL licensing question.
- [x] Save a concrete proposed engine model and per-platform validation sequence; distinguish proposals from approved implementation.
- [x] Record the user's selection: independent Rust engine modeled on Espanso's architecture.
- [x] Preserve verified-target insertion for the first offline build; defer a history-only compatibility mode as an explicit future decision.
- [ ] Approve the resulting engine design/amendment and implementation plan before changing the expansion contract.

Deliverable: [Espanso source audit and Texpio engine options](docs/research/espanso-source-audit.md). No Espanso runtime code was added to Texpio, and upstream code was not built, installed or run. P0 remains unqualified with 0/300 physical trials.

## 10. Windows offline MVP

The user requested autonomous implementation and authorized Computer Use/app testing while offline on 24 October. See the [MVP design](docs/superpowers/specs/2026-10-24-offline-mvp-design.md), [execution plan](docs/superpowers/plans/2026-10-24-offline-mvp.md) and [acceptance record](docs/research/offline-mvp-acceptance.md). This development milestone does not declare the original P0/P1 release campaigns complete.

- [x] Implement a Rust-owned SQLite library, revision checks, bounded validation, transactional saves/import and recoverable JSON snapshots.
- [x] Implement folders, search, snippet editing/duplicate/trash/restore, explicit Copy and import/export preview workflows.
- [x] Replace fixed-only matching with a validated immutable rule snapshot shared by the observer and insertion worker.
- [x] Add an independent product policy for ordinary Notepad editors without broadening strict P0 fixture profiles.
- [x] Add explicit native Start/Pause/Stop controls and invalidate active/paused old rules before library changes.
- [x] Turn the quick pane into a searchable snippet Copy picker; keep captured-target insertion separate.
- [x] Pass whole-project checks (138 frontend/111 Rust tests plus tooling/static checks) and a Windows debug no-bundle build.
- [x] Complete the whole-change independent review; fix stale Resume publication and native Quit draft loss with RED/GREEN regressions.
- [x] Finish the optimized MVP executable/MSI/NSIS build and record artifact hashes; installation/signing remain unverified.
- [x] Save the portable cross-PC handoff and exact unresolved native evidence before the user-requested checkpoint.
- [x] Verify actual-app Unicode save/restart, duplicate/trash/restore, Quit draft cancellation/discard, explicit clipboard Copy, picker focus/Copy/Escape, close-to-hide and second-launch restoration using isolated synthetic data.
- [ ] Complete file-dialog scopes, physical tray gestures and native insertion acceptance. Fresh Notepad trials fully submitted input but failed exact output even after TextExpander exited; an owned Win32 EDIT receiver passed. See the acceptance record and handoff.
- [ ] Run physical typing, distribution, broader application/platform and original release-quality campaigns.

Current development support target: ordinary Notepad editor controls, qualified US/English India layout mappings, literal lowercase `;` triggers followed by Space, Unicode plain-text bodies. The code retains exact suffix/caret/password/composition/integrity checks; native insertion is failing qualification and must not be called working. Explicit Copy is implemented independently of automatic expansion. Date/time/clipboard/cursor tokens, rich text, sync, macOS/Linux delivery and signing remain later work. The user requested a commit/push checkpoint and [handoff to another PC](docs/handoffs/2026-10-24-offline-mvp.md) before this phase was complete.

## 11. Comparative expansion source research

- [x] Clone/pin Snipt and inspect its actual capture/insertion paths and lockfile-qualified Enigo/rdev sources.
- [x] Compare Espanso/Snipt mechanisms and limitations for Windows, macOS, X11 and Wayland with Texpio's contract.
- [x] Clone/pin Ampello, trace Tauri integration and native backends, and inspect pacing/cancellation/clipboard behavior and configured tests.
- [x] Save source-linked lessons and bounded next experiments without importing upstream runtime code or changing the approved insertion contract.

Deliverables: [Espanso/Snipt comparison](docs/research/cross-platform-expansion-source-comparison.md) and [Ampello audit](docs/research/ampello-source-audit.md). Ampello is a close stack/product reference; it explicitly refuses Wayland. Static inspection and upstream compatibility claims do not qualify Texpio's native effects.

## 12. Windows v1 — Ampello features in Texpio UI

The user installed Ampello successfully on Windows and requested its complete feature set for the first working prototype, preserving Texpio's UI/UX. Windows delivery takes priority over other OS work and the old Notepad-only product contract. Existing autonomous Native/current-checkout execution remains authorized. See the [design](docs/superpowers/specs/2026-10-24-windows-v1-design.md), [plan](docs/superpowers/plans/2026-10-24-windows-v1.md) and [parity tracker](docs/research/windows-v1-parity.md).

- [x] Implement independent product matching: boundaries, longest trigger, punctuation/Tab/Enter, Unicode, Backspace and reset behavior.
- [x] Implement Windows Paste/Type effects, cancellation/target generations and bounded hook/worker lifecycle; keep strict Developer probe separate.
- [x] Observe exact synthetic Notepad Paste/Type fixtures and verify actual-app Start/Pause/Resume/Stop/settings lifecycle. Modern Notepad Type uses a conservative 100 ms interval; strict observer timeouts remain uncertain where recorded.
- [x] Connect product controls/tray, refreshed snapshots after saves, generated typed commands and main-only capabilities.
- [x] Add expansion preferences, empty/large bodies and Unicode triggers; migrate favorites/usage metadata without staling editor revisions.
- [x] Implement attachments, ordering/delay and selected-file intake; ordered native file-only delivery passed. Selected-file and mixed-payload acceptance remains below.
- [x] Implement personal/shared data location, external refresh and failure rollback, with stale-write and real-storage regressions.
- [x] Implement YAML/archive interchange, replace mode and preserved-database recovery, including decompression bounds and shared-library corruption recovery.
- [x] Implement favorites/dashboard and editor keyboard/find/replace workflows; actual-app favorites, draft refresh, literal replacement and undo checks passed.
- [x] Implement clipboard/toggle hotkeys, opt-in hidden autostart and configurable close behavior; hidden launch, owned startup entry, second-instance restore and close/draft checks passed.
- [x] Verify manual software-compatible trigger expansion with exact output, submission and usage evidence; add default-off compatibility preference for remote keyboards/remappers.
- [x] Complete one independent whole-change review/fix pass, full quality checks and optimized executable/MSI/NSIS builds.
- [ ] Qualify remaining native selected-file/import/export/location dialogs, clipboard/toggle hotkey delivery, mixed text/files and broader target behavior. Do not infer these from unit tests or the successful trigger fixture.

Latest complete quality run: 152 frontend/174 Rust tests plus tooling/static gates. One fresh whole-change review found seven substantive issues; the regression-tested fix pass is recorded in the [review record](docs/research/windows-v1-review.md). Optimized executable, MSI and NSIS builds passed. These are prototype delivery checkpoints, not completed universal parity or strict physical qualification.

The trigger blocker was traced to Windows marking all user keystrokes as software-injected. With the explicit compatibility option enabled, the manual owned fixture passed: exact text plus terminator, one match, one submission and one usage record; Texpio's own injected output was ignored. The normal optimized preview is open with compatibility enabled for this PC and automatic expansion stopped at launch. See the [current acceptance record](docs/research/windows-v1-acceptance.md). No backend/sync or other OS delivery is in this scope.

## 13. Immediate activation and Backspace corrections

The user requested TextExpander-style immediate activation and confirmed that the
misspelled abbreviation was corrected with Backspace at the end. See the
[acceptance record](docs/research/immediate-expansion-acceptance.md).

- [x] Default to immediate completion while retaining optional Space/punctuation activation.
- [x] Preserve bounded correction history, reject shortcut edits and prevent truncated context from inventing word boundaries.
- [x] Account for suppressed completing keys without deleting preceding text or appending an unwanted Space.
- [x] Expose the activation preference in the existing UI and all three locales.
- [x] Complete independent review and reproduce/fix unmatched-control history crossing browser fields.
- [x] Pass 154 frontend and 188 Rust tests plus static/tooling checks and the isolated Windows build.
- [x] Verify user-typed correction with exact native output retaining the preceding `prefix ` and no trailing Space; record the separate disposable-profile cleanup conflict.
- [x] Finish optimized packages and reopen the updated preview.

## 14. Library workspace and dedicated Preferences

The user chose list/editor together and requested removal of the right sidebar
and Developer page. See the [workspace acceptance record](docs/research/library-workspace-acceptance.md).

- [x] Remove right-sidebar state, title-bar toggle, commands and native menu entry.
- [x] Remove Developer navigation and product command registration while preserving probe source/evidence.
- [x] Add folder/collection navigation and independently scrolling snippet list/editor panes.
- [x] Replace the Preferences dialog with General/Expansion/Shortcuts/Data pages and retained mutation state.
- [x] Improve New/Save, editor sections, copy/favorite/duplicate/trash actions, stale folder fallback and validation feedback.
- [x] Preserve drafts and pending saves across navigation; complete independent review and regression-tested fixes.
- [x] Pass 162 frontend/188 Rust tests plus static/tooling checks and isolated light/dark/compact/RTL rendered checks.
- [x] Finish optimized Windows packages and reopen the updated preview.

26 October follow-up: title-bar/sidebar wordmarks are removed, a small title-bar
icon replaces them, and the missing native double-click permission is corrected.
Full checks, rendered fixtures and the optimized native build pass; the updated
preview is visibly open. Actual native mouse double-click acceptance remains
unverified. See the
[title-bar acceptance record](docs/research/titlebar-polish-acceptance.md).

## 15. Settings layout based on the Multica reference

- [x] Add a dedicated settings sidebar, translated search and section headings.
- [x] Group existing controls into bordered, responsive label/control rows.
- [x] Preserve settings-save serialization, errors, shortcuts and unsaved snippets.
- [x] Complete read-only review, reproduce/fix translated action clipping and pass
  163 frontend/188 Rust tests plus static/tooling and rendered checks.
- [x] Rebuild/reopen the native preview after save-and-Quit acknowledgement.

See the [settings layout acceptance record](docs/research/settings-layout-acceptance.md).

## 16. Screen-aware launch and expansion startup defaults

- [x] Follow the screen's aspect ratio at startup and ignore old main-window bounds.
- [x] Start expansion on each fresh Windows launch/restart; keep session Stop/Pause
  and cancellation/validation guards, and remove the conflicting launch switch.
- [x] Rename Unfiled to Uncategorized and the sidebar command-search entry to Search.
- [x] Complete read-only review/fix, 163 frontend/197 Rust tests, static/tooling gates
  and headless UI checks without native input.
- [x] Rebuild/reopen after save/Quit acknowledgement and verify actual startup:
  screen-proportional centered bounds within the work area and expansion Running.

See the [startup acceptance record](docs/research/startup-defaults-acceptance.md).

## 17. Settings name and contextual expansion controls

- [x] Move full runtime expansion controls into Settings → Expansion.
- [x] Rename visible Preferences labels to Settings in all three locales.
- [x] Show a persistent paused/stopped warning with Turn on expansion across pages.
- [x] Share pending/error ownership and preserve draft, eligibility and fault guards.
- [x] Complete independent review/fixes, 171 frontend/197 Rust tests, static/tooling
  checks and production light/dark/compact/RTL rendering checks.
- [x] Rebuild/reopen after save-and-Quit acknowledgement; confirm visible Settings,
  removal of the library expansion bar and correct native window bounds.

See the [expansion controls acceptance record](docs/research/expansion-controls-acceptance.md).

## Future decisions and execution work

- [ ] Check desired domains/app-store names and brand clearance before publishing.
- [ ] Review the feature/business/phase proposals and decide the initial audience, license, supported Windows versions, and app matrix.
- [x] Approve the detailed P0 probe/spec before implementation.
- [ ] Run P0 and measure feasibility; revise proposed targets using evidence.
- [ ] Conduct interviews, beta cohorts, willingness-to-pay tests, and real cost estimates at the stated gates.
- [ ] Produce implementation plans for later reviewed phases after P0 qualification.
