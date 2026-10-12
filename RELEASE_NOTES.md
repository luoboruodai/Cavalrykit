# CavalryKit 0.1.1

> Local release candidate / 本地待发布候选：尚未创建 v0.1.1 Release，等待界面确认与发布权限条件满足。

## 中文

按标注调整：品牌与状态收进左侧，导入卡片下面独立放置无省略号的文件按钮，四入口竖排；筛选旁为类型色计数，刷新与搜索同行。脚本绿、插件蓝，显示和卸载统一图标。导入区悬停为四边等宽细描边，图标/标题/格式说明协同过渡。


本轮补齐真实交互：整块导入卡片可点击并响应普通悬停；搜索图标/留白可聚焦，设置开关可点整行；扩大热区并分隔安全与删除动作。已用原生坐标输入与短时诊断复查，不再把合成GIF当真机动效证明。


本版更新双端性能、Cavalry 风格界面、可选安全重启和设置：
- 后台串行处理识别、解压、安装、扫描和卸载；缓存筛选、虚拟列表、分页结果。
- 深灰/绿色默认主题，明亮/自定义配色，四语言（中/英/日/韩），4/8/16/24pt 间距与右对齐弹窗操作。
- 恢复左侧拖入区、右侧列表筛选与搜索；删除重复标题与常驻冗余说明。
- 统一蓝灰深色/白色浅色材质、紧凑绿色按钮、分段筛选、弱边界搜索与轻卡片列表，避免元素各用一套风格。
- 动效分阶段：筛选指示线平滑移动、内容显现、拖入装饰小回弹、弹窗表面和正文先后入场；减少动效立即切换，动画首帧即阻挡背后操作。
- 简中/日/韩区域字形分别内嵌，并提供真实 Regular / Medium 字重；中英混排做缩放边界检查。
- 左侧品牌下方运行状态；安装后可选择重启/打开 Cavalry，也可稍后。只请求正常退出，保留保存提示，绝不强制终止进程。
- 设置重设计为四个分类页，主题以卡片预览，支持实时预览与取消、界面缩放、减少动效、返回自动刷新、重启提示策略、可选 Cavalry 路径。
- 安装覆盖暂存/回滚、同批重名冲突、递归目标/跨平台路径穿越保护。
- 修正 macOS 图标视觉大小，完整 Retina ICNS；Windows 15 档内嵌 ICO、静态 VC++ 运行库与固定路径升级安装器。

macOS 应用版本 **0.1.1 / build 12**。macOS 实际交互与代码自动测试、双端构建/资源检查已纳入发布验证，原生 CI 工作流已准备，但因远程发布权限限制未执行，不计为验证通过。

**验证边界**：Windows 真实 Cavalry、显卡/多屏拖窗、混合 DPI、任务栏缓存的人工验收仍延期；CI 不等价于这些体验测试。安全重启自动测试使用模拟进程，不对用户未保存工程发起退出。不能承诺所有环境绝对无 bug 或零卡顿。

## Downloads / 安装包
- `CavalryKit-0.1.1-mac-arm64.dmg` — macOS Apple Silicon
- `CavalryKit-0.1.1-win-x64.zip` — Windows x64，完整解压后运行 `Install-CavalryKit.cmd`
- `SHA256SUMS.txt` — 安装包 SHA-256

升级前退出 CavalryKit；macOS 替换原应用，Windows 使用固定路径安装器。旧便携副本不会自动删除。macOS 使用 ad-hoc 签名、未公证；Windows 没有商业代码签名证书。

## English

Annotated revision: brand/status in the left rail, soft drop surface plus a file action below, four stacked shortcuts, segmented filters with an adjacent kind-colored badge, and refresh beside search. Script rows use green; plugins use blue. Both row actions are icons. Drop hover has an even outline and coordinated icon/title/format feedback.


Native interaction revision: one full-card import target with ordinary hover feedback, whole-field search focusing, full-row switches and larger, separated action targets. Native coordinate-input traces and optional rendered-frame captures supplement tests; a synthetic GIF is not treated as runtime verification.


Performance and usability update: background I/O, virtualized library, bounded result dialogs, right-aligned actions, Cavalry-inspired dark/light/custom colors and Chinese/English/Japanese/Korean UI.

Settings support preview/cancel, scale, reduced motion, refresh-on-focus and restart policy. After successful installation, choose a normal Cavalry restart or do it later. Save your project first; cancellation, timeouts and multiple instances never trigger forced termination or duplicate launches.

Includes staged replacement/rollback, recursive-target and cross-platform archive-path safeguards, corrected macOS optical icon margins, 15 Windows ICO sizes, static VC++ runtime and an in-place per-user Windows upgrader.

The workspace retains the left-hand import area and right-hand list filters/search without duplicate headings. Unified blue-charcoal/white surfaces, compact green actions, segmented filters and quiet row cards replace heavy outlines. Finite, interruptible choreography includes eased indicator travel, decorative drop settling and staged modal/content reveal; background clicks are blocked immediately and reduced motion is instant. Settings now use four focused pages and visual theme cards. Regional SC/JP/KR fonts replace the shared Japanese UI subset. macOS bundle build: 12.

The native CI workflow is prepared but has not run remotely; it is not reported as passing. Physical Windows/Cavalry, GPU/mixed-DPI and taskbar behavior are still deferred; automated checks do not substitute for those tests. Both packages are unsigned for commercial distribution (macOS is ad-hoc signed without notarization).
