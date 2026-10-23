# Windows expansion probe evidence

Status on 24 October 2026: **implementation in progress; no input strategy qualified**. P0 resumed after the template/Bun migration. This report distinguishes code checks from native acceptance and physical typing evidence. The preview starts Stopped; no live hook has been armed during these checks.

## Environment recorded during preflight

| Item                         | Observed version                  |
| ---------------------------- | --------------------------------- |
| OS API description           | Microsoft Windows 10.0.26300, X64 |
| Notepad package              | 11.2607.14.0, X64                 |
| Visual Studio Code candidate | 1.143.0, x64                      |
| Chrome candidate             | 155.0.8059.85, x64                |
| Node / pnpm                  | 24.18.0 / 12.15.0                 |
| Rust                         | 1.99.1                            |
| Windows bindings             | 0.62.2                            |

These app versions were inventoried, not qualified for expansion. The OS description is the literal measured API result; kernel version 10 alone does not identify the Windows product edition.

The table is the original preflight inventory. Current project tooling after the approved migration is Bun 1.4.4 with Node 24.18.0 and Rust 1.99.1; pnpm is historical. The migrated shell's basic opening and both launchers were user-confirmed, but its detailed native lifecycle/settings/platform acceptance remains open in [template parity](tauri-template-parity.md).

## Foundation acceptance

Observed: native metadata reports Connected/version/platform; System/Light/Dark theme switching; close hides a still-running process; launching a second actual process exits that process and restores the original window with one process remaining.

The user subsequently verified close → tray Open → tray Quit manually. A CLI process check found zero Texpio processes before the requested relaunch, and the preview was relaunched successfully. The minimized-window variant remains unverified. Computer Use was stopped after the earlier stalled closure and has not been restarted. Repository rules require brief sessions and immediate cleanup.

The updated native preview was built successfully and the user confirmed the probe panel, initial Stopped status and fixed snippets. A screenshot also shows the rendered snippets and controls. This confirms the initial screen only; running/paused tray parity and native insertion remain unverified.

## Read-only context implementation

Rust owns worker-local UIA objects in an MTA apartment. Captured tokens retain the original native foreground/focus/process/thread stamp, logical focused control, process handle, and worker session identity. Tokens and captured text do not serialize to React.

Before adjacent text is read, the adapter requires explicit `VT_BOOL` non-password and enabled properties, a qualified candidate control, a single collapsed selection and explicit non-readonly text attribute. Unsupported properties and provider defaults refuse. It checks standard-user medium integrity, the explicit US/India HKL allowlist and all 28 recognition-key mappings, modifiers, positive UIA composition evidence, logical control identity, native focus, finite provider timeouts and an overall preparation deadline. Only bounded adjacent text may establish an exact trigger plus Space suffix and whitespace/document-start boundary. Revalidation repeats the suffix, caret and composition checks.

Post-insertion observation is implemented for later dispatch integration: retain the untouched left boundary, reconstruct a bounded range to the fresh caret, and compare exact Unicode output. A proven document start uses a fresh document range. UIA Character units are not UTF-16 offsets, so verification never walks backward by body UTF-16 length. Line endings are compared exactly; no application-specific conversion has been qualified.

Candidate selectors are deliberately strict and **not yet measured support**:

| Candidate | Intended scope                                                                                                     | Native qualification         |
| --------- | ------------------------------------------------------------------------------------------------------------------ | ---------------------------- |
| Notepad   | Disposable `texpio-p0-notepad.txt`; Win32 `RichEditD2DPT` document named Text editor                               | Read-only capture/prepare/revalidate passed 24 October; insertion unqualified |
| VS Code   | Disposable `texpio-p0-vscode.txt` plaintext editor; excludes Find and other controls                               | Pending actual UIA discovery |
| Chrome    | Own loopback page; only labeled positive textarea/input; ancestry must prove document URL and foreground ownership | Pending actual UIA discovery |

If these installed builds expose different identities or insufficient password/caret/range/origin evidence, they remain refused. Do not broaden admission to executable-name-only matching to obtain a passing result.

## Repeatable read-only check

Build first with `cargo build --locked --manifest-path src-tauri/Cargo.toml --example probe_context`. Run the resulting `src-tauri/target/debug/examples/probe_context.exe` with `notepad`, `vscode`, or `chrome`. It waits five seconds for the user to focus the synthetic reference document and checks once. It never installs hooks, sends keys, changes clipboard data, or prints captured text/handles/paths. Keep the caret after `;hello` and exactly one Space, at document start or following whitespace.

The helper returns a content-free refusal reason or read-only preparation acceptance. Acceptance alone is not an input-strategy qualification. Record each target's actual UIA capabilities/refusals here before enabling mutation. Password-field values must never be read.

The first user-assisted Notepad check completed and returned `UnsupportedLayout`, with no mutation. The active HKL observed afterward was `0x40094009`, corresponding to English (India), and configured input methods were `4009:00004009` and `0409:00004009`. Both use the India keyboard layout. This was an expected refusal of the original exact-US profile, not a successful UIA qualification or an insertion failure.

The user then approved qualifying English (India) alongside US. The adapter checks an explicit HKL allowlist plus state-preserving native translation of all 28 recognition keys, and includes the layout in target identity. A rebuilt helper's second Notepad check still returned `UnsupportedLayout`; the investigation was paused for the template/Bun migration.

On 24 October, a read-only native diagnostic isolated the current refusal: both installed India HKLs, `0x40094009` and `0x40090409`, map all 28 fixed keys exactly, while `ImmIsIME` returns true for both. Registry metadata identifies `KBDINEN.DLL` and no `Ime File` value. These are measured host observations, not a claim that every India layout behaves this way. No system input preference changed.

`ImmIsIME` classifies an input locale; it does not prove whether the captured control is composing. Microsoft's [IMM threading requirements](https://learn.microsoft.com/en-us/windows/win32/intl/developing-ime-aware-multiple-thread-applications) also prohibit treating a failed foreign-thread input-context query as idle evidence. The adapter therefore replaces the broad IME refusal and foreign-thread IMM context queries with an explicit captured-control [UIA TextEdit](https://learn.microsoft.com/en-us/windows/win32/winauto/textedit-control-pattern) check, after non-password/enabled/profile validation. The control must explicitly support TextEdit and return **S_OK plus a null active-composition range**. Active ranges, unknown success codes, unsupported patterns, provider errors, stale identity and timeouts refuse. A returned range is released without reading composition text. The same evidence is rechecked before dispatch.

The installed bindings' convenience wrapper converts a successful null interface into an error. A small native helper preserves the HRESULT and nullable owned reference separately, following [GetActiveComposition's contract](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtexteditpattern-getactivecomposition). Four new regressions went RED to GREEN for successful-null idle, non-S_OK refusal, active-range refusal/release, and failure with an owned range. Controls without this evidence remain refused.

The third user-assisted Notepad helper run passed on 24 October: native profile, layout mapping and collapsed caret accepted; exact synthetic suffix/boundary accepted; final read-only revalidation accepted. The helper exited successfully within its 10-second process bound and sent no input. This observes positive idle TextEdit evidence for that exact synthetic control. It does not qualify live composition transitions, input dispatch, output verification, other controls, or actual expansion reliability. The Notepad package remains 11.2607.14.0 x64; VS Code 1.143.0 and Chrome 155.0.8059.85 were also re-inventoried, not newly qualified.

The updated read-only helper's SHA256 is `37C0530B41A77857C609F9CAD0D7866520BEC3049A71AEFBA8BFC9761B704694`. The Windows debug application also rebuilt successfully with both frontend entries (optimized release rebuilding was not part of this checkpoint).

The first user-assisted VS Code read-only check returned `UnsupportedProfile`, without mutation. Its disposable profile enables `editor.accessibilitySupport` and disables extensions; this is an explicit test condition, not qualification of the normal editor configuration. Two metadata-diagnostic attempts stopped at their foreground/caption prerequisite before reading UIA properties. No actual VS Code control identity or failed profile predicate was established, and no profile restriction was relaxed. On the same day, the user redirected the work to the [Espanso source/architecture audit](espanso-source-audit.md) before further native probing. The temporary loopback server was stopped; no Computer Use session was started.

For the browser fixture, run `bun run probe:fixture` and open `http://127.0.0.1:4175/` in Chrome. The server binds IPv4 loopback, serves only three known GET assets, and accepts no uploads or request bodies. The page includes synthetic sentinels, positive controls, second focus target, password, readonly, disabled and exploratory contenteditable controls. Use only these synthetic fixtures. Stop the server with Ctrl+C when done.

## Evidence boundaries and remaining work

### Clipboard comparison: disabled

Clipboard mode remains disabled in the selector. Direct native start requests for it return the closed `ClipboardState` refusal before arming the gate or creating any native job. The supported native clipboard format set is **empty**; this implementation never opens, enumerates, reads, publishes, clears or restores the user's clipboard. Unicode does not fall back to clipboard insertion.

Microsoft documents that advertising a format with a NULL handle defers its rendering until data is requested. `EnumClipboardFormats` lists available formats, including synthesized formats, but its documented contract supplies no eager/delayed flag. Calling `GetClipboardData` to discover data can itself trigger `WM_RENDERFORMAT`; consequently a plain-text format ID alone does not prove bounded, eager snapshotting. This is a limitation found in the examined APIs, not a claim that every Windows mechanism is incapable. See [clipboard operations](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-operations), [format enumeration](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumclipboardformats) and [the Microsoft engineering discussion of delayed rendering](https://devblogs.microsoft.com/oldnewthing/20220608-00/?p=106727).

Even a known empty clipboard or a test-owned eager fixture does not establish that an asynchronous paste was committed by the intended control. A later implementation would need exact target-output evidence and an owner/sequence check while holding the clipboard lock before restoring supported snapshots. A newer user's copy must win, with no repair/replay after uncertainty. Sequence numbers can change only when delayed data renders; clipboard retrieval is not proof of text insertion. These requirements follow from [sequence number semantics](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboardsequencenumber), [OpenClipboard](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openclipboard) and [GetClipboardData ownership](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboarddata).

The approved plan explicitly permits keeping this mode disabled when preservation/consumption cannot be qualified. We take that branch: no lease/restore implementation, modeled roundtrip claims, native clipboard trials, timing comparison or strategy-success claim. A refusal regression verifies all three reference profiles stay Stopped with zero counters, a closed gate and no native job. The UI test verifies the option is disabled and its reason is visible. Clipboard history/manager exposure would also need to be explained before any later opt-in publication.

- Unit tests cover bounded matching, rollover/repeat/reset behavior, explicit property semantics, unsafe guard states, exact suffix/boundary rules, two-field identity mismatch and candidate selectors. These are synthetic evidence and do not validate actual UIA providers.
- The fixture-server test covers loopback binding, asset allowlist, traversal rejection, method rejection and response security headers.
- Runtime code now compiles with a dedicated observation/message-loop thread, separate worker-local UIA backend, bounded candidate queue, generation checks, one Unicode batch and exact output observation. Construction and ordinary tests install no hooks and send no input. Native hook teardown, provider shutdown and actual insertion remain unqualified.
- The resumed aggregate Rust suite has 81 passing tests, including the adopted template services and four new composition regressions. Probe coverage includes Stop during blocked preparation, acknowledged in-flight uncertainty, stale/consumed tickets, unexpected injections, explicit key mapping, UTF-16 encoding, zero-submission accounting, closed command serialization, latched teardown failure and disabled clipboard admission. Strict Clippy passes.
- No native provider identity/caret/range campaign has been completed. The preparation deadline and UIA timeouts still need slow-provider and cancellation qualification with the later runtime watchdog.
- Probe commands, a validated React bridge, content-free status polling, maintainer controls and tray state compile. The resumed frontend suite has 118 passing tests, including the adopted shell and previous probe regressions for stale responses after Stop, malformed IPC, browser refusal, pending control behavior and the disabled clipboard reason. Full `bun run check:all` passed with 81 Rust, 118 frontend, 11 tooling, one loopback fixture and six ast-grep groups. Clipboard remains disabled as recorded above; the captured-target picker remains pending.
- A headless browser visual attempt failed during navigation and produced no screenshot. The owned browser session and temporary frontend server were subsequently confirmed closed. This is not visual acceptance evidence; the native initial-screen confirmation above is separate.
- Eligible physical automatic attempts: **0 / 300 required**, with **0 / 100** per candidate application. No zero-incident claim or latency/resource benchmark is available.
- Remote Windows CI has not been verified; the connected GitHub tool returned repository-specific 404 responses. This is an access/evidence limitation, not proof a workflow ran or failed.

Decision: **continue feasibility work**. No advance-to-product-MVP decision is justified yet. UIA revalidation and `SendInput` cannot form an atomic OS transaction; residual focus races require measured qualification, and uncertain output must stop mutation without repair/replay.

## Primary API references

- [UI Automation client threading](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading)
- [UI Automation properties and unsupported defaults](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-propertiesforclients)
- [UI Automation text units](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-uiautomationtextunits)
- [Text range endpoint movement](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtextrange-moveendpointbyrange)
- [GetText length contract](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtextrange-gettext)
- [Microsoft windows-rs bindings](https://github.com/microsoft/windows-rs)
- [Keyboard layout handle contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeyboardlayout)
- [Windows default input profiles](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/default-input-locales-for-windows-language-packs?view=windows-11)
