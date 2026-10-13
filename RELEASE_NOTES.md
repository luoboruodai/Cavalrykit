# CavalryKit 0.1.1

> Local release candidate / 本地待发布候选：尚未创建 v0.1.1 Release，等待界面确认与发布权限条件满足。

## 中文

**build17最新调整**：应用已确认的 A「石墨柔光」图标方向；保留上传图形构图，独立重建可缩放线条并对 Windows 小尺寸做像素覆盖/对齐修正。Windows ICO 改为18档独立资源，macOS ICNS与应用内品牌图分开处理。正式包应用前保留源帧、hash和用户库/偏好基线，Windows实机DPI与系统缓存仍需单独验收。

**build16最新调整**：行右侧显示/卸载/装饰三点同步响应整行悬停，热区保持不动；程序位置统一为线性图标。以深灰形式为锚点，专项调整明亮模式取消按钮、结果计数和运行状态框的灰度/材质，保留焦点与语义色。

**延续build15的调整**：独立圆角侧栏、字形光学居中、搜索/刷新与卡片宽度对齐；恢复显示/卸载图标，三点只做弱灰装饰。入场不再叠加disabled灰化；完整内容字体统一中英文件名基线（安装包会增大）。运行绿点在前台轻呼吸，减少动效/后台暂停。设置保存去图标、重置去底板，安全说明降为小字，程序位置页统一路径/浏览/图标。

**此前build14（其中三点菜单/保存图标由build15替代）**：三点操作菜单（卸载仍确认）、整卡逐个向上入场/悬停外扩、搜索字形居中与16条偶发提示、带底板刷新、品牌同行状态徽章、四个绿色快捷入口、较清晰的导入表面和同文件动作样式的保存按钮。默认提示占20秒、短提示4秒，输入/失焦/减少动效不轮换；设置入场/退出支持中途取消及重新播放。

按标注调整：品牌与状态收进左侧，导入卡片下面独立放置无省略号的文件按钮，四入口竖排；筛选旁为类型色计数，刷新与搜索同行。脚本绿、插件蓝，显示和卸载现在为独立图标，三点仅装饰。导入区悬停为四边等宽细描边，图标/标题/格式说明协同过渡。


此前build14/15补齐的真实交互：整块导入卡片可点击并响应普通悬停；搜索图标/留白可聚焦，设置开关可点整行；扩大热区并分隔安全与删除动作。已用原生坐标输入与短时诊断复查，不再把合成GIF当真机动效证明。


本版更新双端性能、Cavalry 风格界面、可选安全重启和设置：
- 后台串行处理识别、解压、安装、扫描和卸载；缓存筛选、虚拟列表、分页结果。
- 深灰/绿色默认主题，明亮/自定义配色，四语言（中/英/日/韩），4/8/16/24pt 间距与右对齐弹窗操作。
- 恢复左侧拖入区、右侧列表筛选与搜索；删除重复标题与常驻冗余说明。
- 统一蓝灰深色/白色浅色材质、紧凑绿色按钮、分段筛选、弱边界搜索与轻卡片列表，避免元素各用一套风格。
- 动效分阶段：筛选指示线平滑移动、内容显现、拖入装饰小回弹、弹窗表面和正文先后入场；减少动效立即切换，动画首帧即阻挡背后操作。
- 简中/日/韩区域字形分别内嵌，并提供真实 Regular / Medium 字重；中英混排做缩放边界检查。
- 左侧品牌同行状态徽章；安装后可选择重启/打开 Cavalry，也可稍后。只请求正常退出，保留保存提示，绝不强制终止进程。
- 设置重设计为四个分类页，主题以卡片预览，支持实时预览与取消、界面缩放、减少动效、返回自动刷新、重启提示策略、可选 Cavalry 路径。
- 安装覆盖暂存/回滚、同批重名冲突、递归目标/跨平台路径穿越保护。
- 修正 macOS 图标视觉大小，完整 Retina ICNS；Windows 18 档内嵌 ICO、静态 VC++ 运行库与固定路径升级安装器。

macOS 应用版本 **0.1.1 / build 17**。macOS 实际交互与代码自动测试、双端构建/资源检查已纳入发布验证，原生 CI 工作流已准备，但因远程发布权限限制未执行，不计为验证通过。

**验证边界**：Windows 真实 Cavalry、显卡/多屏拖窗、混合 DPI、任务栏缓存的人工验收仍延期；CI 不等价于这些体验测试。安全重启自动测试使用模拟进程，不对用户未保存工程发起退出。不能承诺所有环境绝对无 bug 或零卡顿。

## Downloads / 安装包
- `CavalryKit-0.1.1-mac-arm64.dmg` — macOS Apple Silicon
- `CavalryKit-0.1.1-win-x64.zip` — Windows x64，完整解压后运行 `Install-CavalryKit.cmd`
- `SHA256SUMS.txt` — 安装包 SHA-256

升级前退出 CavalryKit；macOS 替换原应用，Windows 使用固定路径安装器。旧便携副本不会自动删除。macOS 使用 ad-hoc 签名、未公证；Windows 没有商业代码签名证书。

## English

**Build16 refinement**: row-end icons and decorative dots share row-hover motion without moving click targets. Program Location uses a consistent line icon. Light secondary buttons, count badges and status surfaces follow the dark theme's geometry/hierarchy with tuned neutral materials, contrast and focus states.

**Build15 refinement**: inset rounded rail, optical ink alignment, equal search/list widths, repaired refresh arrow, separate reveal/remove icons and decorative-only dots. Reveal no longer applies disabled tint; a complete content face aligns mixed-script filenames (with a larger package). Foreground-only status breathing pauses in reduced motion/background. Save, reset, safety copy and Program Location share the refined styling.

**Earlier build14 (menu/Save-icon details superseded above)**: a three-dot action menu with removal confirmation, staggered whole-card upward reveal and expanding hover, centered search with sixteen occasional localized hints, a refresh tile, inline status badge, four green shortcuts, a clearer drop surface and soft-tinted Save. The default hint occupies 20s and a suggestion 4s; focus/input/reduced motion pauses rotation. Modal entrance/exit can be interrupted and replayed.

Annotated revision: brand/status in the left rail, soft drop surface plus a file action below, four stacked shortcuts, segmented filters with an adjacent kind-colored badge, and refresh beside search. Script rows use green; plugins use blue. Both row actions are separate icons; the trailing dots are decorative. Drop hover has an even outline and coordinated icon/title/format feedback.


Earlier build14/15 native interaction revision: one full-card import target with ordinary hover feedback, whole-field search focusing, full-row switches and larger, separated action targets. Native coordinate-input traces and optional rendered-frame captures supplement tests; a synthetic GIF is not treated as runtime verification.


Performance and usability update: background I/O, virtualized library, bounded result dialogs, right-aligned actions, Cavalry-inspired dark/light/custom colors and Chinese/English/Japanese/Korean UI.

Settings support preview/cancel, scale, reduced motion, refresh-on-focus and restart policy. After successful installation, choose a normal Cavalry restart or do it later. Save your project first; cancellation, timeouts and multiple instances never trigger forced termination or duplicate launches.

Includes staged replacement/rollback, recursive-target and cross-platform archive-path safeguards, corrected macOS optical icon margins, 18 Windows ICO sizes, static VC++ runtime and an in-place per-user Windows upgrader.

The workspace retains the left-hand import area and right-hand list filters/search without duplicate headings. Unified blue-charcoal/white surfaces, compact green actions, segmented filters and quiet row cards replace heavy outlines. Finite, interruptible choreography includes eased indicator travel, decorative drop settling and staged modal/content reveal; background clicks are blocked immediately and reduced motion is instant. Settings now use four focused pages and visual theme cards. Regional SC/JP/KR fonts replace the shared Japanese UI subset. macOS bundle build: 16.

The native CI workflow is prepared but has not run remotely; it is not reported as passing. Physical Windows/Cavalry, GPU/mixed-DPI and taskbar behavior are still deferred; automated checks do not substitute for those tests. Both packages are unsigned for commercial distribution (macOS is ad-hoc signed without notarization).
