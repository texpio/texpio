#!/bin/sh
set -eu
# Build only: does not install, enable, or run the helper. Run on Linux.
arch=${1:?Usage: build-deb.sh amd64|arm64 TARGET_BINARY OUTPUT_DIRECTORY}
binary=${2:?target binary required}
output=${3:?output directory required}
case "$arch" in amd64|arm64) ;; *) exit 2 ;; esac
base=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
[ -f "$binary" ] && [ -d "$output" ]
stage=$(mktemp -d)
trap 'rm -rf -- "$stage"' EXIT HUP INT TERM
install -d "$stage/DEBIAN" "$stage/usr/libexec" "$stage/usr/lib/systemd/system" "$stage/usr/share/doc/texpio-wayland-helper" "$stage/usr/share/gnome-shell/extensions/wayland-bridge@texpio.local"
install -m 0755 "$binary" "$stage/usr/libexec/texpio-wayland-helper"
install -m 0644 "$base/packaging/linux-wayland/texpio-wayland-helper.service" "$base/packaging/linux-wayland/texpio-wayland-helper.socket" "$stage/usr/lib/systemd/system/"
install -m 0644 "$base/native/gnome-extension/extension.js" "$base/native/gnome-extension/auth-policy.js" "$base/native/gnome-extension/metadata.json" "$stage/usr/share/gnome-shell/extensions/wayland-bridge@texpio.local/"
install -m 0644 "$base/packaging/linux-wayland/README.md" "$base/native/wayland-helper/FEASIBILITY.md" "$stage/usr/share/doc/texpio-wayland-helper/"
cat > "$stage/DEBIAN/control" <<EOF
Package: texpio-wayland-helper
Version: 0.1.0
Section: utils
Priority: optional
Architecture: $arch
Maintainer: Texpio contributors
Depends: libc6, libsystemd0, libatspi2.0-0, libglib2.0-0t64, libxkbcommon0, systemd, gnome-shell (>= 46), gnome-shell (<< 47)
Description: Texpio Wayland broker and GNOME 46 feasibility preview
 Separate session-scoped input helper. Native field composition qualification
 remains incomplete; installation does not establish expansion support.
EOF
install -m 0755 "$base/packaging/linux-wayland/postinst" "$stage/DEBIAN/postinst"
install -m 0755 "$base/packaging/linux-wayland/prerm" "$stage/DEBIAN/prerm"
install -m 0755 "$base/packaging/linux-wayland/postrm" "$stage/DEBIAN/postrm"
dpkg-deb --root-owner-group --build "$stage" "$output/texpio-wayland-helper_0.1.0_${arch}.deb"
