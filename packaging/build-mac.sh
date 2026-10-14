#!/bin/bash
# Build a versioned DMG and an ad-hoc signed app. Does not alter installed apps.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$PWD=/build/cavalrykit"$'\x1f'"--remap-path-prefix=$HOME=/build/home"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)"
ARCH="$(uname -m)"
[ "$ARCH" = "aarch64" ] && ARCH="arm64"
mkdir -p dist
WORK="$(mktemp -d "dist/.package-mac.XXXXXX")"
MOUNT="$WORK/mounted"
cleanup() {
    if mount | grep -F " on $PWD/$MOUNT " >/dev/null; then hdiutil detach -quiet "$MOUNT"; fi
    rm -rf "$WORK"
}
trap cleanup EXIT
if [ "$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' packaging/Info.plist)" != "$VERSION" ]; then
    echo "Info.plist version differs from Cargo.toml" >&2; exit 1
fi
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
for name in INSTALL.zh-CN.txt INSTALL.en.txt RELEASE_NOTES.md; do
    source="packaging/$name"
    [ "$name" = RELEASE_NOTES.md ] && source="$name"
    head -1 "$source" | grep -F "CavalryKit $VERSION" >/dev/null
    grep -F "/releases/tag/v$VERSION" "$source" >/dev/null
    cp "$source" "$WORK/dmg/$name"
done
ln -s /Applications "$WORK/dmg/Applications"
CANDIDATE="$WORK/candidate.dmg"
hdiutil create -quiet -volname "CavalryKit $VERSION" -srcfolder "$WORK/dmg" -format UDZO "$CANDIDATE"
hdiutil verify -quiet "$CANDIDATE"
mkdir "$MOUNT"
hdiutil attach -quiet -readonly -nobrowse -mountpoint "$MOUNT" "$CANDIDATE"
codesign --verify --deep --strict "$MOUNT/CavalryKit.app"
diff -qr "$APP" "$MOUNT/CavalryKit.app"
for name in INSTALL.zh-CN.txt INSTALL.en.txt RELEASE_NOTES.md; do
    cmp "$WORK/dmg/$name" "$MOUNT/$name"
done
[ "$(readlink "$MOUNT/Applications")" = /Applications ]
hdiutil detach -quiet "$MOUNT"
mv -f "$CANDIDATE" "dist/CavalryKit-$VERSION-mac-$ARCH.dmg"
if [ -e dist/CavalryKit.app ]; then mv dist/CavalryKit.app "$WORK/previous-build.app"; fi
ditto "$APP" dist/CavalryKit.app
shasum -a 256 "dist/CavalryKit-$VERSION-mac-$ARCH.dmg"
echo "Build complete. To upgrade, quit CavalryKit and replace the existing app in Applications."
