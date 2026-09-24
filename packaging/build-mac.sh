#!/bin/bash
# 构建 macOS 应用 CavalryKit.app
# 用法: packaging/build-mac.sh [--install]
#   --install 构建成功后拷贝到 /Applications
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

APP_NAME="CavalryKit"
APP_DIR="$APP_NAME.app"

echo "==> 生成图标（若需要）"
if [ ! -f assets/icon.png ]; then
    cargo run --quiet --example gen_icon
fi
if [ ! -f assets/icon.icns ]; then
    ICONSET="$(mktemp -d)/icon.iconset"
    mkdir -p "$ICONSET"
    for size in 16 32 64 128 256 512; do
        sips -z $size $size assets/icon.png --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    done
    for size in 32 64 256 512; do
        sips -z $size $size assets/icon.png --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
    done
    iconutil -c icns "$ICONSET" -o assets/icon.icns
    echo "    已生成 assets/icon.icns"
fi

echo "==> 编译 release"
cargo build --release

echo "==> 组装 $APP_DIR"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
cp target/release/cavalrykit "$APP_DIR/Contents/MacOS/$APP_NAME"
cp packaging/Info.plist "$APP_DIR/Contents/Info.plist"
cp assets/icon.icns "$APP_DIR/Contents/Resources/icon.icns"
chmod +x "$APP_DIR/Contents/MacOS/$APP_NAME"

echo "==> ad-hoc 签名"
codesign --force --sign - "$APP_DIR"

echo "==> 完成: $(pwd)/$APP_DIR"

if [ "${1:-}" = "--install" ]; then
    echo "==> 拷贝到 /Applications"
    rm -rf "/Applications/$APP_DIR"
    cp -R "$APP_DIR" "/Applications/$APP_DIR"
    echo "==> 已安装到 /Applications/$APP_DIR"
fi
