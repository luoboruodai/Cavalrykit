# Changelog

## 0.1.1 — 2026-10-12

### UI and settings / 界面与设置
- Integrate brand/status into the left rail; put the file action below the soft drop surface, without ellipsis.
- Stack four shortcut cards vertically with consistent type-colored accents.
- Use green scripts / blue plugins across row icons, accent bars, hover outlines and count badges.
- Match reveal/remove as two icon actions, keeping separate hit targets and safe spacing.
- Use adaptive segmented filters and an adjacent result badge; place refresh beside search.
- Coordinate import title/format/icon feedback; use an even drop outline without a thick left stripe.

- Make the entire left import card a single actionable target with ordinary pointer feedback; file-drag feedback is a separate state.
- Focus search from its icon/padding, keep clear separate, use full-row animated switches and isolate removal targets from safe actions.
- Increase action/search/switch-row targets to 40/44/48 logical units; add edge-click, keyboard, long-idle hover and overlap regressions.
- Add explicit, bounded native trace-only diagnostics and optional window-frame capture; remove the old public synthetic GIF as native verification evidence.

- Restore the familiar left import area and right-hand list filters/search; remove duplicate library headings, navigation captions and the permanent policy footer.
- Apply a unified blue-charcoal / white surface system, compact green actions with restrained depth, segmented filters, quiet search and borderless row cards.
- Replace the generic bordered-panel look while keeping the agreed layout and readable light/dark states.
- Coordinate interruptible eased filter travel, row/page content reveal, a small decorative drop settle and staged modal entrance.
- Block background actions from the very first modal frame; respect reduced motion and stop repainting when settled.
- Provide a reproducible actual-component motion renderer (synthetic data and demo input only).
- Replace overlapping folder/plugin icon strokes with bounded single outlines; keep small checkbox corners distinct from larger button corners.
- Split settings into General / Appearance / Installation / Cavalry pages with visual theme previews.
- Change the Chinese search hint to “搜索插件或脚本名称”; keep helper text in normal wrapped flow instead of a fixed status slot.
- Use separate SC/JP/KR Regular and Medium UI font subsets; verify mixed-script ink bounds and all translated glyphs.
- Add Cmd/Ctrl+F, accessible search labeling and contextual removal labels; keep main/footer actions within small-window bounds.
- Cavalry-inspired dark/light/custom palette with readable text across extreme custom colors.
- Verify the actual rendered library scroll thumb, not just palette tokens; remove a local foreground override that made the idle thumb too dim.
- Four UI languages (Chinese, English, Japanese, Korean) and regional OFL font subsets for consistent glyph coverage.
- Right-aligned modal actions, fixed-height footers, 4/8/16/24pt spacing, same-row status and visible keyboard focus.
- Preview/cancel/save preferences, scale, reduced motion, refresh-on-focus and restart policy.
- Optional safe Cavalry restart/open after success; cancellation/timeout/multiple instances never force-close or duplicate-launch.
- Preserve prior input failures when cancelling a conflict batch; fix search clear-button overflow and localized row-action sizing.
- Prepare native macOS/Windows CI and isolated Windows installer/shortcut/rollback tests (remote jobs not yet run); physical Windows acceptance remains deferred.

### Performance / 性能
- Move file I/O, archive preparation and installation to a serialized background worker.
- Replace Windows tasklist subprocess polling with native ToolHelp process enumeration.
- Use wgpu with native window decorations; retain opt-in Glow compatibility mode.
- Cache filtering, virtualize list rows and replace repeated character-by-character ellipsis shaping.
- Repaint when results arrive; idle process probes only trigger a redraw when the status changes.

### UI / 界面
- Present all import results in a bounded modal dialog; keep the main layout stable.
- Clip independent columns and bound long labels/details; support small windows.
- Preserve the original icon artwork; add macOS optical margins with full Retina ICNS and 15 Windows ICO frames.
- Separate in-app branding from macOS Dock padding; retain the icon revision and advance the redesigned macOS bundle to build 12.
- Add accessible labels to custom buttons.
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
- Windows cross-build and static PE checks do **not** replace native Windows runtime acceptance; physical native interaction testing is deferred and is not claimed as complete.

## 0.1.0

Initial dual-platform installer, Material Design 3 UI, archive support and functional tests.
