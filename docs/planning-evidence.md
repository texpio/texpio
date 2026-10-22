# Preparatory evidence for product planning

Checked: 23 October 2026 (Asia/Kolkata).

This is a source notebook for Texpio's feature plan, business plan, and phased PRD summaries. The user selected Texpio and confirmed Windows first, then macOS. The user also created the [Texpio GitHub organization](https://github.com/texpio), verified through the public API as an Organization on 23 October 2026. These notes do not approve product scope or establish runtime feasibility.

## Desktop framework

Tauri 2 provides a system tray, a global-shortcut plugin, and capabilities that constrain frontend access to native functionality. Those are useful building blocks for a snippet editor and tray utility. Sources: [System Tray](https://v2.tauri.app/learn/system-tray/), [Global Shortcut](https://v2.tauri.app/plugin/global-shortcut/), [Capabilities](https://v2.tauri.app/security/capabilities/).

The shortcut plugin registers key combinations. It does not document an ordinary typed-character stream or abbreviation replacement. **Planning inference:** keep the native expansion engine as a separate feasibility question from the Tauri shell, which the user has now selected; see the [technology stack](tech-stack.md). A framework choice alone does not prove system-wide expansion works.

No prototype, installation, keyboard hook, or injection test has been run. Compatibility, latency, memory use, keyboard layouts, Unicode/IME behavior, clipboard preservation, permission handling, and recovery after focus changes remain unmeasured.

### OS-specific evidence

| Area | Documented behavior | Implication to validate |
| --- | --- | --- |
| Windows insertion | `SendInput` can inject only into applications at an equal or lower integrity level; pressed modifiers can affect synthesized input. [Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) | Do not promise that a normally running app expands into elevated editors. Test focus changes, modifiers, and insertion failures. |
| Windows observation | Low-level keyboard hooks need a message loop and prompt callbacks; a timed-out hook can be silently removed. [Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc) | Keep observation independent of editor work; validate health detection and recovery. |
| macOS | Apple exposes Input Monitoring controls and Core Graphics event APIs. [Input Monitoring](https://support.apple.com/guide/mac-help/control-access-to-input-monitoring-on-mac-mchl4cedafb6/mac), [CGEvent](https://developer.apple.com/documentation/coregraphics/cgevent) | Test packaged-app permissions, grant/revoke behavior, and secure-entry behavior for the selected approach. |
| Linux | The first-party `global-hotkey` README lists Linux support as X11 only. [README](https://github.com/tauri-apps/global-hotkey/blob/dev/README.md) | Tauri's general Linux support is insufficient evidence for a Wayland text-expansion promise. Evaluate the selected compositor and input approach separately. |

Physical key observation and final composed text are different layers. Microsoft documents dead-key/stateful translation and IME composition messages. Recognition must therefore be tested separately from Unicode output. Sources: [ToUnicodeEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex), [IME composition](https://learn.microsoft.com/en-us/windows/win32/intl/wm-ime-composition).

Tauri's [clipboard plugin](https://v2.tauri.app/plugin/clipboard/) supplies clipboard access. Clipboard preservation, native paste synthesis, concurrent copies, and app-specific insertion behavior remain application-level questions, not established plugin guarantees.

## Existing alternatives

| Product | Verified facts relevant to planning | Primary source |
| --- | --- | --- |
| TextExpander | Free includes five active snippets and device sync. Its pricing FAQ lists Pro at USD 60/year or USD 6/month; Business at USD 120/year or USD 12/month. Annual figures are per user. | [Pricing](https://textexpander.com/pricing) |
| Espanso | Open-source GPL-3, Rust-based, Windows/macOS/Linux. Offers search, dates, forms, scripts, and packages. The site describes file-based configuration and says a GUI is planned. | [Official site](https://espanso.org/) |
| Espanso sync | Documents third-party cloud-folder or GitHub synchronization, an established symlink approach, and experimental custom-directory options. Managed sync would compete with existing user-operated alternatives. | [Synchronization documentation](https://espanso.org/docs/sync/) |
| Beeftext | Windows snippet manager. Its README explicitly places it in maintenance mode, with major-issue fixes but no new features, and identifies an MIT license. Maintenance mode does not mean abandoned. | [Project README](https://github.com/xmichelo/Beeftext/blob/master/README.md) |
| Obsidian | Free app with optional services. Sync Standard is USD 4/user/month billed annually or USD 5 monthly; Plus is USD 8 annually or USD 10 monthly. These are reference prices, not proof of willingness to pay for a text expander. | [Sync plans](https://obsidian.md/sync) |

Prices and product status need rechecking when the business plan is written or published. No market size, adoption, conversion, revenue, or customer-demand figures have been established.

## Sync and licensing considerations

Obsidian documents end-to-end encryption as the default for new remote vaults, distinguishes remote encryption from local storage, and explains the consequences of losing an encryption password. **Planning inference:** any proposed encrypted service needs an explicit device-enrollment and recovery model; account recovery and content-key recovery are different problems. Source: [Sync security and privacy](https://obsidian.md/help/sync/security).

Its conflict documentation offers automatic merging or a separate conflict copy, and describes different handling for different file types. **Planning inference:** a text-expander sync design needs a documented policy for concurrent edits, deletion, restoration, and version history. Do not assume overwriting the latest library file is sufficient. Source: [Conflict handling](https://obsidian.md/help/sync/troubleshoot).

Obsidian's optional-service revenue model is the analogy; its license reserves rights to its app code. Our open-source license remains a separate decision. Espanso's repository contains GPL version 3. Any reuse option needs a dependency and licensing review before selection. Sources: [Obsidian license overview](https://obsidian.md/license), [Espanso license](https://github.com/espanso/espanso/blob/dev/LICENSE).

## Hypotheses to test during product validation

1. A visual editor with a capable, unlimited offline core could appeal to people who find file configuration inconvenient or limited free plans restrictive.
2. Reliable expansion, understandable app compatibility, easy migration, and fast troubleshooting may matter more than a long list of advanced features.
3. Multi-device users may pay for convenient managed sync with history and recovery. Single-device adoption does not establish demand for that service.
4. Small text payloads do not establish service profitability. Measure billing, support, monitoring, backup/history, authentication, security maintenance, and infrastructure costs before setting prices or projecting margins.

These are research directions, not validated positioning. Windows first, then macOS is user-confirmed. Initial audience, license, paid-service scope, pricing, and launch criteria are proposals to review in the plans.

## Evidence status

Official documentation and public repository text were inspected. The notebook contains no measured benchmark, customer interview, paid-demand test, or runtime compatibility result. The product plans use this evidence while keeping their assumptions and future validation gates explicit.
