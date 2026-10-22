# Texpio phased PRD summaries

Planning proposal · 23 October 2026 · Review pending

## Brief and sequence

Texpio is the selected name for an open-source, local-first text expander with optional paid managed sync later. The user confirmed **Windows first, then macOS** and the official Tauri starter with React/TypeScript and shadcn/ui; a native expansion engine still needs validation. See the [selected stack](tech-stack.md) and [foundation design](superpowers/specs/2026-10-23-desktop-foundation-design.md). These are product requirements summaries, not an implementation plan or evidence of a working product.

Use the [feature plan](feature-plan.md) for interaction and quality definitions and the [business plan](business-plan.md) for commercial hypotheses. Proposed audiences are individual knowledge workers, developers, and support professionals. Their needs and willingness to pay remain unvalidated.

| Phase | Outcome | Decision before proceeding |
| --- | --- | --- |
| P0 | Evidence that native Windows expansion is viable | Select an engine approach and a defensible support contract |
| P1 | A useful offline Windows application | Pass safety, durability, accessibility, packaging, and declared compatibility gates |
| P2 | Better everyday workflows and a separately validated macOS beta | Preserve Windows quality; certify macOS independently |
| P3 | Optional paid personal sync | Demonstrate demand and safe operation, recovery, and economics |
| P4 | Conditional team libraries | Establish organizational demand and a workable ownership/access model |

Phases are gated by evidence, not calendar dates. Business-plan 30/60/90-day checkpoints are capacity-dependent research windows. A failed gate can extend or change a phase.

## Requirements that carry across phases

- Local creation, editing, expansion, search, backup, and export remain usable without an account or subscription; capabilities arrive in the sequence below.
- Publish tested applications, controls, OS versions, layouts, and insertion modes. Do not claim universal compatibility. Decline uncertain contexts before deleting or inserting text.
- Keep raw input and snippet/clipboard contents out of diagnostics and analytics. Use only a bounded transient matching buffer; reset it when context becomes uncertain.
- Preserve acknowledged local saves, user copies, and concurrent edits. Recovery behavior is a release requirement, not a later support workaround.
- Treat all numerical gates as proposed targets until measured on a documented reference setup. No benchmark, interview, cohort, or payment experiment has yet been run.

## P0 — Native expansion feasibility

**Problem and user:** the entire product depends on replacing an abbreviation safely in another app. The immediate audience is the maintainer evaluating engine choices, not end users seeking a production release.

**Scope:** compare a small native Rust engine with viable existing-engine integration; document dependency/license implications. Probe Tauri tray/picker integration, Windows observation, literal recognition, trigger deletion, Unicode output, focus changes, pause/exclusions, and clipboard-assisted versus direct insertion. Use a small test library and report failures rather than building the full editor.

**Exclusions:** polished management UI, paid service, macOS delivery, broad IME support, rich text, images, scripts, and a general plugin system. Picker-only insertion does not demonstrate the automatic-expansion requirement.

**Dependencies and risks:** access to representative Windows apps and keyboard layouts; reproducible fixtures; native API evaluation. Elevated targets, uncertain focus, stateful input, clipboard ownership, and hook lifecycle are explicit risks. A declaration of unsupported behavior must precede input modification.

**Acceptance evidence:** a written probe report with tested builds/settings, strategy comparisons, reproducible failure cases, and at least 300 eligible expansion attempts distributed across three reference applications. Include separate negative tests for focus races, modifiers, pause/exclusion, and unsupported contexts. Require zero wrong-target insertions or unrelated deletion in all probe cases. Measure recognition/insertion timing and resource use without presenting this small sample as release certification.

**Release gate:** choose an engine approach, initial insertion strategies, and a support matrix with known limits. If reliable replacement cannot be established, revise the architecture or scope explicitly; do not relabel a picker prototype as the completed product. A detailed P0 probe/spec and its implementation review are the next development artifacts after review of these plans.

## P1 — Windows offline MVP

**Problem and user:** an individual Windows user needs to save repeated text visually and reuse it without maintaining configuration files or buying a subscription.

**Scope:** visual create/edit/duplicate/delete, folders, enabled state, collision checks, literal case-sensitive prefixed triggers, title/trigger search and keyboard picker. Space activates expansion; preserve the trailing space by default with an explicit removal setting. Include minimal declarative date/time, explicit clipboard-text, and cursor tokens, plus preview/validation. Include tray pause, exclusions, health/status, safe context reset, guarded reversal, native JSON import/export, local snapshots/trash, and tested restore.

**Exclusions:** cloud accounts/sync, Tab/Enter activation, rich text/images, arbitrary scripts, nested templates, mobile, browser extensions, teams, and automatic administrator elevation. Unicode bodies are supported only where tested; composition-based trigger recognition is not implied.

**Dependencies and risks:** P0 decisions; a versioned transactional local library and interchange format; a reviewed Windows packaging/update approach. Picker focus restoration, undo, template output, partial insertion, and password/control detection require explicit supported behavior. A failed or uncertain insertion must not trigger blind recovery backspaces.

**Acceptance evidence:** the feature plan's proposed correctness campaign of at least 10,000 eligible attempts per supported insertion strategy across the declared matrix, with at least 99.9% exact completion and zero wrong-target insertion or unrelated deletion. Count eligible safe refusals as unsuccessful completions and report them separately from wrong output; test negative/race cases separately. Run the specified latency/resource measurements, 100 forced-termination/restart durability cycles, import/export/restore checks, offline operation, keyboard/screen-reader/high-contrast/200% scaling checks, and install/update/uninstall scenarios. These are future gates, not completed tests.

**Release gate:** no known data-loss or target-safety defect, reproducible package, recovery instructions, truthful compatibility notes, and capacity to triage a beta. In a voluntary cohort of at least 20 installers, the business hypothesis is 60% creating/importing three snippets and achieving ten successful expansions within seven days. Diagnose poor activation before increasing scope.

## P2 — Workflow depth and macOS beta

**Problem and user:** retained Windows users need less friction, while macOS users need a native integration that respects their OS permission and input behavior.

**Scope:** improve search/editing, bulk operations, migration reports for selected external formats, simple fill-ins, and per-app troubleshooting based on observed demand. Evaluate tags/favorites/body search before committing to all three. Investigate Tab/Enter activation only with proof it cannot accidentally navigate or submit. Build and independently test the macOS adapter, packaged-app permission onboarding/revocation, secure-entry behavior, lifecycle, insertion, and clipboard handling.

**Exclusions:** guaranteed format-perfect migration, untested rich-content expansion, macOS general availability by a date, Linux parity, cloud operations, and team administration. Rich text/images require a separate compatibility decision if research justifies them.

**Dependencies and risks:** stable P1 semantics; Mac test hardware, signing/notarization access, and platform testing capacity. Reusing template semantics does not certify another OS adapter. Unsupported imported constructs must be reported without executing them or silently changing meaning.

**Acceptance evidence:** migration fixture/round-trip and conversion-loss reports; fill-in cancel/focus tests; continued Windows quality gates. macOS beta must independently satisfy the applicable safety, correctness, privacy, durability, accessibility, packaging, and lifecycle gates for its declared matrix. Track retention through consent-based counters or voluntary reports, never content collection. The business target is 40% day-30 retention among activated users, defined as expansion on three days during days 24–30.

**Release gate:** publish macOS beta only after its own evidence is available; broader release is a separate decision. If retention is below 25% across two cohorts, reassess audience/core experience. Multi-device interviews and priced-offer research can run here without building a sync server.

## P3 — Optional paid personal sync

**Problem and user:** a retained individual using multiple supported devices wants consistent snippets and recovery without operating a sync setup.

**Scope:** one personal library/account model, authenticated device enrollment/revocation, end-to-end encrypted content, offline queue/retry, record revisions, conflicts preserving both edits, deletion/restoration rules, history, understandable status, and billing/cancellation/export. Account recovery and content-key recovery must be distinct. Review the protocol, key lifecycle, metadata exposure, authenticated record identity, backups, and purge policy before making security claims.

The business hypothesis is $24/year or $3/month. Its device/storage/history limits are experiments, not committed requirements. Validate the interaction between a history promise and a storage cap; at capacity, stop new uploads with clear status while keeping local data and promised history. Local editing/expansion continue during network failure, quota exhaustion, and subscription expiry.

**Exclusions:** team sharing, enterprise controls, arbitrary attachments, an always-available SLA, and an operator promise to recover a lost encryption key. Syncing a live local database through a shared folder is not the proposed protocol.

**Dependencies and risks:** multi-device demand and willingness to pay, detailed sync/security design, verification of billing-country/provider support in the user's existing Laravel/PostgreSQL service, verified costs, staffing, and independently reviewed recovery/data-integrity behavior. Reuse that separate service for accounts and billing; no server is built in this desktop repository now. At least two certified devices are required; Windows-to-Windows can be evaluated without claiming macOS support before its gate.

**Acceptance evidence:** fault tests for concurrent edits, edit/delete conflicts, duplicate/reordered requests, interrupted uploads, clock skew, long-offline devices, version migration, key loss, revocation, restore, cancellation, quota, and account deletion. No acknowledged edit may vanish silently. Verify both intended server-visible metadata and absence of plaintext content from service logs/storage. Paid beta requires passed integrity/security gates and a supported operating process before collecting payment.

**Release gate:** the proposed demand signal is five of eight multi-device interviewees with a recurring unresolved problem and interest in the priced pilot, followed by learning from ten paid pilot users. These small samples are directional. Establish actual cost/support measurements and recovery results; demand never overrides a corruption or key-management failure. Refresh pricing/terms before sale.

## P4 — Conditional team libraries

**Problem and user:** an organization with purchase authority needs maintained shared replies and controlled distribution beyond personal sync.

**Scope candidate:** library owner/editor/member roles, personal versus shared namespaces, trigger-conflict rules, invitations, organization billing, ownership transfer, offboarding, and recoverable history. Define how shared encryption keys and membership changes work. Revocation can restrict future access; it cannot erase plaintext already copied onto another person's device.

**Exclusions:** assumed demand, general enterprise compliance claims, SSO/SCIM, custom contracts, audit-retention guarantees, and scripting/automation administration unless separate discovery justifies them.

**Dependencies and risks:** stable personal sync, evidence from at least three qualified organizations requesting a paid pilot, clear purchasing authority, reviewed access/key model, and sustainable support capacity. Personal/shared data ownership and cancellation consequences must be unambiguous.

**Acceptance evidence:** role/access tests, invitation/revocation and offline-member behavior, trigger collisions, owner departure, billing state transitions, recovery, and cross-organization isolation. Specify price, support, ownership, and deletion terms from this research before implementation rather than borrowing personal-tier terms.

**Release gate:** proceed only when organizations accept a concrete priced offer and access/data-integrity tests pass. If demand does not support this scope, remain an excellent personal app and service; P4 is an option, not an obligation.

## Review and execution boundary

Planning deliverables are the feature plan, business plan, and these summaries. The selected Texpio GitHub organization is created and verified. Customer research, license selection, detailed technical specifications, prototype implementation, and release verification are future work tracked in [TASKS.md](../TASKS.md). Review the proposed product/engine approach before turning P0 into an implementation plan.
