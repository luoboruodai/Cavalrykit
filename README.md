# CavalryKit(Cavalry 装备库)

Cavalry 脚本 / 插件快捷安装器,macOS 与 Windows 双平台。
Google Material Design 3 浅色界面,中文文案,即拖即装。

把 `.jsc / .js / .jsx` 脚本或压缩包拖进窗口,自动识别类型并安装到
Cavalry 的正确目录;支持已安装列表、搜索、卸载、冲突询问。

## 下载

| 平台 | 文件 | 说明 |
|---|---|---|
| macOS (Apple Silicon) | `CavalryKit-mac.dmg` | 拖入「应用程序」即可;未签名,首次打开需在「系统设置 → 隐私与安全性」允许 |
| Windows | `CavalryKit-win.zip`(内含 `CavalryKit.exe`) | 免安装,解压即用 |

## 功能

- **拖放安装**:脚本文件直接入库;压缩包自动解压识别
- **多压缩格式**:`.zip` / `.7z` / `.tar` / `.tar.gz`(`.tgz`)/ `.tar.bz2`(`.tbz2`)
  - 插件包(含 `definitions.json`)装入 `Third-Party/Plugins`,产品目录任意深度嵌套都能找到;包根直接是插件内容时以包名命名
  - 脚本包自动剥掉 GitHub 式单包裹壳,保留内部子目录分组,跳过 README/LICENSE
  - 混合包(同包既有脚本又有插件)两者都装,插件目录内部文件不误当脚本
- **智能冲突处理**:同名已存在时逐个询问 覆盖 / 跳过 / 保留两者(自动改名 `名称 2.jsc`)
- **已安装管理**:脚本与插件列表(插件显示作者)、搜索过滤、在文件管理器中显示、卸载(移入废纸篓/回收站,可恢复)
- **状态感知**:显示 Cavalry 是否运行;脚本即时生效,插件提示需重启 Cavalry
- **安全可靠**:压缩包路径穿越(zip-slip)防护;自动剔除 `__MACOSX`、`._*`、`.DS_Store`;卸载仅限 Cavalry 目录内路径

## 安装位置

| 平台 | 脚本 | 插件 |
|---|---|---|
| macOS | `~/Library/Application Support/Cavalry/Scripts` | `~/Library/Application Support/Cavalry/Third-Party/Plugins` |
| Windows | `%APPDATA%\Cavalry\Scripts` | `%APPDATA%\Cavalry\Third-Party\Plugins` |

## 开发

```bash
cargo test                          # 单元 + 功能测试(25 例)
cargo test -- --ignored             # 追加:真实环境往返(写真实 Cavalry 目录,自清理)
cargo run                           # 运行
cargo run --example gen_icon        # 重新生成图标(assets/icon.png)
```

## 打包

见 [packaging/README-打包.md](packaging/README-打包.md):

- macOS:`packaging/build-mac.sh`(产出 `.app` 与 `.dmg`)
- Windows:在 Windows 上运行 `packaging\build-win.bat`,或在 macOS 上用
  `cargo-xwin` 交叉编译:`rustup target add x86_64-pc-windows-msvc` 后
  `cargo xwin build --release --target x86_64-pc-windows-msvc`
