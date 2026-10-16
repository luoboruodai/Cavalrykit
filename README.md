# CavalryKit 0.1.2

[English](README.en.md) · 简体中文

本地 Cavalry 脚本／插件安装与管理工具，支持 **macOS Apple Silicon、Windows x64**，界面支持简体中文、English、日本語、한국어。不会上传导入内容，也不会执行导入的脚本。

<img src="assets/sponsor-wechat.svg" alt="微信赞助" width="232" height="64">

![主界面，合成示例数据](assets/screenshots/main.png)

<details><summary>明亮主题：同一布局</summary>

![明亮主题，合成示例数据](assets/screenshots/main-light.png)

</details>

## 下载与从 0.1.1 升级

从 [v0.1.2 Release](https://github.com/luoboruodai/Cavalrykit/releases/tag/v0.1.2) 下载对应平台的**安装附件**，不要把 GitHub 自动生成的 Source code 包当作安装包。

| 平台 | 附件 | 安装／升级 |
|---|---|---|
| macOS Apple Silicon | `CavalryKit-0.1.2-mac-arm64.dmg` | 退出 CavalryKit，将 `CavalryKit.app` 拖到「应用程序」，替换同名应用，再推出镜像 |
| Windows x64 | `CavalryKit-0.1.2-win-x64.zip` | 完整解压，关闭 CavalryKit，运行 `Install-CavalryKit.cmd` |
| 完整性校验 | `SHA256SUMS.txt` | 核对下载文件的 SHA-256；不要仅依赖文件名 |

两个安装包均附 **`INSTALL.zh-CN.txt`、`INSTALL.en.txt` 和双语 `RELEASE_NOTES.md`**，离线即可查看安装、升级与恢复步骤。

- **保留数据**：原位升级不迁移或删除 Cavalry 的 Scripts／Plugins，也不重置 CavalryKit 偏好。升级前仍建议备份自己的库与设置。
- **macOS**：统一入口 `/Applications/CavalryKit.app`。不要保留改名的运行副本，以免启动旧版。0.1.2 的内部构建号为 **32**。
- **Windows**：固定入口 `%LOCALAPPDATA%\Programs\CavalryKit\CavalryKit.exe`。桌面／开始菜单快捷方式指向该路径；更新会备份旧 EXE 至 `%LOCALAPPDATA%\CavalryKit\Backups`。其他位置的便携副本不会被擅自删除。旧任务栏图标可取消固定后从新快捷方式重新固定。
- **签名说明**：macOS 使用 ad-hoc 签名，未公证；Windows 没有商业代码签名。系统可能提示无法验证开发者，程序不会关闭系统保护。

校验示例：macOS 在下载目录执行 `shasum -a 256 CavalryKit-0.1.2-mac-arm64.dmg`，与清单对应行比较；Windows 执行 `Get-FileHash .\CavalryKit-0.1.2-win-x64.zip -Algorithm SHA256`，与校验文件的对应行比较。

## 0.1.1 → 0.1.2 的主要变化

- **重复导入更安静**：同名脚本／插件仅在内容完全相同时自动标记「相同」并跳过，不再产生「名称 2」；不同内容仍保留覆盖／保留两者／跳过。
- **空搜索状态更开阔自然**：约 680×400pt 大面板中，主搜索卡片与两侧 Script／Plugin 卡片在静止时已有间隙，左右卡片带相反小角度和高低差，图标随卡片旋转；面板外框改用与左侧边栏一致的分隔灰度。插画、标题、固定提示和按钮按 32/16/24pt 分层，提示不复述查询；「× 清除搜索」无左侧色条，命中区域不动。
- **侧栏反馈更完整**：四个快捷入口恢复 40pt 间距，整张卡片及图标、文字统一缩放，左侧绿条在内高 52%–76% 间生长提亮，所有中间帧均不裁切父层；图标不再叠加绿色底板，所有点击目标保持原位。
- **稳定列表渲染减负**：快捷卡片直接计算有界几何，列表仅在入场阶段应用视觉变换，稳定列表不再为每行重复设置恒等变换；这不代替具体 GPU／系统卡顿排查。
- **全局配色平滑切换**：设置中的主题变化与取消恢复采用 smoothstep 缓入缓出，约 0.3 秒完成；减少动效时立即切换，稳定后停止重绘。
- **字体资源同步**：六种区域／字重 UI 子集补齐新增状态文案。

## 使用方法

1. 拖入 `.jsc`／`.js`／`.jsx`、插件文件夹或受支持压缩包，或点击「选择文件」。只安装可信来源的内容。
2. 同名冲突选择覆盖／保留两者／跳过，也可取消本批。覆盖前暂存，失败恢复；危险操作不会由默认 Enter 误接受。
3. 在独立弹窗查看成功、跳过或失败，必要时展开技术详情。
4. 插件安装后如需重启，**先保存 Cavalry 工程**。工具只请求正常退出；如果 Cavalry 显示保存提示，请在那里处理。「停止等待」不会撤回已经发给宿主的退出请求。
5. 库中可筛选类型、按名称／分组／作者搜索；「显示」定位文件，「卸载」经确认移入废纸篓／回收站。
6. 外部修改库后点击刷新，或启用返回窗口时自动刷新。

支持 ZIP、7z、TAR、TAR.GZ／TGZ、TAR.BZ2／TBZ2。含 `definitions.json` 的目录视为插件；插件内部 `.js` 不会误装成独立脚本。过滤 macOS 元数据并拒绝归档路径穿越。

### 设置与快捷键

![设置，合成示例数据](assets/screenshots/settings.png)

- 界面语言：中／英／日／韩；不修改文件名称和脚本内容。
- 配色：Cavalry 深灰、明亮、自定义；界面缩放 90%–125%，可减少动效。
- 安装：重启提示范围、返回窗口自动刷新；可指定 Cavalry 程序位置。
- 偏好位于系统配置目录的 `CavalryKit/preferences.json`。修改先预览，保存后保留；写入失败不覆盖原文件。

`Cmd/Ctrl+F` 搜索 · `Cmd/Ctrl+O` 选择文件 · `Cmd/Ctrl+R` 刷新 · `Cmd/Ctrl+,` 设置 · `Esc` 取消／关闭弹窗。

| 平台 | 脚本目录 | 插件目录 |
|---|---|---|
| macOS | `~/Library/Application Support/Cavalry/Scripts` | `~/Library/Application Support/Cavalry/Third-Party/Plugins` |
| Windows | `%APPDATA%\Cavalry\Scripts` | `%APPDATA%\Cavalry\Third-Party\Plugins` |

### 显卡兼容模式

若旧显卡驱动无法初始化默认渲染器，可在 Windows 命令提示符中执行：

```bat
set CAVALRYKIT_RENDERER=glow
CavalryKit.exe
```

这不是性能保证，也不替代驱动与硬件问题排查。

## 验证范围

- 本地测试覆盖安装／解包、失败恢复、任务状态、偏好、多语言字体、布局、图标与安全重启；默认只用临时目录／合成数据。真实用户目录写入测试保持忽略。
- [Desktop checks](https://github.com/luoboruodai/Cavalrykit/actions/workflows/desktop.yml) 在 macOS 和 Windows 运行格式、Clippy、测试、打包与公开源码审计；Windows 另有安装器／回滚夹具。以 Release 标签对应提交的结果为准。
- macOS 正式安装版主窗口、品牌图标已观察，Dock 以用户提供的真实截图核对；图标资源另作字节验证。
- **真实 Windows Cavalry、GPU／多屏拖窗、混合 DPI、任务栏缓存的人工体验仍未验收。** 托管 CI 不是这些场景的替代证据。合成界面图与连续帧也不冒充原生交互录像。

## 构建与项目资料

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
python3 -m unittest discover -s packaging -p 'test_release_packaging.py'
cargo run --locked --example import_icon -- --check
```

图标输入冻结于 `assets/icon-design.json`；`scripts/render_platform_icons.py` 只校验／组装批准帧，不重新绘制旧图案。UI 预览功能不进入普通 release。源码包不包含用户库、私有日志、批准记录或客户素材。

- [双语发布说明](RELEASE_NOTES.md)
- [更新历史](CHANGELOG.md)
- [打包说明](packaging/README-打包.md)
- [验证清单](packaging/VALIDATION.md) · [UI 复查](packaging/UI_REVIEW.md)
- [第三方许可](packaging/THIRD-PARTY-NOTICES.txt)
