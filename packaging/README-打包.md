# CavalryKit 打包 / Packaging

版本以 Cargo.toml 为主；更新版本时同步 Info.plist、Windows manifest、说明和资源验证器。
默认构建不会更新系统安装目录，不会清理用户的其他软件副本。

## macOS

前提：Rust、Xcode Command Line Tools、macOS 原生打包工具。

```bash
cargo test --locked
packaging/build-mac.sh
```

输出 `dist/CavalryKit.app` 和架构命名的 DMG（Apple Silicon 为 `CavalryKit-0.1.1-mac-arm64.dmg`）。
脚本每次由 `assets/icon-macos.png` 重建全部 ICNS 尺寸，进行 ad-hoc 签名及验证，并打印 SHA-256。
这是本地签名，不是 Apple Developer ID 签名/公证。正式分发的公证需要另行配置开发者凭据，不能把凭据提交到仓库。
安装时退出原应用，替换 Applications 中同一 `CavalryKit.app`，再推出旧镜像；不要改名并排安装。

## Windows（原生构建，优先）

前提：Rust stable、MSVC C++ 工具链、Windows SDK。脚本不会替用户自动安装依赖。

```bat
packaging\build-win.bat
```

`build.rs` 强制嵌入 ICO、版本资源和 PerMonitorV2 manifest，资源编译失败时中止构建。
`package-win.ps1` 生成包含 EXE、固定路径安装器和说明的版本化 ZIP。
安装器只操作当前用户安装路径和固定名称快捷方式，更新前要求用户关闭应用，不擅自杀进程、不修改系统文件关联。

## macOS 交叉构建 Windows

前提：cargo-xwin、Windows Rust target、Clang/lld、资源编译器（llvm-rc 或 MinGW windres）。

```bash
rustup target add x86_64-pc-windows-msvc
# 若没有 llvm-rc，可以显式选择本机安装的 MinGW windres：
export RC_x86_64_pc_windows_msvc="$(command -v x86_64-w64-mingw32-windres)"
export RUSTFLAGS="-C target-feature=+crt-static --remap-path-prefix=$PWD=C:/build/cavalrykit --remap-path-prefix=$HOME=C:/build/home"
cargo xwin build --locked --release --target x86_64-pc-windows-msvc
python3 packaging/verify_windows.py target/x86_64-pc-windows-msvc/release/cavalrykit.exe
python3 packaging/package-win.py target/x86_64-pc-windows-msvc/release/cavalrykit.exe
```

Windows 默认静态链接 VC++ 运行时；原生构建使用 encoded flags，保留带空格的用户名/项目路径。

静态验证检查 GUI 子系统、x64、内嵌 ICO 原始像素帧、版本、DPI manifest 和本机路径泄漏；**不会执行 EXE**。
原生 Windows 构建与安装/拖窗/任务栏验收仍须按 [VALIDATION.md](VALIDATION.md) 进行。

## 发布前检查

- `cargo fmt --check`、`cargo clippy --all-targets --locked -- -D warnings`、`cargo test --locked`。
- 只使用公开文件白名单打包；私有 PROJECT.md、日志、截图、备份、用户导入内容不入库。
- 对待提交 blob、历史和归档实际成员检查；为两个安装包生成 SHA-256。
- 按用户本轮授权检查通过后上传，核对源码提交与附件一致；如实标明 Windows 人工实机验收尚未完成；不移动已有发布标签。


## 平台图标规范与生成

- 原始上传图案：`assets/icon-source.ico`，保留不覆盖；禁止用已缩小或带留边的输出再次充当输入。
- `cargo run --locked --example import_icon` 进行可重复生成，不使用 AI 重绘。
- macOS：`assets/icon-macos.png` 为 1024×1024 母图；有效不透明底板为 824×824（约 80.5%），这来自本机系统图标的视觉比例对照，**不是 Apple 强制规定的数值**。原图含抗锯齿边缘，因此完整图案缩放尺寸为 830px。另保留透明外边距与轻微投影。
- ICNS 按 Apple 的 16、32、128、256、512 pt 各配 1×/2×，共 10 个槽位（像素覆盖 16～1024）。包内资源文件为 `CavalryKitIcon.icns`。
- Windows：`assets/icon.ico` 是 32-bit RGBA 多尺寸容器，包含 16、20、24、30、32、36、40、48、60、64、72、80、96、128、256 px；覆盖 Microsoft 建议的最少 16/24/32/48/256 以及常用缩放档位。
- 每个 Windows 尺寸都直接从原图采样，不从小图再次放大。`assets/icon.png` 保留 256px 的应用内品牌版本，macOS 原生窗口另用带留边的 256px PNG。
- 原图只有 256px；生成高分辨率容器不会凭空增加原始图案细节。
- 核对实际解包的 ICNS/EXE 资源，而非只检查文件扩展名。原图、重建、边距、ICO 每一帧均有自动回归测试。

官方参考：
- Apple High Resolution Guidelines / Provide High-Resolution Versions of All App Graphics Resources: https://developer.apple.com/library/archive/documentation/GraphicsAnimation/Conceptual/HighResolutionOSX/Optimizing/Optimizing.html
- Microsoft App icon construction: https://learn.microsoft.com/en-us/windows/apps/design/style/iconography/app-icon-construction


## 本版验证入口

`cargo test --all-targets --all-features --locked` 和严格 Clippy 同时验证生产/预览代码。Windows 原生 `packaging/test-installer-win.ps1` 在临时目录验证原位升级、图标快捷方式、恢复包与失败路径，不启动 Cavalry。CI 工作流为 `.github/workflows/desktop.yml`，只有 Contents 读取权限，没有发布或凭据写入权限。

安装包包含 `THIRD-PARTY-NOTICES.txt`，保留随应用分发的字体授权。`ui-preview` 不编译进正常 release，仅用于输出没有真实用户内容的 UI 图片。
