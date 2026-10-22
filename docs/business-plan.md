# Texpio business plan

Planning draft, 23 October 2026. **Texpio**, **Windows first, then macOS**, and the official Tauri starter are user decisions. Linux remains conditional. The local-first open-source desktop product and optional paid hosted sync are the brief; native-engine feasibility still needs testing. Future accounts and billing will reuse the user's existing Laravel/PostgreSQL project; see the [technology stack](tech-stack.md). Audiences, license, prices, limits, targets, and budgets below are recommendations or hypotheses, not approved commitments. No customers, traction, runtime benchmarks, or market-size estimate have been established. This document authorizes no spending, registration, outreach, or license change.

## Positioning and competition

Proposed positioning: **An approachable text expander that keeps your snippet library on your computer, with reliable everyday expansion and optional managed sync.** The potential advantage is a visual workflow, understandable compatibility, easy migration, and dependable recovery. “Free” and “private” alone are weak differentiation because credible alternatives already exist.

Official sources rechecked on 23 October 2026:

| Alternative | Verified reference | Implication to test |
| --- | --- | --- |
| TextExpander | Free permits five active snippets; its FAQ lists Pro at $60/user/year and Business at $120/user/year. [Pricing](https://textexpander.com/pricing) | An unlimited offline library could appeal, but price alone will not displace an established workflow. |
| Espanso | GPL-3, Windows/macOS/Linux, search, forms, scripts, and file configuration; its site says a GUI is planned. [Official site](https://espanso.org/) | Texpio must demonstrate easier setup and maintenance, not claim novel expansion functionality. |
| Espanso sync | Documentation describes cloud-folder/GitHub synchronization and symlinks. [Sync documentation](https://espanso.org/docs/sync/) | Managed sync competes with user-operated solutions as well as paid products. |
| Beeftext | Windows snippet manager under MIT; maintenance mode includes major-issue fixes, with no new features planned. [README](https://github.com/xmichelo/Beeftext/blob/master/README.md) | An active visual alternative may interest users; maintenance mode does not establish abandonment or migration demand. |
| Obsidian | Optional Sync Standard costs $4/user/month billed annually or $5 monthly. [Sync plans](https://obsidian.md/sync) | Optional-service revenue is a useful analogy, not evidence that text-expander users will pay. |

Do not advertise universal application compatibility or quantified time savings before measurement. The central uncertainty is whether a focused visual experience is sufficiently better to sustain use.

## Initial customers and jobs

Start with individual Windows users; add macOS after the Windows product passes its gates.

| Proposed segment | Recurring job | Evidence needed |
| --- | --- | --- |
| Freelancers and independent operators | Reuse replies, introductions, addresses, and routine instructions across apps. | Frequent repetition and frustration with existing setup. |
| Support or operations professionals choosing personal tools | Insert accurate standard responses with occasional edits. | Their employer permits the tool and local storage; personal purchase authority exists. |
| Developers and technical writers | Reuse commands, boilerplate, and documentation fragments. | A visual editor wins despite capable file-based alternatives. |

Prioritize people repeating text daily, willing to install desktop software, and dissatisfied with a current workaround. Interview multi-device users separately: offline adoption does not establish sync demand. Regulated deployments, centrally managed fleets, and enterprise procurement are outside the initial commercial scope. Individual professionals are a research segment, not a compliance claim.

## Free promise, ownership, and governance

Propose a permanent free desktop core: unlimited local snippets, expansion, visual editing, search, organization, pause/exclusions, local backup, and documented import/export, delivered in the sequence in the [feature plan](feature-plan.md). No account or recurring payment should be required for local operation. Paid-service cancellation must leave local libraries usable. Retain the ability to export without a subscription and avoid paid-only file formats.

The business sells operated convenience: device enrollment, encrypted synchronization, conflict handling, version recovery, monitoring, and service support. A functioning free product is the distribution engine and trust commitment, not a trial that expires. Donations or sponsorship can supplement funding but are not forecast revenue.

License decision remains pending; no license file is changed:

| Candidate | Relevant terms and business tradeoff |
| --- | --- |
| MIT | Broad use, modification, distribution, and sale permissions with notice retention. Simple adoption; downstream proprietary derivatives remain possible. [License text](https://opensource.org/license/mit) |
| Apache-2.0 | Permissive copyright grant, explicit contributor patent grant and termination provisions, and redistribution/notice conditions. More notice administration; compatible with a service-led strategy. [License text](https://www.apache.org/licenses/LICENSE-2.0) |
| GPL-3.0 | Copyleft conditions attach to distribution of covered works, including corresponding-source obligations; network interaction alone is not conveying. Stronger reciprocity, with more integration constraints to evaluate. [License text](https://opensource.org/license/gpl-3.0) |

**Recommended candidate: Apache-2.0 for original desktop code**, pending owner choice and dependency/reuse review. This is a product recommendation, not a legal conclusion. Espanso code reuse requires a separate license assessment and may change the choice. Decide hosted-server licensing separately; keep the desktop client and documented data format open.

Propose public issue triage, contributor guidelines, a maintainer decision log, dependency notices, and clear release/security ownership. Use a lightweight contribution process after review; do not promise unlimited maintainer support. Reserve project identity and signing access to accountable maintainers; clarify trademarks separately from code permissions.

## Proposed paid offer and terms

Recommend one personal tier, **$24/year or $3/month**, USD, annual-first. The annual offer equals $2/month and reduces per-payment overhead. This is an opening experiment, not a published price. Show applicable taxes and the full renewal amount before purchase; establish billing-country eligibility and provider costs before launch.

Provisional scope: five devices, text-only snippets, 50 MiB live data, and 90-day version history within a 250 MiB total account allowance. Validate these limits against real libraries and history growth. At capacity, pause new uploads with a clear status and preserve local originals and promised retained history; provide export and explicit cleanup choices. Attachments, team sharing, SSO, audit guarantees, and service-level agreements are excluded.

The proposed service requires end-to-end encrypted content, authenticated device enrollment/revocation, observable sync state, conflict copies or explicit resolution, and tested recovery. Account recovery must not imply that the operator can recover a lost content key. Metadata exposure, local storage protection, key recovery, deletion, and backup retention need a reviewed design before any security promise or payment.

Proposed support: community/documentation for the free app; email support for billing and sync, targeting an initial response within two business days once staffing supports it. This is not a guaranteed SLA. Offer a proposed 14-day refund window, subject to finalized terms and applicable requirements.

Cancellation should stop renewal while service continues through the paid term. At expiry, local operation and export continue; propose a clearly disclosed 30-day remote retrieval period, then deletion of active remote copies, with backup expiry documented separately. Account deletion needs confirmation and a stated purge schedule. Finalize these policies before sale, not after the first cancellation.

## Acquisition and demand experiments

Future acquisition should begin with useful demonstrations: creating three reusable replies, migrating a small library, and recovering an accidental edit. Publish accurate compatibility notes, migration guides, troubleshooting, and an accessible project README. Invite voluntary beta participation through relevant communities under their rules; consider creator tutorials and searchable workflow guides after retention evidence. No outbound, promotion, or contact has been performed under this plan.

Before building hosted infrastructure, propose 12–15 interviews across the segments, including at least eight regular multi-device users. Observe their current tools and actual sync failures. Present the $24 annual offer with explicit limits; compare a separate $36 annual offer if the sample supports a useful comparison. Record objections and intent separately from payment behavior. Small samples are directional, not statistically conclusive.

After explicit authorization and a working safe service, a paid pilot with ten users is stronger evidence than a waitlist. Seek repeated second-device use and renewal intent. Until then, use nonbinding offers and observed workflows; do not count hypothetical signups as sales. Require at least five of eight multi-device interviewees to describe a recurring unresolved problem and willingness to join the priced pilot before funding P3. If they prefer existing free sync, retain an excellent offline product and revisit the revenue model.

## Illustrative unit economics

All figures below are **assumptions**, not supplier quotes or revenue forecasts. Use steady-state full-year subscribers, with no acquisition spending. Annual payment fee assumption: 3% + $0.30 per charge. Per subscriber/year: variable infrastructure $1.80, paid-service support six minutes at $20/hour = $2, and refund/payment-loss reserve 2% of gross receipts. Fixed annual allowance: $1,200 for base hosting, monitoring, release distribution, signing/notarization, CI, and administration. Verify actual quotes and capacity before relying on it.

`Annual contribution per subscriber = price − payment fees − variable infrastructure − support − reserve`

At $24/year: `$24 − $1.02 − $1.80 − $2 − $0.48 = $18.70` (77.9%). At $3/month for twelve paid months: `$36 − $4.68 − $1.80 − $2 − $0.72 = $26.80`; this comparison assumes no monthly churn and twelve payment fees, not identical retention.

| Annual subscribers | Gross receipts | Payment fees | Infra | Support | Reserve | Contribution | After $1,200 fixed costs |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | $2,400 | $102 | $180 | $200 | $48 | $1,870 | $670 |
| 500 | $12,000 | $510 | $900 | $1,000 | $240 | $9,350 | $8,150 |
| 1,000 | $24,000 | $1,020 | $1,800 | $2,000 | $480 | $18,700 | $17,500 |

Cash break-even under these assumptions is `ceil($1,200 / $18.70) = 65` annual subscribers. If support rises to 30 minutes/year at $20/hour, contribution falls to $10.70 and break-even becomes 113. Every additional $1,000 fixed cost requires about 54 base-case annual subscribers.

These results exclude founder development time, free-user support, taxes, acquisition, substantial legal/security work, and hardware purchases beyond the placeholder allowance; they are not founder-income profitability. Obtain actual processor/international fees, refund costs, storage/history/egress, security review, signing, Mac testing hardware, and ongoing support estimates. Windows signing still may show reputation warnings; macOS distribution introduces signing/notarization work. [Tauri Windows guide](https://v2.tauri.app/distribute/sign/windows/), [macOS guide](https://v2.tauri.app/distribute/sign/macos/).

## Evidence milestones and stop gates

The following 30/60/90-day windows are tentative evidence checkpoints from an approved project start, contingent on team capacity and recruitment. They are not shipping commitments.

| Window / phase | Proposed evidence and decision |
| --- | --- |
| Days 1–30 / P0 | Complete native-engine feasibility and interviews. Stop or re-scope if safe, reliable insertion cannot meet the technical gates; reconsider the stack rather than funding sync. |
| Days 31–60 / P1 | Windows offline MVP only after P0 passes. In an opt-in cohort of at least 20 installers, target 60% creating/importing three snippets and making ten successful expansions in seven days. Investigate failed activation before adding features. |
| Days 61–90 / P2 | Improve workflows and beta quality; begin macOS only with capacity and platform evidence. Target 40% day-30 retention among activated users, defined as expansion on three days during days 24–30. If below 25% across two cohorts, reassess audience and core experience. |
| Later / P3 | Optional personal sync only after demand, security, integrity, and operating-cost gates pass. Hypothesize 10% trial-to-paid conversion among eligible multi-device trial users and 80% paid retention at 90 days; ten pilot users are learning evidence, not a reliable conversion estimate. |
| Conditional / P4 | Evaluate teams only after sustained requests, purchase authority, and willingness to pay are demonstrated. Make no enterprise roadmap or compliance commitments now. |

Measure cohorts with consent-based local counters or voluntary reports; never collect typed content or snippet bodies for funnel analytics. Technical success also requires no known data-loss defects and the [phased PRD](phased-prds.md) acceptance gates. Low conversion should trigger interviews and cost review; unresolved corruption or recovery failures block sync launch regardless of demand. License incompatibility, excessive support load, and insufficient maintenance capacity require re-scoping before further commitments.
