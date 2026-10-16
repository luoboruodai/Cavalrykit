# CavalryKit 0.1.2

2026-10-16 · macOS build 32 · Apple Silicon / Windows x64

<img src="https://raw.githubusercontent.com/luoboruodai/Cavalrykit/v0.1.2/assets/sponsor-wechat.svg" alt="微信赞助" width="232" height="64">

## 简体中文

### 从 0.1.1 升级

0.1.2 聚焦重复导入、空搜索状态与交互动效。它不改变 Cavalry 工程格式，不执行导入脚本，也不迁移或删除现有用户库。

- 同名脚本或插件在内容完全相同时自动标记为“相同”并跳过，不再生成“名称 2”副本；不同内容仍由用户选择覆盖、保留两者或跳过。脚本逐字节比较，插件比较完整目录树，读取失败不会静默跳过。
- 无搜索结果页在约 680×400pt 的大面板中让主搜索卡片与两侧 Script／Plugin 卡片在静止时就有间隙，左右卡片采用相反小角度和不同高低，图标随卡片旋转；面板外框改用与左侧边栏一致的分隔灰度。插画、标题、固定提示和按钮按 32/16/24pt 拉开，提示不再复述搜索词；“× 清除搜索”无左侧色条，命中区域不动。
- 四个快捷入口恢复 40pt 间距；整张卡片及图标、文字统一缩放，左侧绿条在内高 52%–76% 间生长提亮，所有中间帧均不裁切父层；图标不再叠加绿色底板，所有点击目标保持原位。
- 快捷卡片直接计算有界几何，列表仅在入场阶段应用视觉变换，稳定列表不再为每行重复设置恒等变换；这只是针对性渲染减负，不代表所有 GPU／系统卡顿均已消除。
- 设置中的配色切换使用约 0.3 秒的全局颜色过渡，取消设置也平滑恢复；采用 smoothstep 缓入缓出，开启减少动效时立即切换，并在过渡结束后停止重绘。
- 六种区域/字重 UI 字体子集已同步新增文案。

### 安装、保留设置与恢复

下载本页平台安装附件，不要用自动生成的 Source code 压缩包安装。macOS 退出 CavalryKit 后替换「应用程序」中的同名 .app；Windows 完整解压后运行 Install-CavalryKit.cmd。原位更新保留 Cavalry 用户库和 CavalryKit 偏好；恢复旧程序时先退出当前程序，仅恢复备份的程序文件，不覆盖较新的用户库或设置。

离线细节见安装包内 `INSTALL.zh-CN.txt` / `INSTALL.en.txt`。macOS 为 ad-hoc 签名、未公证；Windows 未商业签名。不会禁用 Gatekeeper、SmartScreen 或其他系统保护。

### 验证范围

本地候选检查包括 124 项自动测试（另有 1 项真实用户目录写入测试保持忽略）、严格 Clippy、格式检查、实际组件合成预览与连续帧。发布标签对应提交仍须通过 macOS／Windows 托管 CI 的格式、Clippy、测试、平台打包和源码审计；Windows 另测安装器与回滚夹具。

合成图片和连续帧只用于界面回归，不冒充原生交互录像。真实 Windows Cavalry、GPU／多屏拖窗、混合 DPI 和任务栏缓存的人工体验仍未验收，CI 不是这些场景的替代证据。

## English

### Upgrading from 0.1.1

Version 0.1.2 focuses on duplicate imports, the empty-search state, and interaction motion. It does not change Cavalry project formats, execute imported scripts, or migrate/delete existing user libraries.

- Same-named scripts and plugins are now marked Identical and skipped only when their contents match exactly, instead of creating “name 2” copies. Different contents still require Replace / Keep both / Skip. Scripts are compared byte-for-byte, plugins by their full directory trees, and read failures never skip silently.
- The no-results state gives the central search card and Script/Plugin cards real separation at rest, with opposite small side-card angles, different heights, and icons rotated to match. The panel outline now uses the sidebar's low-contrast divider gray. The 32/16/24pt content rhythm, fixed helper, and Close-icon Clear search remain without a left rail or moving hit regions.
- All four shortcuts retain 40pt spacing while the entire card, icon, and label scale on hover/focus without clipping the parent region. Rails grow between 52% and 76% of the inner height without green backplates, and click targets stay fixed.
- Shortcut geometry is computed directly and row visual transforms are limited to their entrance phase, avoiding repeated identity transforms for settled rows. This is targeted overhead reduction, not a claim that every GPU or system stall is eliminated.
- Appearance changes use a roughly 0.3-second smoothstep transition, including Cancel restoration. Reduced motion switches immediately, and repainting stops after the transition settles.
- All six regional/weight UI font subsets include the new strings.

### Installation, preferences, and recovery

Use the platform installation attachment, not GitHub's automatic Source code archive. On macOS, quit CavalryKit and replace the same-named app in Applications. On Windows, extract everything and run Install-CavalryKit.cmd. In-place updates preserve the Cavalry library and CavalryKit preferences. To restore an older program, quit the current one and restore only its backed-up program files—not newer user content or settings.

See `INSTALL.en.txt` / `INSTALL.zh-CN.txt` inside the package. macOS is ad-hoc signed and not notarized; Windows is not commercially signed. The tool does not disable Gatekeeper, SmartScreen, or other protections.

### Verification limits

The local candidate checks cover 124 automated tests (with one real-user-directory write test still ignored), strict Clippy, formatting, actual-component synthetic previews, and frame sequences. The release-tag commit must also pass hosted macOS/Windows formatting, Clippy, tests, platform packaging, and source audits; Windows additionally runs installer and rollback fixtures.

Synthetic images and frame sequences support UI regression checks; they are not native-interaction recordings. Physical Windows/Cavalry, GPU/multiple-monitor dragging, mixed DPI, and taskbar-cache experience remain manually unverified, and CI does not replace those checks.

## Attachments / 发布附件

- `CavalryKit-0.1.2-mac-arm64.dmg`
- `CavalryKit-0.1.2-win-x64.zip`
- `SHA256SUMS.txt`

Online references / 以下需要联网：
- Release: https://github.com/luoboruodai/Cavalrykit/releases/tag/v0.1.2
- Chinese guide: https://github.com/luoboruodai/Cavalrykit/blob/v0.1.2/README.md
- English guide: https://github.com/luoboruodai/Cavalrykit/blob/v0.1.2/README.en.md
- Changes since 0.1.1: https://github.com/luoboruodai/Cavalrykit/compare/v0.1.1...v0.1.2
