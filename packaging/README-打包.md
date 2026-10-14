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
脚本先检查 `assets/icon-frames/`、`assets/icon.ico` 与 `assets/icon.icns` 的批准帧，使用 `iconutil` 解码校验后直接嵌入 ICNS，进行 ad-hoc 签名及验证，并打印 SHA-256。
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

- 原始上传图案 `assets/icon-source.ico` 保持不变；当前批准设计为 `icon-final-inset-v6`，不得重画已退回的金属边框或后续候选。
- `python3 scripts/render_platform_icons.py --check` 先校验冻结的逐尺寸SHA，再检查组装结果；此脚本保留历史名称，但不会重新渲染图案。
- `cargo run --locked --example import_icon` 仅将批准PNG帧组装成ICO/ICNS和应用运行时图像，不重采样。
- macOS：1024画布中不透明板面824px为本项目光学约定，不是Apple强制值；11个ICNS块覆盖16–1024px表示。资源文件名含当前内容hash，用于区分旧图标资源，不重置全局图标缓存。
- Windows：18帧ICO分别为16/20/24/30/32/36/40/48/60/64/72/80/96/128/144/160/192/256px；每帧保留批准的小尺寸光学校正。
- `app`和`macos-brand`使用批准的无Dock留白图形，运行时读取精确256px帧。原始符号仍是256px位图，放大不增加真正矢量细节。
- 生产PNG、ICO、ICNS和PE内嵌帧须与批准清单匹配；浏览器DPR与容器检查不替代原生Dock/任务栏/Explorer/DPI验证。

## 本版验证入口

`cargo test --all-targets --all-features --locked` 和严格 Clippy 同时验证生产/预览代码。Windows 原生 `packaging/test-installer-win.ps1` 在临时目录验证原位升级、图标快捷方式、恢复包与失败路径，不启动 Cavalry。CI 工作流为 `.github/workflows/desktop.yml`，只有 Contents 读取权限，没有发布或凭据写入权限。

安装包包含 `THIRD-PARTY-NOTICES.txt`，保留随应用分发的字体授权。`ui-preview` 不编译进正常 release，仅用于输出没有真实用户内容的 UI 图片。
