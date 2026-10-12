# CavalryKit 0.1.1

## 中文

本版更新双端性能、Cavalry 风格界面、可选安全重启和设置：
- 后台串行处理识别、解压、安装、扫描和卸载；缓存筛选、虚拟列表、分页结果。
- 深灰/绿色默认主题，明亮/自定义配色，四语言（中/英/日/韩），4/8/16/24pt 间距与右对齐弹窗操作。
- 标题同行运行状态；安装后可选择重启/打开 Cavalry，也可稍后。只请求正常退出，保留保存提示，绝不强制终止进程。
- 设置支持实时预览与取消、界面缩放、减少动效、返回自动刷新、重启提示策略、可选 Cavalry 路径。
- 安装覆盖暂存/回滚、同批重名冲突、递归目标/跨平台路径穿越保护。
- 修正 macOS 图标视觉大小，完整 Retina ICNS；Windows 15 档内嵌 ICO、静态 VC++ 运行库与固定路径升级安装器。

macOS 应用版本 **0.1.1 / build 4**。macOS 实际交互与代码自动测试、双端构建/资源检查已纳入发布验证，原生 CI 结果见仓库 Actions。

**验证边界**：Windows 真实 Cavalry、显卡/多屏拖窗、混合 DPI、任务栏缓存的人工验收仍延期；CI 不等价于这些体验测试。安全重启自动测试使用模拟进程，不对用户未保存工程发起退出。不能承诺所有环境绝对无 bug 或零卡顿。

## Downloads / 安装包
- `CavalryKit-0.1.1-mac-arm64.dmg` — macOS Apple Silicon
- `CavalryKit-0.1.1-win-x64.zip` — Windows x64，完整解压后运行 `Install-CavalryKit.cmd`
- `SHA256SUMS.txt` — 安装包 SHA-256

升级前退出 CavalryKit；macOS 替换原应用，Windows 使用固定路径安装器。旧便携副本不会自动删除。macOS 使用 ad-hoc 签名、未公证；Windows 没有商业代码签名证书。

## English

Performance and usability update: background I/O, virtualized library, bounded result dialogs, right-aligned actions, Cavalry-inspired dark/light/custom colors and Chinese/English/Japanese/Korean UI.

Settings support preview/cancel, scale, reduced motion, refresh-on-focus and restart policy. After successful installation, choose a normal Cavalry restart or do it later. Save your project first; cancellation, timeouts and multiple instances never trigger forced termination or duplicate launches.

Includes staged replacement/rollback, recursive-target and cross-platform archive-path safeguards, corrected macOS optical icon margins, 15 Windows ICO sizes, static VC++ runtime and an in-place per-user Windows upgrader.

Native CI results are in Actions. Physical Windows/Cavalry, GPU/mixed-DPI and taskbar behavior are still deferred; automated checks do not substitute for those tests. Both packages are unsigned for commercial distribution (macOS is ad-hoc signed without notarization).
