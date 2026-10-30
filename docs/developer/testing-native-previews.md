# Downloading native test previews

Open the repository's **Actions → Foundation checks**, then select the successful
run for the commit you want to test. On its **Summary** page, scroll to
**Artifacts**. Download the ZIP for your operating system, Linux session type and
CPU architecture. You must be signed in with read access to the repository.
Each native job summary also contains its direct artifact download link.

| Platform | x64 artifact | ARM64 artifact |
| --- | --- | --- |
| Windows | `texpio-preview-windows-x64` | `texpio-preview-windows-arm64` |
| macOS | `texpio-preview-macos-x64` | `texpio-preview-macos-arm64` |
| Linux X11 | `texpio-preview-linux-x11-x64` | `texpio-preview-linux-x11-arm64` |
| Linux Wayland | `texpio-preview-linux-wayland-x64` | `texpio-preview-linux-wayland-arm64` |

Downloads expire after 14 days, subject to repository retention policy. The
workflow's **Run workflow** button can create a new set for a selected branch.
Runs made before artifact uploads were added contain no downloadable app files.
These Actions downloads do not publish a GitHub Release.

Extract the outer ZIP and read `README.md`. `build-info.json` identifies the
source commit, version and target; `SHA256SUMS` records the staged files' hashes.
Only installers, app archives and test instructions are included, not Cargo
caches or debug files.

Windows downloads contain an unsigned NSIS installer. macOS downloads contain an
ad-hoc signed `.app` inside `.tar.gz`; extract that archive before opening the
application. They are not notarized, so macOS may require approval in **System
Settings → Privacy & Security**. Linux downloads contain a Debian package and an
AppImage. An AppImage extracted from the outer ZIP may need `chmod +x` before
running. Debian installation resolves system dependencies; an AppImage is not a
promise of compatibility with every distribution.

Wayland downloads additionally contain the separate helper Debian package and
`WAYLAND-HELPER.md`/`FEASIBILITY.md`. The current helper preview targets Ubuntu
24.04 GNOME 46. Installing the main application does not install or enable the
helper. Follow its explicit setup instructions only on a qualification machine.
X11 downloads contain no Wayland helper.

Close another Texpio instance before testing and back up the library: these
previews use the normal application identity and data location. Do not run the
application as root. macOS/Linux interactive qualification and the missing
composition providers remain separate from CI success. Fields lacking required
protection or composition evidence stay disabled. See
[native acceptance](../research/native-platform-acceptance.md).

References: [GitHub artifact downloads](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/download-workflow-artifacts),
[artifact file permissions](https://github.com/actions/upload-artifact#permission-loss),
[Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/).
