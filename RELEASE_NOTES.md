# CavalryKit 0.1.1 — 发布前验证候选 / Pre-release validation candidate

重点：Windows 阻塞操作移出 UI 线程、双端列表性能、独立导入结果弹窗、新图标与原位升级。
Highlights: background I/O, virtualized library, independent result dialogs, embedded Windows icon and stable-path updates.

## Packages
- `CavalryKit-0.1.1-mac-arm64.dmg`
- `CavalryKit-0.1.1-win-x64.zip`
- `SHA256SUMS.txt`

## Validation boundary
- macOS: automated regression tests, application import/result/search workflow and packaging verification.
- Windows: x64 cross-build and PE resource inspection. Native dragging, Explorer/taskbar icons, installer and mixed-DPI behavior remain to be tested on Windows.
- No claim of zero lag on every GPU, archive size, network location or operating-system configuration.
- Builds are not notarized/commercially code-signed.

**尚未发布正式版；Windows 实机验收通过后再推送、打标签和发布。**
**Do not publish this as a fully validated stable release until Windows native acceptance is recorded.**
