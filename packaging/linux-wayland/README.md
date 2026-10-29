# Optional Wayland helper preview

This package targets Ubuntu 24.04 GNOME 46 on amd64 or arm64. It is an incomplete
native feasibility preview, not a qualified input backend. The AT-SPI composition
proof described in `FEASIBILITY.md` is still missing. Broker-backed bridge
authentication is implemented but requires native qualification. The helper refuses readiness
when required evidence is absent. Do not install this package expecting working
expansion in ordinary applications yet.

Build on the matching Linux architecture, without running the daemon:

```sh
cargo test --manifest-path native/wayland-helper/Cargo.toml --locked
cargo build --manifest-path native/wayland-helper/Cargo.toml --release --locked
sh packaging/linux-wayland/build-deb.sh amd64 native/wayland-helper/target/release/texpio-wayland-helper dist
```

Use `arm64` for the ARM64 build. Compilation requires libsystemd-dev,
libatspi2.0-dev and libxkbcommon-dev. The package name always contains `wayland`.
Do not reuse the X11 package or updater identity.

Explicit one-time administrator setup, only on a disposable qualification host:

```sh
sudo apt install ./texpio-wayland-helper_0.1.0_amd64.deb
sudo modprobe uinput
sudo systemctl enable --now texpio-wayland-helper.socket
```

Log out and back in after extension installation, then explicitly enable it as the
normal desktop user:

```sh
gnome-extensions enable wayland-bridge@texpio.local
```

Run Texpio as the ordinary desktop user. Never run Tauri as root, setuid it, add
file capabilities to it, or add the user to the `input` group. The system broker
checks SO_PEERCRED and logind, then supervises a worker which drops UID/GID/groups
and its capability bounding set. The worker alone keeps approved seat0 device
FDs. The root broker publishes bounded per-worker PID/UID/kernel-starttime
attestation in `/run/texpio-attest`, managed separately from the app socket
directory. No exclusive keyboard grab is used. Root service restart, lock, inactive
session, app death/disconnect, heartbeat loss and input overflow terminate the
worker and close descriptors; a new connection starts a new generation.

Read-only system status:

```sh
systemctl status texpio-wayland-helper.socket texpio-wayland-helper.service
```

Complete uninstall:

```sh
gnome-extensions disable wayland-bridge@texpio.local
sudo systemctl disable --now texpio-wayland-helper.socket
sudo systemctl stop texpio-wayland-helper.service
sudo apt purge texpio-wayland-helper
```

The package installs no user-group membership, setuid bit, Tauri capability,
udev access rule, snippets, or clipboard history. Purge removes the root helper,
system units, and global extension. User snippet libraries remain owned by Texpio.
