# 0.1.2 验收清单

## 当前本地 build 32 验收范围（2026-10-16）

- R1–R35：扇形图标、固定提示、间距层级、清除按钮和固定命中区保持既有行为。
- R37：空结果面板由 `p.border` 改用侧栏同款 `p.line` 分隔灰度；rest 为 2% 状态色和 0.5pt，hover 仅 18% 和 0.75pt。
- 先完成自动测试、实际组件预览、打包身份与签名，再检查正式安装版的真实 hover、清除恢复和偏好／库保全；Windows 实机仍未验证。

当前自动结果：124 项测试通过、1 项 ignored，严格 Clippy/fmt/diff 与 8 项打包回归通过；60 张静态预览已复核，针对性测试覆盖新描边颜色、旋转边界、rest 正间隙和固定命中区。

## build32 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build32**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 真实窗口中面板外框为与左侧边栏一致的低对比分隔灰度，扇形图标和间距不变；面板 hover 保持克制，清除搜索恢复 53 项。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## build31 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build31**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 真实窗口中左右卡片呈相反小角度和高低差，图标共同旋转，中央搜索卡片水平；内部更分散，面板为中性弱描边。面板 hover 保持有效，清除搜索恢复 53 项。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## build30 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build30**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 真实窗口中静止卡片间距、固定提示和 32/16/24pt 层级清楚；面板 hover 与按钮直接 hover 均有内部反馈，Close 图标明确且无左色条。按住后移出未误清除，最终清除搜索恢复 53 项。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## build29 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build29**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 真实指针进入空结果面板后，后层卡片展开、主卡片与图标上浮放大、圆点外移；静止时恢复前层主卡片与收拢后层，文字和清除目标不动。清除搜索恢复 53 项列表。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## build28 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build28**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 大面板在实际窗口中留白充足，中央 88pt 图标和两个 60pt 辅助图标层级清楚；`wfwadafaf` 已融入“未找到与 query 匹配的脚本或插件”的完整说明，清除后恢复 53 项列表。
- 未悬停绿条为内高 52%–76%，只适度增强可见性，未恢复过高样式或改变命中区。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## build27 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build27**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 实际输入 `wdadawd` 后，搜索词单独一行显示，中心搜索图标与四个文件符号层级清楚；hover 有放大反馈，清除后恢复 53 项列表，未出现“新建文档”等无关动作。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## build26 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build26**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 快捷入口的真实 hover 帧显示整张卡片放大，绿条保持在内高 42%–68%，没有越过命中区；空结果卡片 hover 有放大反馈，清除后恢复 54 项列表。
- 明亮主题预览后通过“取消”恢复深灰，未保存偏好；重复导入现有脚本显示 `成功 0 · 跳过 0 · 相同 1 · 失败 0`，没有生成副本或覆盖原文件。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## build25 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build25**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 快捷入口整张卡片及图标、文字统一缩放，hover 时有亮边和轻微放大，移开恢复且未出现父层裁切；空结果卡片静止描边保持克制，辅助文字未越界，清除后恢复 54 项列表。
- 深灰→明亮→取消恢复最终正确，未保存偏好；重复导入现有脚本显示 `成功 0 · 跳过 0 · 相同 1 · 失败 0`，没有生成副本或覆盖原文件。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## build24 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build24**，codesign、版本和 payload 一致；临时旧副本和构建用额外 `.app` 已清理，仅保留正式入口。
- 空结果卡片在正式窗口居中且辅助文字未越界，hover 底色／边框与轻微放大正常，清除后恢复列表；快捷入口绿条加长且 hover 有亮边和轻微放大，移开恢复，命中目标未改变。
- 深灰→明亮→取消恢复最终正确，未保存偏好；重复导入现有脚本显示 `成功 0 · 跳过 0 · 相同 1 · 失败 0`，未生成副本或覆盖。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## 历史：build23 本机已执行结果（2026-10-16）

- 最终 DMG 已原位安装为 `/Applications/CavalryKit.app` **0.1.2 / build23**，codesign、版本和 payload 一致；仅有此正式运行入口。
- 现有脚本重复导入显示 `成功 0 · 跳过 0 · 相同 1 · 失败 0`，未生成副本或覆盖。空结果卡片为 360pt 宽、24pt 内边距、48pt 图标和 16/8pt 间距，清除后恢复列表；快捷图标不再叠加绿色底板，hover 与边框／绿条反馈正常。
- 深灰→明亮→取消恢复最终正确，未保存偏好；smoothstep 中途改向、精确终态与停止重绘由 motion 自动回归和 240 帧合成序列覆盖，不把离散截图当 GPU 计时证明。
- 偏好字节不变；Scripts/Plugins 原内容无增删改，`.DS_Store` 元数据未变化；Cavalry 宿主进程不变。Windows release 和全部测试 EXE 交叉生成，静态 GUI/x64/18 帧/PerMonitorV2/无外部 VC runtime 与 ZIP 八成员验证通过，未执行 Windows EXE。

## 历史：0.1.1 验收清单

### 当前本地 build 21 图标验收范围（2026-10-14）

- 只应用人类确认的D / V6图标，使用冻结源帧；旧build20金属边框为历史，不再是当前预期。
- 必须检查批准清单→规范PNG→ICO/ICNS→最终PE/安装.app的对应关系，且普通打包不得重新渲染旧样式。
- 本机Mac执行测试/签名/包与安装身份核对；原生主窗口、应用内品牌、Dock按实际可观察范围记录。
- Windows仅交叉构建、测试程序生成与静态资源/依赖/ZIP验证；原生Windows运行和DPI检查继续保留未验证。
- 公共合成预览使用虚构数据，不替代原生使用证据，也不包含用户真实库。

### build21 本机已执行结果

- macOS全目标/全特性测试117通过、1ignored；严格Clippy/fmt/diff通过。
- 批准V6的37规范帧、所有运行时PNG、ICO18帧、ICNS11块及最终PE/DMG内资源相符；安装payload与签名包相同。
- 正式安装路径已启动并观察主窗口和应用内图标；Dock读取接口曾超时，后续已通过用户提供的真实Dock截图核对V6外观，安装身份另行校验通过。设置交互因用户切换未继续，不修改用户偏好。
- 本机交叉构建的Windows release及全目标/全特性测试EXE编译通过；未在这台macOS上执行EXE。静态GUI/x64/18帧/PerMonitorV2/无外部VC运行时与ZIP五成员验证通过。
- 47张真实组件合成预览已生成，公共三图更新为V6；仅虚构数据，不是用户库截图。

## 分支 CI

`.github/workflows/desktop.yml` 在 macOS / Windows 托管运行器执行测试、平台打包和源码检查，Windows另执行安装器/回滚夹具。以具体提交的运行结论为准；首次Windows执行发现测试预期硬编码POSIX分隔符，已改成按本机路径构造预期，不改变安装逻辑。CI结果不代替真实Cavalry、图形交互、Dock/任务栏或混合DPI观察。

## 历史执行摘要（build 20，已被build21图标取代）

- build20 本地回归 **115 项通过、1 项真实用户目录写入测试未运行**；严格 Clippy、格式检查、帧生成和差异检查通过。
- macOS正式包完成签名、DMG/已安装字节检查；原生检查了无遮挡明亮主窗口、明亮四个设置页、深灰对照、两主题程序浏览取消与偏好恢复。
- 右侧行反馈的原生状态已观察；连续120帧为合成组件回归，不是原生帧率或时序基准。
- 原偏好及既有库校验保持一致；未测试真实卸载或宿主重启。Windows 本轮使用 `x86_64-pc-windows-gnu` + MinGW 交叉构建并通过 PE/ICO/ZIP 静态检查，仍未实机运行。

以下为可重复使用的完整检查表，不以本轮摘要将未执行场景一并勾选；逐项范围见UI_REVIEW与私有本地交接。

## Build 17 图标专项（2026-10-14）

本轮在 build16 的 111 项 UI/逻辑回归基础上增加 2 项图标资源回归，build17 合计 113 项通过；build19 增加一个已批准视觉锚点/Windows 资源哈希回归，该历史阶段合计 114 项通过。

- 已将用户确认的 A「石墨柔光」方向应用为 CavalryKit 品牌资源；主界面布局、设置、用户偏好、Cavalry 宿主和脚本/插件库不在本轮改动范围。
- Windows ICO 使用 18 个独立 PNG 帧（16/20/24/30/32/36/40/48/60/64/72/80/96/128/144/160/192/256），不由单个 256px 图像连续缩小；macOS 使用经 `iconutil` 可解码的 ICNS；应用内品牌图与窗口图分离。
- 浏览器定稿页的 2× 预览实际使用 16→32、24→48、32→64、48→96、64→128 资源；这是网页资源验证，不是 Windows 实机 DPI 验收。
- 已验证：源帧 hash、ICO/ICNS 容器解码、PE 资源帧数量/字节、macOS 签名/DMG；Windows 原生任务栏、资源管理器、快捷方式缓存和多 DPI 仍未实机验证。

## Build 18 macOS 图标安装补测（2026-10-14）

- 公开远端可读到的 latest release 仍是旧 0.1.0，因此没有下载它覆盖本地已确认的 build17 图标；使用本地 build18 DMG进行替换。
- 重新安装前后核对 DMG payload、签名、`CFBundleIconFile`、ICNS SHA-256；资源名改为内容寻址的 `CavalryKitIcon-cf28782cc147`，批准 ICNS 字节不变。
- 只定向更新 Launch Services 登记并清理已核对的构建目录/旧不存在镜像登记；没有删除全局图标数据库、重启 Finder/Dock 或修改系统设置。
- Spotlight 搜索 `CavalryKit` 复查为单一正式入口；CavalryKit重新打开并显示应用内品牌图标。Cavalry宿主 PID保持不变，偏好逐字节一致，Scripts52/Plugins208不变。
- macOS系统图标缓存仍属于系统呈现层；直接ICNS与系统返回图像的像素不要求逐字节相同。若其他系统入口仍旧，需记录为缓存/入口范围未验证，不把应用包资源校验冒充所有系统表面通过。

## Build 19 macOS A 方向还原（2026-10-14）

- 按用户反馈修正视觉漂移：`assets/icon-macos.png` 1024px 与第一轮已确认 A 预览字节完全相同；macOS ICNS 的小尺寸每档直接从此原始预览单次缩小。macOS 应用内品牌图另取同轮 A 应用稿。
- Windows ICO、Windows 应用内品牌图及其18帧源资源保持原 SHA-256；两平台不要混用同一 unpadded UI 图标。当前所有图标源/帧的 checksum 由构建测试校验。
- Mac 安装包资源和原生入口都要对照 A 预览，而不只看版本号/资源名；系统合成可有阴影/缩放差异，不能拿网页外层白色预览卡片当应用图标一部分。

## Build 20 银灰材质专项

- 首轮批准A图形源另存、hash核对，外层金属边框/阴影自绘；不将第三方C标志或其位图放进公开产物。
- Mac图标从新的材质主图分别导出ICNS帧，Windows以18个已批准光学尺寸为各自输入加边；监测小尺寸白色图形亮像素覆盖，避免边框挤掉识别特征。
- 静态包资源、安装版Finder和应用内图标分别观察；macOS系统合成与原始源图分开。Windows真实任务栏/Explorer和混合DPI未运行则必须保留范围限制。
- 实际结果：macOS `/Applications/CavalryKit.app` 为 build20，`CavalryKitIcon-8f6d9500666e` codesign/DMG/已安装 payload 一致；Windows ZIP SHA 为 `b178e100981c49fa3227d00cbba45928d0b64497a874fb339e160ee17d5d5b47`，静态检查报告在 `private/icon-metallic-bezel-20261014/windows-static-check.json`。

## 自动验证
- [ ] `cargo test --locked`（默认不运行真实目录 ignored 测试）
- [ ] `cargo clippy --all-targets --locked -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] 失败覆盖保持旧内容、重复同名输入、连续保留两者、重导入自身
- [ ] 后台 ZIP 临时目录在安装结束前保持有效，完成后清理
- [ ] 万条库仅绘制可视区，长错误弹窗不改变左栏尺寸
- [ ] 批准图形保留、Mac 可见底板/透明边距、11 个 ICNS 图标块与 18 个 ICO 尺寸
- [ ] Windows PE 逐帧资源校验、macOS codesign/DMG 校验

## Windows 后续实机验收（本轮已延期；静态检查不可替代）
1. Windows 10/11 x64 原生执行测试；记录 OS、GPU 与驱动版本。
2. 默认渲染器与 `CAVALRYKIT_RENDERER=glow` 都检查启动，不能启动时保留错误信息。
3. 连续拖动标题栏、缩放边框；在 100% / 125% / 150% / 200% DPI 以及双显示器之间移动。
4. 安装大型测试 ZIP 期间移动窗口和切换焦点，确认不出现长时间「未响应」。
5. 成功、部分失败、长路径失败、多个冲突、连续拖入多批：结果必须是独立弹窗，不能撑开导入区。
6. 桌面/开始菜单快捷方式、EXE、任务栏图标：导入前、导入后、重启后均检查新图标。
7. 旧版到固定路径安装；再重复安装 0.1.1：不出现第二个版本目录或不同快捷方式目标。
8. 文件锁/权限不足时覆盖：旧版本必须保留；关闭窗口操作不应中断正在执行的安装。
9. 打开/取消原生选择文件窗口；中文目录、中文名称；滚动大型库和搜索。
10. 合成脚本/插件卸载到回收站并手动恢复；不能误删用户的其他数据。

## macOS 必须实机完成
1. 从打包的 `.app` / DMG 启动，检查版本、新图标、中文和原生窗口。
2. 合成脚本导入、冲突跳过、长失败信息、结果关闭后尺寸不变。
3. 文件选择异步窗口打开/取消；搜索、类型切换、滚动、调整窗口大小。
4. 覆盖 Applications 原位置；推出旧版 DMG；再次启动确保实际路径与版本正确。
5. 测试前后比较既有 Cavalry 用户文件，确保除合成测试对象外没有改变。

只记录真正完成的项；未执行的平台、GPU 或场景必须留空。真实测试记录保存在私有本地交接目录，不附带用户工程发布。

## 原生动效 / 热区（沿用 build 16，build 17 未改动）

先测最终已安装包、真实鼠标/键盘和长时间空闲后再操作。搜索图标/空白点击应聚焦输入；导入卡片空白点击应打开选择器；取消不导入内容；设置开关标签/轨道都可切换；显示和卸载的目标不能相交。

开发诊断是显式 opt-in，不会默认写用户日志或开启持续重绘：

```sh
mkdir -p private/native-motion
CAVALRYKIT_MOTION_TRACE="$PWD/private/native-motion/trace.jsonl" cargo run --locked
```

只创建不存在的日志文件，最多5分钟/10万条、后台有界队列；文本只记录控件匿名ID/时间/值/事件数量，不含文件名或搜索内容。trace-only不发额外重绘请求。应用关闭后结束；同名日志不会被覆盖。

需要检查真实渲染帧时，在**另一轮**诊断中额外设置 `CAVALRYKIT_MOTION_FRAMES` 为已存在的私有目录。最多160帧，采集异步写PNG；截图读回可能影响帧时间，因此先做trace-only，不把采集速度当UI/GPU性能基准。截图可能包含真实库内容，不能加入源码或公开附件。debug构建可在编译时设置这些值以便专用诊断bundle；release只读取显式运行时环境，不包含本机诊断路径。

若参考软件是独立HUD窗口，设置页截图不等于HUD中间帧。记录实际看到的内容和无法捕获的部分，不制造“参考原生全面验收”结论。

本轮专项：按实际字形而非line-box检查搜索/导航/筛选居中；装饰三点不弹菜单，独立显示/卸载各自响应；入场中间颜色不出现禁用灰化；呼吸仅前台运行态且减少动效关闭时允许；Program Location浏览取消不写偏好。

Build16专项：在无遮挡明亮主界面检查脚本/插件计数、运行状态框与row-hover右侧图标/三点同步；菜单不是装饰三点行为。对照深灰设置取消/程序位置线性图标，保存主题前后校验其他偏好；最终恢复用户原主题。
