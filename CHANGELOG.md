# Changelog

## 0.1.1 — 2026-10-12 (local validation candidate)

### Performance / 性能
- Move file I/O, archive preparation and installation to a serialized background worker.
- Replace Windows tasklist subprocess polling with native ToolHelp process enumeration.
- Use wgpu with native window decorations; retain opt-in Glow compatibility mode.
- Cache filtering, virtualize list rows and replace repeated character-by-character ellipsis shaping.
- Repaint when results arrive; idle process probes only trigger a redraw when the status changes.

### UI / 界面
- Present all import results in a bounded modal dialog; keep the import rail size unchanged.
- Clip independent columns and bound long labels/details; support small windows.
- Update the shared app icon and add accessible labels to custom buttons.
- Add explicit Refresh and prevent closing during an active I/O operation/file picker.

### Reliability / 可靠性
- Stage copies on the destination filesystem before replacing; preserve/restore old data on failure.
- Handle duplicate destinations and repeated keep-both choices inside one batch.
- Give same-named archives independent extraction directories.
- Avoid symlink recursion while scanning, bound metadata parsing and prohibit root-library uninstall.
- Statically link the Windows VC++ runtime and preserve spaced build paths.
- Embed Windows icon/version/DPI resources; add per-user stable-path installer/upgrader.
- Rebuild macOS ICNS every time, verify ad-hoc signatures and generate versioned packages.

### Verification / 验证
- Synthetic tests cover headless layout, 10,000-entry list virtualization, worker temp-file lifetime, icon pixels and failed overwrites.
- macOS native import/result/search interaction is checked locally.
- Windows cross-build and static PE checks do **not** replace native Windows runtime acceptance; stable publishing is pending that check.

## 0.1.0

Initial dual-platform installer, Material Design 3 UI, archive support and functional tests.
