# Windows v1 prototype acceptance record

24 October 2026 · Checked Windows build; remaining native feature acceptance recorded below

## Scope and environment

The user requested Ampello's Windows feature families while retaining Texpio's UI/UX. The [Windows v1 design](../superpowers/specs/2026-10-24-windows-v1-design.md), [implementation plan](../superpowers/plans/2026-10-24-windows-v1.md), [parity tracker](windows-v1-parity.md) and [independent review/fix record](windows-v1-review.md) govern this work. Execution began at `84fefd6449cc66503da6bcd3fc70de5a10099be0` in the authorized current checkout. The [Ampello source audit](ampello-source-audit.md) records the reference revision; no upstream runtime implementation was copied.

This record supersedes the old product restrictions in the [offline MVP acceptance record](offline-mvp-acceptance.md). Historical failures and the strict Developer/P0 evidence remain valid as recorded. The new product engine is a separate Windows path; synthetic submission is not physical typing or proof of exact target output.

The test PC runs Windows 11 Home Single Language build 26300, x64, WebView2 155.0.4283.62 and Notepad 11.2607.14.0. Local tooling remains Bun 1.4.4, Node 24.18.0, Rust 1.99.1, Visual Studio Build Tools 17.14 and Windows SDK 10.0.26100.0. Ignored local tools and scratch logs are not portable evidence by themselves.

## Native insertion evidence

All observations below used disposable content. The explicit helpers are outside ordinary tests and install no input hooks themselves. The manual trigger harness explicitly started the product observer only for its bounded check. No uncertain input was repaired or replayed.

| Check | Submission / observation | Result and limit |
| --- | --- | --- |
| Initial Notepad Paste | 18/18 events; exact output observed | Passed for the owned fixture |
| Earlier Notepad Type, including after Ampello and TextExpander were closed | 52/52 events; output differed | Failed; concurrent expanders were not established as the cause |
| Notepad Type, production 100 ms minimum interval for `RichEditD2DPT`, ASCII greeting | 52/52 events; fresh full text and collapsed end caret matched | Product fixture passed; strict observer timed out and retained its uncertain outcome |
| Same production Type path, emoji and combining text | 38/38 events; exact observed output | Passed for the owned fixture |
| Same production Type path, Devanagari | 42/42 events; fresh full text and collapsed end caret matched | Product fixture passed; strict observer timeout remained uncertain |
| Same production Type path, multiline text | 50/50 events; Notepad returned CR line endings; Windows line-equivalent content and collapsed end caret matched | Semantic newline check passed; byte comparison against LF remained uncertain |
| Long Notepad Paste, 2,000-character body | 16/16 events; strict exact output observation | Passed for the owned fixture |
| Ordered file Paste into an owned native receiver | Two exact ordered `CF_HDROP` groups, including a Unicode filename; 8/8 events | Passed for file-only delivery; mixed text/files remains unverified |
| User typing `:v1probe` followed by Space, classified by Windows as software input | Complete capture: 69 callbacks, 69 injected events, zero own events; Running status, 0 submissions, 0 cancellations, no error, 0 usage | Failed before translation: the default input policy rejected every observed software-injected event. No exact output or physical qualification pass |
| Manual trigger with software-keyboard compatibility explicitly enabled | Owned receiver observed exact body plus terminating Space; receiver exit 0, one completed submission and one usage record, zero cancellations, no error | Passed for the owned native fixture on this PC; software input, not hardware/strict P0 qualification |

The control-specific Type floor does not slow Paste or classic EDIT controls. Its native-independent regression failed before implementation and passed afterward. It is evidence for this Notepad version and these fixtures, not a claim about every Windows editor. See [product effects](../../src-tauri/src/product/windows.rs) and the explicit [Notepad trial helper](../../src-tauri/examples/mvp_native_trial.rs).

The strict runtime's observation deadline was not extended or bypassed. The product helper's separate exact-output check requires a complete native submission plus a fresh bounded observation of the owned fixture. It does not turn `Uncertain` into a strict-probe success. **The strict P0 physical campaign remains 0/300.**

## Software keyboard diagnosis and compatibility setting

The complete capture report resolves the manual-trigger failure's discard stage: `callbacks=69`, `own_events=0`, `injected_events=69`, `key_down=0`, `stamp_refused=0`, `shortcut_refused=0`, `translation_failed=0`, `translated_scalars=0`, `spaces=0`, `matches=0`, `accepted=0`, and `callback_max_us=26`. Product state remained Running with zero submissions/cancellations, no error and zero usage. Windows classified all observed keyboard events as software injected, and the default guard rejected them before translation. This explains this trial; no silent hook timeout or caret-verification failure is claimed. The earlier truncated counter report did not establish a cause.

An explicit Rust-owned `allowSoftwareInput` preference has now been implemented, **off by default**, with a localized “Remote keyboard and key remapper compatibility” switch. When enabled, the product observer permits software events from the same or higher integrity level; its own marked output and lower-integrity injected events are always rejected. Matching, cancellation, effect, clipboard and target guards remain in place. The strict probe is unchanged, and software-generated input must never be counted as physical qualification.

Source learning from Ampello revision `c6ba8ea4a227b063df945597d1223f43f6e2f074` identified its own-marker filtering while allowing foreign injected events. Texpio's opt-in policy is an independent implementation; no source was copied.

The rebuilt debug application, including the compatibility option, **passed manual trigger-to-Paste acceptance** in the owned Win32 EDIT receiver on Windows 11 build 26300 with WebView2 155.0.4283.62. The user typed `:v1probe` followed by Space with compatibility explicitly enabled. The receiver observed exactly `Texpio Windows paste works.` followed by one Space and closed automatically; `receiverExit=0`. Before Stop, product state was Running with `submitted=1`, `cancelled=0`, `error=null` and `usage=1`.

The passing capture recorded `callbacks=42`, `own_events=22`, `injected_events=20`, `lower_integrity_events=0`, `software_accepted=20`, `key_down=10`, `sided_modifiers=1`, `generic_modifiers=0`, `stamp_refused=0`, `context_resets=1`, `own_target=0`, `shortcut_refused=0`, `translation_failed=0`, `translated_scalars=9`, `spaces=1`, `matches=1`, `accepted=1`, and `callback_max_us=76`. The 22 marked output events were ignored rather than recaptured. The app, receiver and loopback acceptance connection all ended afterward.

This resolves the central manual-trigger blocker for this owned fixture and software-classified input path. The default remains off; this PC's software-keyboard path needs explicit compatibility. The passing target was the owned native EDIT receiver, not a new Notepad manual trial. Other editors, hardware input and the remaining native feature checks do not inherit this result. **Strict P0 remains 0/300; this software-input trial contributes no physical qualification count.**

## Clipboard ownership

A native diagnostic exposed a real restoration bug: Windows synthesized additional clipboard formats when `CloseClipboard` ran, changing the sequence from 520 while locked to 523 after closing, with the same owner. Capturing the lease sequence before close caused cleanup to mistake its own publication for a newer copy.

The implementation now records the publication sequence after close, verifies its owner, and compares owner plus sequence under the clipboard lock before replacement or restoration. It checks ownership immediately before Ctrl+V and after settling. A zero sequence refuses publication. Unsupported snapshot formats remain a disclosed best-effort limitation in the product; the strict diagnostic requires a complete supported snapshot.

The explicit native ownership check, including the stronger post-restoration text-equality assertion, passed: a stale lease preserved a newer owned copy and the original supported formats and text were restored. The earlier failed diagnostic left its own disposable baseline on the clipboard; the user was told, and the known disposable marker was restored through an exact-match guarded helper. This record makes no claim that the earlier failed check left the clipboard unchanged.

Implementation and reproducible helper: [clipboard lease](../../src-tauri/src/product/clipboard.rs), [owned clipboard diagnostic](../../src-tauri/examples/clipboard_ownership_trial.rs). The native file receiver is in [product workflow trial](../../src-tauri/examples/product_workflow_trial.rs).

## Actual application workflows

Bounded loopback WebView2 checks used the isolated application identifier `io.github.texpio.texpio.acceptance`, real React/native commands and SQLite. They did not alter the user's normal library or Ampello data. Each completed check stopped observation, disconnected its browser connection and ended its owned app process before development work resumed.

The following passed:

- Product Start, Pause, Resume, Stop and settings round trip. No typed input was submitted by this lifecycle check.
- Favorite mutation, attachment byte intake/order/options, and metadata/file refresh while retaining an unsaved body draft.
- Literal find/replace, explicit undo, and Keep editing cancellation of the draft-discard guard.
- Healthy-library recovery disabled in the dialog and an ungranted export path refused by native scope checks.

The first settings save failed because Windows refused the original default Ctrl+Shift+Space registration with error 1409. A bounded registration/unregistration probe found Ctrl+Alt+Shift+Space available. The fresh default now uses that chord; explicitly saved shortcuts are preserved and conflicts have a localized error. Registration availability does not qualify actual hotkey delivery.

The first desktop acceptance check failed because `--hidden` produced a visible main window. The Windows-specific window configuration replaced the base window entry without retaining `visible: false`. A regression demonstrated the missing effective value, then passed after the Windows configuration was corrected. The failed attempt ended its owned app and connection before diagnosis.

The corrected embedded-build desktop check passed: hidden startup, creation/removal of the owned HKCU Run entry, second-instance restoration of the visible main window, close-to-tray hiding, and close-to-Quit cancellation retaining an unsaved draft through Keep editing. No typing was submitted to another app. The owned acceptance connection and process ended afterward. This verifies the sign-in command's registration and a hidden launch; it is not a Windows sign-out/sign-in campaign or a physical tray-menu gesture.

The prior checkpoint separately passed explicit main/picker Copy, picker focus/Escape, close-to-tray and second-instance restore; see the [earlier record](offline-mvp-acceptance.md). Those checks do not establish acceptance of the new desktop preferences. The current workflow screenshot caught a closing dialog animation and is not a stable final visual check.

## Automated and package evidence

The final serial `bun run check:all` completed with exit 0: **152 frontend tests, 174 Rust tests, 12 tooling tests and one loopback fixture test**, plus TypeScript, ESLint, AST checks, Prettier, rustfmt and strict all-targets Clippy. Earlier task 8 and review-fix checkpoints passed 150/158 and 151/169 frontend/Rust tests respectively; the final run includes the subsequent state/policy/UI changes.

The independent reviewer examined immutable snapshot `79e91e3b30bf2ff6490dbedd0afa82501921669e` against the execution base and returned **No** for that snapshot, with seven Important findings, no Critical findings and no Minors. All seven entered one fix pass with failing-then-passing regressions, followed by the 169-test Rust library run. The [review record](windows-v1-review.md) maps findings to changes and evidence. No second independent review of the fixes was performed; remaining native acceptance is still required before describing v1 as complete.

The implementation includes tests for matching, cancellation and publication generations; migrations and metadata revisions; attachment integrity/order; local-library routing and external refresh; bounded JSON/YAML/ZIP imports; corruption-only recovery preserving DB/WAL/SHM; shortcut/startup rollback; and draft preservation. Ordinary checks do not arm global input.

Later native-independent state-edge regressions covered generic Shift and injected key-up handling (two failures first, then 5/5 focused tests passed) and initial keyboard-state seeding (failed first, then passed). The Rust library suite passed 173/173 before the new compatibility policy regression, which also failed first and passed. The final 174-test run passed. The compatibility UI passed 5/5, and affected frontend files passed 26/26. An intermediate full frontend run during concurrent Rust compilation was 150/152, with a library-folder timeout and a probe status assertion observing Stopped instead of Running; the final serial run passed all 152. No cause is assigned to concurrency solely from that timing.

| Final gate | Current status |
| --- | --- |
| Fresh `bun install --frozen-lockfile` | Passed with Bun 1.4.4: 687 installs across 787 packages, no changes |
| Fresh `bun run check:all` after latest changes | Passed with exit 0: 152 frontend, 174 Rust, 12 tooling, one fixture and static checks |
| Generated bindings and local document links | Regenerated after the final DTO; full typecheck passed. Root link check passed 79/79 before this final documentation update; final recheck follows |
| One fresh whole-change independent review | Completed for snapshot `79e91e3`; verdict No. Seven Important findings fixed in one regression-tested pass; no rereview |
| Rust library suite after review fixes | Final pass 174/174, including compatibility policy |
| Current normal-identifier optimized executable, MSI and NSIS | `bun run tauri build -- --locked` passed with exit 0; artifacts and hashes below |
| Commit/push of implementation checkpoint | Pending |

The optimized executable and both Windows bundles were built successfully. Paths are relative to the repository root; these local build outputs are not committed source files.

| Artifact | Bytes | SHA-256 |
| --- | --- | --- |
| `src-tauri/target/release/texpio.exe` | 12,037,120 | `DB1543A2D765AA9967CE728FFB1FC39309BD601D725EDEC76762B59CB7783AC9` |
| `src-tauri/target/release/bundle/msi/Texpio_0.1.0_x64_en-US.msi` | 5,816,320 | `E7D89F5409CA0E897B77DE6A4AF8DD5C4D3677114D6A02DCD5A90F67AA1ECD1D` |
| `src-tauri/target/release/bundle/nsis/Texpio_0.1.0_x64-setup.exe` | 4,242,270 | `307C3A8BA84743765CCC2C2633DE192B173527C4F7E73547F5F6A89FE229E275` |

No installer was executed, and no signing, publishing or public distribution occurred. Other operating systems and all-editor compatibility remain outside the current evidence.

## Normal release preview

The normal optimized executable was configured through permitted native commands, retaining the original product settings except `allowSoftwareInput=true` for this PC's software-keyboard path and `enabled=false` so observation does not start on launch. No snippet or library content was changed for this preview setup. The temporary app exited through `quit-requested` and the existing draft guard; its loopback connection ended before a clean launch without debugging arguments.

Read-only native/process checks verified the exact normal release executable path, a visible main window (PID 12416 at verification), the normal profile's two settings, and no listener on port 9225. The release preview was left open for the user, with automatic observation off on launch; the user can choose Start expansion. Factory defaults for new profiles still keep compatibility off. This preview check does not add another expansion or installer acceptance result.

## Remaining native checks

- Exercise expansion-toggle and clipboard insertion shortcuts against an owned native target, including cancellation/focus change during their effects.
- Qualify a concurrent real copy without overwriting it. The stronger owned clipboard restoration assertion has passed.
- Verify mixed text/file order and delay, missing files and cancellation between groups in a native receiver.
- Exercise actual selected-file attachment intake, JSON/YAML/ZIP export/import dialogs and selected-directory personal/shared switching. Unit/transaction tests and refusal of ungranted paths do not substitute for these flows.
- Complete physical tray gestures, broader Quit/restart persistence and a stable visual check. Hidden startup, startup-entry create/remove, second-instance restore, close-preference draft cancellation and the normal release preview launch have passed as described above.

The central manual trigger-to-exact-output check passes with software-keyboard compatibility explicitly enabled in the owned fixture, and the final checks/packages pass. Full Ampello feature acceptance, broader editor compatibility and hardware/strict P0 qualification remain open; commit/push is pending.
