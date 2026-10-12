# CavalryKit 0.1.1 · Cavalry 装备库

本地 Cavalry 脚本 / 插件安装与管理工具，支持 **macOS（Apple Silicon）与 Windows x64**。
保留 Material Design 3 浅色中文界面；不上传导入文件、不自动执行脚本，也不修改系统脚本文件关联。

> **0.1.1 验证状态：** macOS 本地自动测试和应用交互已验证；Windows 已交叉编译并检查 EXE 资源，但仍需 Windows 实机验收。待验收完成再发布正式版，不将交叉编译等同于拖窗/显卡/任务栏体验验证。

## 0.1.1 更新

- **界面线程不再做耗时文件操作**：识别、解压、复制、扫描、卸载和进程检测由单一后台线程串行处理，避免操作互相覆盖；文件选择采用异步窗口。
- **Windows 针对性处理**：以系统进程枚举替代定时启动 `tasklist`；保留系统原生标题栏/边框；使用 wgpu 渲染后端，提供 OpenGL 兼容模式。
- **万条列表可视区绘制**：缓存过滤索引，仅绘制可见条目，长名称一次排版省略，不再每帧克隆整个库或反复逐字排版。
- **导入结果独立弹窗**：成功、跳过、失败、插件重启提示集中显示；长错误支持换行/滚动，不占据或撑宽导入区。
- **统一新图标**：窗口、应用内品牌、macOS ICNS、Windows EXE 资源使用同一图标来源。Windows 固定路径快捷方式显式引用 EXE 内嵌图标。
- **更安全的覆盖**：先完整暂存，再替换旧条目；暂存失败保留旧文件，替换失败尝试回滚。支持把已安装的脚本重新导入原位置，不会先删除源文件。
- **可重复升级**：Windows 提供固定用户安装目录的升级脚本；macOS 替换同一个 `CavalryKit.app`，不要并排保留不同版本应用。

## 安装与升级

发布包名称：

| 平台 | 文件 | 安装方式 |
|---|---|---|
| macOS Apple Silicon | `CavalryKit-0.1.1-mac-arm64.dmg` | 退出旧版，将 `CavalryKit.app` 拖到「应用程序」，选择替换，然后推出旧 DMG |
| Windows x64 | `CavalryKit-0.1.1-win-x64.zip` | 完整解压，关闭 CavalryKit，运行 `Install-CavalryKit.cmd` |

Windows 默认安装到 `%LOCALAPPDATA%\Programs\CavalryKit\CavalryKit.exe`，之后使用桌面/开始菜单快捷方式。
重复运行安装器会更新这一固定路径，不创建版本号目录；旧 EXE 压缩备份保存在 `%LOCALAPPDATA%\CavalryKit\Backups`。
手动解压在其他位置的旧便携版不会被擅自删除，请自行归档或移除；若旧任务栏固定图标仍指向其他路径，取消固定后从新版快捷方式重新固定。

macOS 包采用 ad-hoc 签名，**未经过 Apple 公证**；Windows EXE **没有商业代码签名证书**。首次运行可能出现系统安全提示；工具不会禁用系统保护。

### 使用教程

1. 启动 CavalryKit，将 `.jsc`、`.js`、`.jsx`、插件目录或受支持压缩包拖入窗口，也可以点击「选择文件…」。
2. 等待后台识别与安装；期间仍可查看窗口，队列中的其他输入会依次处理。
3. 同名冲突会询问「覆盖 / 保留两者 / 跳过」。一次导入中的多个同名目标也会询问。
4. 在结果弹窗查看各条目的成功、跳过或失败原因，点击「知道了」后继续处理下一批。
5. 新脚本可由 Cavalry 加载，插件需要重启 Cavalry。CavalryKit 本身不会运行导入的脚本。
6. 右侧支持按类型过滤、按名称/分组/作者搜索；外部修改后点击「刷新」更新库。
7. 「显示」定位到系统文件管理器；「卸载」经确认后移入废纸篓 / 回收站，卸载不允许直接删除整个库根目录。

**压缩包**：支持 `.zip`、`.7z`、`.tar`、`.tar.gz` / `.tgz`、`.tar.bz2` / `.tbz2`。自动识别含 `definitions.json` 的插件目录；混合包中的独立脚本与插件分别安装；过滤常见 macOS 压缩垃圾文件。只安装可信来源的内容。

| 平台 | 脚本目录 | 插件目录 |
|---|---|---|
| macOS | `~/Library/Application Support/Cavalry/Scripts` | `~/Library/Application Support/Cavalry/Third-Party/Plugins` |
| Windows | `%APPDATA%\Cavalry\Scripts` | `%APPDATA%\Cavalry\Third-Party\Plugins` |

### Windows 显卡兼容模式

默认使用 wgpu；旧显卡/驱动若无法启动，可在安装目录打开命令行：

```bat
set CAVALRYKIT_RENDERER=glow
CavalryKit.exe
```

此选项只更换绘图后端，不更换文件安装引擎。Windows 任务栏缓存、混合 DPI、多显示器与远程桌面仍需在真实环境验证，见 [验收清单](packaging/VALIDATION.md)。

## 开发与验证

```bash
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo run --locked
cargo run --locked --example import_icon  # 从 assets/icon.ico 更新窗口 PNG
```

默认测试使用合成数据与临时目录。`real_env_roundtrip` 默认忽略，因为它写入真实 Cavalry 目录；不要在不了解测试范围时运行所有 ignored 测试。

打包方法见 [打包说明](packaging/README-打包.md)，版本内容见 [CHANGELOG](CHANGELOG.md)。本地交接记录、测试日志、用户工程与备份不属于公开发布内容。

---

## English

**CavalryKit 0.1.1** is a local script/plugin installer and library manager for Cavalry, targeting macOS Apple Silicon and Windows x64. It keeps a Chinese Material Design 3 interface.

### What changed

- Serialized background I/O for extraction, copying, library scans, uninstalling and process checks; asynchronous file picker.
- Native Windows process enumeration instead of polling `tasklist`, native window decorations, wgpu rendering with an optional OpenGL compatibility mode.
- Cached filtering and virtualized rows; bounded, scrollable result dialogs that do not resize the import area.
- A shared icon source for the window, macOS bundle and embedded Windows EXE resources.
- Staged replacement with rollback protection, including safe self-import and same-batch conflict handling.
- A per-user Windows installer/upgrader with a stable application path and explicit shortcut icon target.

### Quick start

On macOS, quit the old app, replace `CavalryKit.app` in Applications, and eject the old disk image. On Windows, extract the entire ZIP and run `Install-CavalryKit.cmd`, then use the desktop/Start menu shortcut. Old portable copies elsewhere are not automatically deleted.

Drag scripts, plugin folders or supported archives into the window. Resolve each conflict, inspect the result dialog, and restart Cavalry for newly installed plugins. Search or filter the library; use Refresh after external changes. Uninstall moves an item to the system trash/recycle bin.

**Validation boundary:** automated tests and native macOS interaction are covered. The Windows build is cross-compiled and its icon, manifest and version resources are checked statically; Windows runtime, dragging, mixed-DPI and taskbar behavior still require native testing before a stable release. Neither platform build has a paid distribution-signing certificate; macOS uses ad-hoc signing without notarization.

For older GPU drivers, set `CAVALRYKIT_RENDERER=glow` before launching. No imported scripts are executed by this tool and no imported content is uploaded.

For learning purposes only. Not affiliated with Scene Group.
