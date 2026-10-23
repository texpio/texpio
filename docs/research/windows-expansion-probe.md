# Windows expansion probe evidence

Status on 23 October 2026: **implementation in progress; no input strategy qualified**. This report distinguishes code checks from native acceptance and physical typing evidence. The preview starts Stopped; no live hook has been armed during these checks.

## Environment recorded during preflight

| Item | Observed version |
| --- | --- |
| OS API description | Microsoft Windows 10.0.26300, X64 |
| Notepad package | 11.2607.14.0, X64 |
| Visual Studio Code candidate | 1.143.0, x64 |
| Chrome candidate | 155.0.8059.85, x64 |
| Node / pnpm | 24.18.0 / 12.15.0 |
| Rust | 1.99.1 |
| Windows bindings | 0.62.2 |

These app versions were inventoried, not qualified for expansion. The OS description is the literal measured API result; kernel version 10 alone does not identify the Windows product edition.

## Foundation acceptance

Observed: native metadata reports Connected/version/platform; System/Light/Dark theme switching; close hides a still-running process; launching a second actual process exits that process and restores the original window with one process remaining.

The user subsequently verified close → tray Open → tray Quit manually. A CLI process check found zero Texpio processes before the requested relaunch, and the preview was relaunched successfully. The minimized-window variant remains unverified. Computer Use was stopped after the earlier stalled closure and has not been restarted. Repository rules require brief sessions and immediate cleanup.

The updated native preview was built successfully and the user confirmed the probe panel, initial Stopped status and fixed snippets. A screenshot also shows the rendered snippets and controls. This confirms the initial screen only; running/paused tray parity and native insertion remain unverified.

## Read-only context implementation

Rust owns worker-local UIA objects in an MTA apartment. Captured tokens retain the original native foreground/focus/process/thread stamp, logical focused control, process handle, and worker session identity. Tokens and captured text do not serialize to React.

Before adjacent text is read, the adapter requires explicit `VT_BOOL` non-password and enabled properties, a qualified candidate control, a single collapsed selection and explicit non-readonly text attribute. Unsupported properties and provider defaults refuse. It checks standard-user medium integrity, exact US keyboard layout, modifiers/composition, logical control identity, native focus, finite provider timeouts and an overall preparation deadline. Only bounded adjacent text may establish an exact trigger plus Space suffix and whitespace/document-start boundary. Revalidation repeats the suffix and caret checks.

Post-insertion observation is implemented for later dispatch integration: retain the untouched left boundary, reconstruct a bounded range to the fresh caret, and compare exact Unicode output. A proven document start uses a fresh document range. UIA Character units are not UTF-16 offsets, so verification never walks backward by body UTF-16 length. Line endings are compared exactly; no application-specific conversion has been qualified.

Candidate selectors are deliberately strict and **not yet measured support**:

| Candidate | Intended scope | Native qualification |
| --- | --- | --- |
| Notepad | Disposable `texpio-p0-notepad.txt`; Win32 `RichEditD2DPT` document named Text editor | Pending actual UIA discovery |
| VS Code | Disposable `texpio-p0-vscode.txt` plaintext editor; excludes Find and other controls | Pending actual UIA discovery |
| Chrome | Own loopback page; only labeled positive textarea/input; ancestry must prove document URL and foreground ownership | Pending actual UIA discovery |

If these installed builds expose different identities or insufficient password/caret/range/origin evidence, they remain refused. Do not broaden admission to executable-name-only matching to obtain a passing result.

## Repeatable read-only check

Build first with `cargo build --locked --manifest-path src-tauri/Cargo.toml --example probe_context`. Run the resulting `src-tauri/target/debug/examples/probe_context.exe` with `notepad`, `vscode`, or `chrome`. It waits five seconds for the user to focus the synthetic reference document and checks once. It never installs hooks, sends keys, changes clipboard data, or prints captured text/handles/paths. Keep the caret after `;hello` and exactly one Space, at document start or following whitespace.

The helper returns a content-free refusal reason or read-only preparation acceptance. Acceptance alone is not an input-strategy qualification. Record each target's actual UIA capabilities/refusals here before enabling mutation. Password-field values must never be read.

One user-assisted Notepad check completed and returned `UnsupportedLayout`, with no mutation. The active HKL observed afterward was `0x40094009`, corresponding to English (India), and configured input methods were `4009:00004009` and `0409:00004009`. Both use the India keyboard layout. This is an expected refusal of the current exact-US profile, not a successful UIA qualification or an insertion failure. No system input preference was changed. Further provider/caret checks remain pending with the approved US layout.

For the browser fixture, run `pnpm probe:fixture` and open `http://127.0.0.1:4175/` in Chrome. The server binds IPv4 loopback, serves only three known GET assets, and accepts no uploads or request bodies. The page includes synthetic sentinels, positive controls, second focus target, password, readonly, disabled and exploratory contenteditable controls. Use only these synthetic fixtures. Stop the server with Ctrl+C when done.

## Evidence boundaries and remaining work

- Unit tests cover bounded matching, rollover/repeat/reset behavior, explicit property semantics, unsafe guard states, exact suffix/boundary rules, two-field identity mismatch and candidate selectors. These are synthetic evidence and do not validate actual UIA providers.
- The fixture-server test covers loopback binding, asset allowlist, traversal rejection, method rejection and response security headers.
- Runtime code now compiles with a dedicated observation/message-loop thread, separate worker-local UIA backend, bounded candidate queue, generation checks, one Unicode batch and exact output observation. Construction and ordinary tests install no hooks and send no input. Native hook teardown, provider shutdown and actual insertion remain unqualified.
- The latest Rust suite has 44 passing tests. It includes Stop during blocked preparation, acknowledged in-flight uncertainty, stale/consumed tickets, unexpected injections, explicit US mapping, UTF-16 encoding, zero-submission accounting, closed command serialization and latched teardown failure. Strict Clippy passes with only host filesystem incremental-cache hard-link warnings.
- No native provider identity/caret/range campaign has been completed. The preparation deadline and UIA timeouts still need slow-provider and cancellation qualification with the later runtime watchdog.
- Probe commands, a validated React bridge, content-free status polling, maintainer controls and tray state now compile. The frontend suite has 55 passing tests, including stale responses after Stop, malformed IPC, browser refusal and pending control behavior. Clipboard and captured-target picker remain pending; results will be recorded separately.
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
