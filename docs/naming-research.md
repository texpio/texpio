# Naming research

Research date: 23 October 2026 (Asia/Kolkata)

Status: **Texpio selected and its GitHub organization created by the user**, verified on 23 October 2026.

## Brief

A short, pronounceable brand for an open-source, local-first desktop text expander, with optional paid sync later. The brand should work for both the app and a future service. Tauri is a proposed implementation choice, not part of the naming requirement.

## Selected name

**Texpio**, pronounced **teks-PEE-oh**.

The user selected it for its connection to text expansion. Use **Texpio** for the desktop app and **Texpio Sync** as the proposed optional service name. The GitHub organization is created; no deployed application or sync service is implied.

Initial checks after selection found no public account at `texpio`, no public repository at `nikitech0103/texpio`, and zero public repository-name search results for `texpio`. Both API identity/repository lookups returned HTTP 404. GitHub's public signup page returned HTTP 403, so the assistant could not test username registration eligibility at that stage.

The user then confirmed that GitHub showed the name as available and reported creating [github.com/texpio](https://github.com/texpio). A fresh [public API lookup](https://api.github.com/users/texpio) returned **HTTP 200**, login **texpio**, type **Organization**, creation time **2026-10-22T21:00:58Z** (23 October 2026 in Asia/Kolkata), and **zero public repositories** at that initial verification. This confirms the created organization and supersedes the initial availability uncertainty. The user subsequently created the canonical [texpio/texpio repository](https://github.com/texpio/texpio). Domain and trademark checks remain separate launch followups.

## Initial shortlist recommendation

The initial recommendation was **Frazlo**, pronounced **FRAYZ-lo**. The user's subsequent choice of Texpio supersedes that recommendation.

The intended association is phrase + flow. It is six letters, has a connection to reusable text without describing every feature, and supports names such as Frazlo Sync. These are branding judgments, not researched etymology.

Possible tagline: **Your words, a shortcut away.**

## Shortlist

| Name | Pronunciation | Why it fits | Tradeoff |
| --- | --- | --- | --- |
| **Frazlo** | FRAYZ-lo | Compact; suggests phrases and fluid typing; initial recommendation | Some unrelated online uses; spelling may need to be stated aloud |
| **Tersivo** | TUR-see-vo | Polished, distinctive; suggests terse input becoming useful text | More abstract and one syllable longer |
| **Texpio — selected** | teks-PEE-oh | Compact; strongest text-expansion association | Tex can suggest TeX or textiles; pronunciation needs introduction |
| **Abblio** | AB-lee-oh | Friendly; subtle abbreviation association | Double b can be missed when dictated; similar-sounding brands need a closer check |

## GitHub verification

The initial screening used two separate checks before the organization was created:

1. Public repository-name searches, including combined and individual candidate queries. No exact-name public repository was returned for these four candidates. Repository names are scoped to their owner; this search screens collisions rather than reserving a globally unique repository name.
2. Live unauthenticated requests to GitHub's public account endpoint. The exact handles below each returned **HTTP 404 Not Found**.

| Proposed handle | Direct evidence | Response before creation |
| --- | --- | --- |
| `frazlo` | [GitHub account API](https://api.github.com/users/frazlo) | 404 |
| `tersivo` | [GitHub account API](https://api.github.com/users/tersivo) | 404 |
| `texpio` | [GitHub account API](https://api.github.com/users/texpio) | 404 |
| `abblio` | [GitHub account API](https://api.github.com/users/abblio) | 404 |

**Historical interpretation:** these initial results showed apparently unclaimed handles, not confirmed registration eligibility. A browser check at [GitHub signup](https://github.com/signup) encountered a device/network access restriction before username validation was available. The user subsequently created Texpio, and the HTTP 200 organization lookup above resolves that uncertainty for the selected name. No organization or repository was created by the assistant during naming research.

The connected GitHub login was identified. A fresh public lookup for [nikitech0103/texpio](https://api.github.com/repos/nikitech0103/texpio) returned 404, which does not exclude a private repository outside the connection's visibility. This research focused on public identity and project collisions; no registration or repository creation was attempted.

## Product and web screening

Exact-name searches and searches with app/software/text-expansion terms found no obvious exact-name competing software for the shortlisted names. Search coverage cannot establish exclusivity.

Frazlo has unrelated uses, including a [gaming character](https://www.wowprogress.com/character/us/undermine/Frazlo) and a [lighting vendor label](https://lumeerrs.shop/products/plafonnier-moderne-design-feuille-lotus-en-bois-avec-lumiere-led-9863251034458). These did not appear to be text-expansion software.

Several candidates were removed or downgraded after research:

| Candidate | Reason | Evidence |
| --- | --- | --- |
| Snipora | Existing local-first open-source snippet manager | [Official project site](https://snipora.github.io/) |
| Brevlo | Existing software product | [Official site](https://www.brevlo.app/) |
| Scriblo | Existing writing software | [Product dashboard](https://scriblo.co/dashboard) |
| Tersio | Existing software name | [Public repository](https://github.com/KurutoDenzeru/tersio) |
| Phrivo | Existing software/web business | [Official site](https://www.phrivo.com/) |
| Keebli | Exact GitHub handle occupied | [GitHub profile](https://github.com/keebli) |
| Frazlet | Pronunciation is close to Phraselet, a phrase-related extension | [Chrome Web Store listing](https://chromewebstore.google.com/detail/phraselet/hpkpdkhoflhmabngfmaboicnlonnmnkp) |
| Phravo | Nearby Phravora software name makes it less distinctive | [Project README](https://github.com/mahbod5959/phravora/blob/main/README.md) |

Domain availability, app-store names, and formal trademark clearance were not established. Texpio is selected and its GitHub organization is created and verified; those other launch checks remain separate.

## References for later planning

- [TextExpander pricing](https://textexpander.com/pricing): the current Free plan lists five active snippets; paid plans remove that limit. A useful planning hypothesis is to differentiate through a capable offline open-source core, then validate whether users want managed sync.
- [Obsidian pricing](https://obsidian.md/pricing): the app is free and optional services include paid Sync and Publish. This is a revenue-model reference; our open-source licensing decision is separate.
