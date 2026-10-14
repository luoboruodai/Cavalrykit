# CavalryKit 0.1.1

2026-10-14 · macOS build 21 · Apple Silicon / Windows x64

## 简体中文

### 从 0.1.0 升级

0.1.1 是一次界面、响应性和安全交接更新。最终采用已确认的 D／V6 图标；不改变 Cavalry 工程格式，不执行导入脚本，也不迁移或删除现有用户库。

- 将识别、解压、复制、扫描、卸载、进程查询移出界面线程；缓存过滤结果、虚拟化列表，降低大型库重复布局开销。
- 恢复并统一左侧导入工作区、独立选择按钮、宿主状态和纵向快捷入口；右侧保留筛选与搜索，结果／失败信息使用独立有界弹窗。
- 四语界面、区域字体、混排文件名；深灰／明亮／自定义配色和即时设置预览，取消不落盘。
- 分阶段、可打断的筛选／列表／悬停反馈，固定命中区域与减少动效；空闲后停止逐帧重绘。
- 原生文件选择、同名冲突选择、覆盖暂存与恢复；Windows固定安装位置、旧EXE备份和快捷方式校验。
- 安装后可请求安全重启 Cavalry；保存提示、超时、多实例或取消时不强杀宿主。
- macOS Dock／窗口／应用内图标与Windows18档ICO全部使用批准资源；正常构建不再重画旧样式。
- 修复原生Windows测试中的路径分隔符预期，以及PowerShell 5.1打包中文文件名的UTF-8 BOM问题。
- 安装包附中英文离线指南；Release附件附SHA-256清单。

### 安装、保留设置与恢复

下载本页平台安装附件，不要用自动生成的Source code压缩包安装。macOS退出CavalryKit后替换「应用程序」中的同名.app；Windows完整解压后运行Install-CavalryKit.cmd。原位更新保留Cavalry用户库和CavalryKit偏好；恢复旧程序时先退出当前程序，仅恢复备份的程序文件，不覆盖较新的用户库或设置。

离线细节见安装包内 `INSTALL.zh-CN.txt` / `INSTALL.en.txt`。macOS为ad-hoc签名、未公证；Windows未商业签名。不会禁用Gatekeeper、SmartScreen或其他系统保护。

### 验证范围

发布标签对应提交须通过macOS／Windows托管CI：格式、Clippy、测试、平台打包、源码审计；Windows另测安装器原位升级、备份、文件锁与失败回滚。已验证批准PNG→ICO／ICNS→PE／安装.app资源链。本机macOS安装入口、主窗口与品牌图标已观察，Dock以用户提供的真实截图核对。

真实Windows Cavalry、GPU／多屏拖窗、混合DPI和任务栏缓存的人工体验仍未验收。CI不是这些场景的替代证据；不承诺所有环境无卡顿或无缺陷。

## English

### Upgrading from 0.1.0

0.1.1 updates the workspace, responsiveness, and safe installation workflow. It uses the final approved D/V6 artwork. It does not change Cavalry project formats, execute imported scripts, or migrate/delete existing user libraries.

- Moves identification, extraction, copying, scanning, removal, and process checks off the UI thread. Cached filtering and virtualized lists reduce repeated work for large libraries.
- Unifies the left-hand import workspace, separate file button, host status, and vertical shortcuts; preserves right-hand filters/search and places results/errors in bounded dialogs.
- Adds four-language settings, regional fonts, mixed-script filenames, dark/light/custom colors, and immediate preference previews that do not persist on Cancel.
- Provides staged, interruptible filter/list/hover feedback with stable hit targets and reduced motion. Frame-by-frame repainting stops after activity settles.
- Uses native file selection, explicit conflict choices, staged replacement/recovery, a fixed Windows install path, old-EXE backups, and checked shortcuts.
- Requests normal Cavalry restart after installation; save prompts, cancellation, timeout, or multiple instances never trigger a forced kill.
- Freezes approved macOS Dock/window/in-app artwork and all 18 Windows ICO sizes so normal builds cannot redraw rejected styles.
- Corrects Windows-native path expectations in tests and the UTF-8 BOM needed for Chinese filenames in Windows PowerShell 5.1 packaging.
- Includes Chinese/English offline guides and a SHA-256 list for release attachments.

### Installation, preferences, and recovery

Use the platform installation attachment, not GitHub's automatic Source code archive. On macOS, quit CavalryKit and replace the same-named app in Applications. On Windows, extract everything and run Install-CavalryKit.cmd. In-place updates preserve the Cavalry library and CavalryKit preferences. To restore an older program, quit the current one and restore only the backed-up program files—not newer user content or settings.

See `INSTALL.en.txt` / `INSTALL.zh-CN.txt` inside the package. macOS is ad-hoc signed and not notarized; Windows is not commercially signed. The tool does not disable Gatekeeper, SmartScreen, or other protections.

### Verification limits

The release-tag commit must pass hosted macOS/Windows formatting, Clippy, tests, platform packaging, and source audits. Windows additionally exercises in-place installation, backups, file locks, and rollback fixtures. Approved PNG→ICO/ICNS→PE/installed-app resources are checked. The local macOS main window/brand have been observed; Dock appearance is supported by a user-provided real screenshot.

Physical Windows/Cavalry, GPU/multiple-monitor dragging, mixed DPI, and taskbar-cache experience remain manually unverified. CI does not replace those checks or guarantee perfect performance in every environment.

## Attachments / 发布附件

- `CavalryKit-0.1.1-mac-arm64.dmg`
- `CavalryKit-0.1.1-win-x64.zip`
- `SHA256SUMS.txt`

Online references / 以下需要联网：
- Release: https://github.com/luoboruodai/Cavalrykit/releases/tag/v0.1.1
- Chinese guide: https://github.com/luoboruodai/Cavalrykit/blob/v0.1.1/README.md
- English guide: https://github.com/luoboruodai/Cavalrykit/blob/v0.1.1/README.en.md
- Changes since 0.1.0: https://github.com/luoboruodai/Cavalrykit/compare/v0.1.0...v0.1.1
