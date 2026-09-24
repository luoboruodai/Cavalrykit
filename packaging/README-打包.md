# CavalryKit 打包说明

CavalryKit 是单一代码库的跨平台应用:macOS 产出 `.app` / `.dmg`,Windows 产出 `.exe`。

## macOS

```bash
packaging/build-mac.sh            # 产出 ./CavalryKit.app
packaging/build-mac.sh --install  # 构建并拷贝到 /Applications
hdiutil create -volname CavalryKit -srcfolder CavalryKit.app -ov -format UDZO dist/CavalryKit-mac.dmg
```

首次构建需要联网拉取 crates.io 依赖;之后可离线。

## Windows

方式一(在 Windows 电脑上构建):把整个文件夹拷到 Windows,双击 `packaging\build-win.bat`,
脚本自动安装 Rust 工具链(winget,需联网)并产出 `dist\CavalryKit.exe`。

方式二(在 macOS 上交叉编译,本项目 dist 内的 exe 即此产出):

```bash
cargo install cargo-xwin --locked
rustup target add x86_64-pc-windows-msvc
cargo xwin build --release --target x86_64-pc-windows-msvc
# 产物:target/x86_64-pc-windows-msvc/release/cavalrykit.exe
```

首次交叉编译会下载 Windows SDK(约 1-2GB)。

## 文件关联(macOS)

`CavalryKit.app` 的 Info.plist 已声明 `.jsc / .js / .jsx / .zip` 文档类型,
可在访达中对这类文件使用「打开方式 → Cavalry 装备库」,或直接把文件拖到
Dock 上的应用图标，都会自动进入安装流程。
