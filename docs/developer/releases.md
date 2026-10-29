# Release preparation and distribution

Update adapters/UI exist, but no active feed/key is configured and startup does not check updates. Native readiness requires a parsable public minisign key and real HTTPS endpoints. Placeholders keep updates unconfigured. Never commit private keys.

`bun run release:prepare 0.2.0 --dry-run` reports local changes. Without dry-run it updates package, Cargo package/lock and Tauri versions. It cannot spawn processes, sign, commit, tag, push or publish. Run full checks/frozen installs afterwards and review the diff.

The three-OS workflow is manual, opt-in and environment gated. It checks explicit version/commit agreement and real updater/signing setup. Windows/macOS need platform certificate/notarization configuration. Normal pushes create no release. Workflow output is draft-only; publication remains separate.

The shared target matrix now builds eight variants: Windows/macOS x64 and ARM64,
and separate Linux X11/Wayland editions on both architectures. Linux downloads
have distinct product names and `latest-linux-x11.json` /
`latest-linux-wayland.json` feeds; do not combine their identical updater platform
keys in one manifest. Native configuration rejects the opposite edition's feed.
Wayland jobs also assemble a separately installed helper Debian package. Source
previews still have functional/qualification gaps in the
[native acceptance record](../research/native-platform-acceptance.md); creating
a draft does not establish that those editions work.

Configured checks are user-triggered; download/install needs confirmation. Cancel closes resources. Install and restart require confirmed probe shutdown; Windows install may exit immediately, so stop happens just before install. Signed delivery and packaged acceptance remain external gates.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
