# Product

<!-- impeccable:product-schema 1 -->

## Platform

macOS / Windows（native 桌面应用，Rust + egui/wgpu，非 web）

## Users

面向分发的本地工具:国内 Cavalry(Scene Group 动效设计软件)用户/动效设计师,在 macOS 或 Windows 上工作;使用者可能不熟命令行,需要即拖即装、状态可视、出错可恢复。自用之外会被分享,需要引导与容错。

## Product Purpose

把 Cavalry 的脚本(.jsc/.js/.jsx)与插件(.zip)装进本地 Cavalry 应用支持目录,并管理已装内容(查看、搜索、卸载)。成功 = 用户不打开终端、不找目录也能装好一条脚本链。

## Positioning

拖入即装 + 冲突逐个询问 + 卸载走废纸篓可还原 —— 一个本地安装器而非包管理器;竞品(手动拷文件)做不到的是「冲突决策可视化」和「一键还原」。

## Operating Context

macOS / Windows 桌面;与 Cavalry 并存;用户常从聊天/网盘拿到 .jsc/.zip 后直接拖入;Cavalry 运行状态影响「插件需重启」提示。

## Capabilities and Constraints

- 支持脚本与压缩包(zip / 7z / tar / tar.gz / tar.bz2)的识别、解压、去包装壳、冲突处理(覆盖/跳过/保留两者)
- 插件包任意深度嵌套识别;包根即插件内容时以包名命名;混合包脚本插件都装
- 卸载移入废纸篓(可还原),目录直达(访达)
- 现有 egui 实现,逻辑/功能/后台任务/布局回归测试,release 签名打包(packaging/build-mac.sh)
- 分发:macOS .dmg(Apple Silicon)/ Windows .zip(cargo-xwin 交叉编译,GUI 子系统)
- 显式未决：Windows 实机拖窗、混合 DPI、任务栏和安装器验收已延期；交叉编译不能替代。本轮界面重设计完成后先展示供用户确认，再推进发布；原生 CI 尚未远程执行。

## Brand Commitments

名称 CavalryKit(CK);仅供学习用途、与 Scene Group 无关(免责声明需保留在关于弹窗)

## Evidence on Hand

合成脚本/插件夹具与私有本机验证记录；用户库、工程与真实截图不进入公开仓库。

## Product Principles

1. 任务第一:界面即清单,每个元素服务于「装 / 查 / 卸」三个动作
2. 状态可视:Cavalry 运行态、装完结果、冲突待办都不藏
3. 可恢复:卸载入废纸篓,冲突逐个询问,不做不可逆动作
4. 分发就绪:陌生人也能看懂,不依赖文档

## UI Direction (0.1.1 redesign)

保留左侧拖入区与目录入口、右侧列表顶部的类型筛选和搜索；不将类型筛选移到侧栏，不重复展示 Cavalry 装备库标题。参考同类本地工具的分层卡片、细描边、统一圆角和有限缓动，但不复制第三方布局、品牌或素材。保持 CavalryKit 绿色强调色。

深色和浅色分别设计表面层次；按钮点击区域固定，悬停/按下只改变内部绘制，短时动画结束后停止重绘。「减少界面动效」立即切换，不引入弹跳、持续波纹、列表整页缩放或云端依赖。设置保留四分类页和右对齐动作组。

使用 4/8/16/24 逻辑间距和 Nielsen 十项检查，不把它们包装成平台认证。详见 packaging/UI_REVIEW.md。
