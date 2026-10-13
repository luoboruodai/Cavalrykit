#!/bin/bash
# Build a versioned DMG and an ad-hoc signed app. Does not alter installed apps.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$PWD=/build/cavalrykit"$'\x1f'"--remap-path-prefix=$HOME=/build/home"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)"
ARCH="$(uname -m)"
[ "$ARCH" = "aarch64" ] && ARCH="arm64"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
# Reject stale generated bundles; never downsample away approved optical-size frames.
cargo run --locked --example import_icon -- --check
# Changing the resource name with the icon content avoids reusing an older
# same-named application icon rendition. This does not reset system-wide caches.
ICON_STEM="$(/usr/libexec/PlistBuddy -c 'Print CFBundleIconFile' packaging/Info.plist)"
ICON_HASH="$(shasum -a 256 assets/icon.icns | cut -c 1-12)"
if [ "$ICON_STEM" != "CavalryKitIcon-$ICON_HASH" ]; then
    echo "CFBundleIconFile must match the current icon content: CavalryKitIcon-$ICON_HASH" >&2
    exit 1
fi
cargo build --release --locked
# iconutil must decode the approved ICNS before it is embedded in the app.
ICNS_CHECK="$WORK/iconset-check.iconset"
iconutil -c iconset assets/icon.icns -o "$ICNS_CHECK" >/dev/null
APP="$WORK/dmg/CavalryKit.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" dist
cp target/release/cavalrykit "$APP/Contents/MacOS/CavalryKit"
cp packaging/Info.plist "$APP/Contents/Info.plist"
cp assets/icon.icns "$APP/Contents/Resources/$ICON_STEM.icns"
cp packaging/THIRD-PARTY-NOTICES.txt "$APP/Contents/Resources/THIRD-PARTY-NOTICES.txt"
chmod +x "$APP/Contents/MacOS/CavalryKit"
codesign --force --sign - "$APP"
codesign --verify --deep --strict "$APP"
plutil -lint "$APP/Contents/Info.plist"
if [ -e dist/CavalryKit.app ]; then mv dist/CavalryKit.app "$WORK/previous-build.app"; fi
ditto "$APP" dist/CavalryKit.app
ln -s /Applications "$WORK/dmg/Applications"
hdiutil create -quiet -volname "CavalryKit $VERSION" -srcfolder "$WORK/dmg" -ov -format UDZO "dist/CavalryKit-$VERSION-mac-$ARCH.dmg"
shasum -a 256 "dist/CavalryKit-$VERSION-mac-$ARCH.dmg"
echo "Build complete. To upgrade, quit CavalryKit and replace the existing app in Applications."
