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
脚本每次由 `assets/icon.png` 重建全部 ICNS 尺寸，进行 ad-hoc 签名及验证，并打印 SHA-256。
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
```

Windows 默认静态链接 VC++ 运行时；原生构建使用 encoded flags，保留带空格的用户名/项目路径。

静态验证检查 GUI 子系统、x64、内嵌 ICO 原始像素帧、版本、DPI manifest 和本机路径泄漏；**不会执行 EXE**。
原生 Windows 构建与安装/拖窗/任务栏验收仍须按 [VALIDATION.md](VALIDATION.md) 进行。

## 发布前检查

- `cargo fmt --check`、`cargo clippy --all-targets --locked -- -D warnings`、`cargo test --locked`。
- 只使用公开文件白名单打包；私有 PROJECT.md、日志、截图、备份、用户导入内容不入库。
- 对待提交 blob、历史和归档实际成员检查；为两个安装包生成 SHA-256。
- 上传前确认 Windows 实机验收已完成、源码提交与附件一致；不移动已有发布标签。
