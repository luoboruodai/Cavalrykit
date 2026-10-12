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
cargo build --release --locked
# Always regenerate ICNS from the macOS master with optical margins; no stale icon cache in packaging.
ICONSET="$WORK/icon.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
    sips -z "$size" "$size" assets/icon-macos.png --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z "$double" "$double" assets/icon-macos.png --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
APP="$WORK/dmg/CavalryKit.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" dist
cp target/release/cavalrykit "$APP/Contents/MacOS/CavalryKit"
cp packaging/Info.plist "$APP/Contents/Info.plist"
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/CavalryKitIcon.icns"
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
