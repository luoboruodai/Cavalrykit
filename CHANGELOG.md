# Changelog

## 0.1.2 — 2026-10-16

### build 32 / 空结果面板分隔灰度
- Use the sidebar's low-contrast `p.line` divider instead of the brighter `p.border` for the empty panel; keep only a 2% resting accent and a restrained hover response while leaving every other component unchanged.

### build 31 / 扇形图标与中性弱描边
- Give the side cards opposite small angles and different heights while keeping the central card level; rotate each icon with its card and spread the decorative dots without changing fixed targets.
- Replace the empty panel's abrupt green outline with a quieter neutral stroke that only gains a restrained accent on hover.

### build 30 / 空状态间距层级与清除动作
- Give the resting illustration real separation before hover, and use 32/16/24pt spacing between the illustration, title, helper and action while retaining compact fallbacks.
- Remove the dynamic query from the helper, restore the fixed four-language guidance and regenerate the six UI font subsets.
- Add a Close icon to Clear search; animate the title/helper and button with restrained group hover, then strengthen the button's own visual/icon feedback without a left accent rail or moving hit targets.

### build 29 / 空状态层叠插画与内部联动
- Recompose the empty-search illustration as a front search card with layered Script/Plugin cards and restrained dots, preserving the spacious panel, query explanation and fixed targets.
- Drive the cards, icons and decorative dots from the existing finite hover progress: rear cards spread, the central card and icon lift and grow, and reduced motion still settles immediately without an idle repaint loop.

### build 28 / 大气化空状态与绿条修正
- Replace the compact empty-state card with a spacious panel: an 88pt central state icon, two 60pt supporting tiles and restrained decorative dots, while preserving the current palette and fixed hit regions.
- Integrate the query into a clear four-language explanation instead of a standalone line; keep Clear search and do not add unrelated creation actions.
- Increase shortcut accent rails from 52% to 76% of the inner height so the resting rail is easier to perceive without restoring excessive height.

### build 27 / 搜索空状态的参考化层级
- Surround the central empty-state icon with restrained script, plugin, folder and library glyphs while preserving the existing vector icon set and avoiding unrelated actions.
- Show the active search query on its own line above the existing guidance, truncate only the displayed long query, and keep Clear search and its hit target unchanged.

### build 26 / 间距、空状态层级与渲染减负
- Restore 40pt shortcut spacing and keep accent rails at 42%–68% of the inner height; compute bounded center-out card geometry directly so every animation frame stays inside the original hit region.
- Enlarge the no-results card to 480pt with a 64pt icon, 20pt title and 14pt helper text, while reducing the resting accent outline to 12% and preserving the clear action's fixed target.
- Skip identity visual transforms for settled library rows and remove the shortcut's whole-subtree transform as targeted rendering overhead reduction; this is not a claim that every GPU or system stall is eliminated.

### build 25 / 整体动效与视觉协调
- Animate the complete shortcut card, including its icon and label, with a bounded center-out scale that never clips the parent region.
- Soften the empty card’s resting outline and enlarge its content while preserving fixed hit targets and consistent spacing.

### build 24 / 空结果交互与快捷入口反馈
- Paint the empty state as one centered card with import-like hover feedback; keep its hit region fixed and center helper text by actual ink bounds.
- Lengthen shortcut accent rails and add subtle card scaling while keeping rail geometry and click targets inside their original hit regions.

### build 23 / 空状态间距、图标反馈与缓动细化
- Center no-results content in a compact 360pt card with 24pt padding, 48pt icon tile, and 16/8pt spacing between title, hint, and action.
- Remove green backplates from shortcut icons while preserving icon color changes, card borders, and accent-rail motion with stable hit regions.
- Use smoothstep easing for palette transitions, keeping retargets, exact final styles, reduced motion, and stop-repaint behavior intact.

## 0.1.2 — 2026-10-15

### build 22 / 重复导入、空状态与交互动效
- Skip same-named scripts and plugins as Identical only when their full contents match; preserve conflict choices for different content and never turn read failures into a silent skip.
- Redesign the no-results state with an icon tile, hierarchical guidance and a Clear search action; remove the separate × from the search field.
- Grow and brighten shortcut accent rails on hover/focus without moving their hit regions.
- Interpolate the complete palette for about 0.3 seconds on appearance changes and cancellation, with exact final-style application, reduced-motion fallback and no idle repaint loop.
- Regenerate all six regional UI font subsets for the new duplicate-status strings.

## 0.1.1 — 2026-10-14

### 正式发布 / Release summary
- Publish complete Chinese/English README and offline install/upgrade/recovery guides; include bilingual release notes inside both platform packages.
- Keep the approved build21/V6 runtime unchanged; freeze the tested release commit and validate downloaded release attachments before local handoff.
- Native macOS/Windows CI validates tests, platform packaging and source audit; Windows additionally validates installer/rollback. Physical Windows GPU/taskbar/mixed-DPI checks remain outside this evidence.

以下 build 记录是研发历史；最终发布采用 build21 / V6，不包含已退回的图标方案。

### build 21 / 正式采用已确认的 V6 图标
- Apply the explicitly approved D / V6 icon pixels to macOS Dock/window, both in-app brands, and all 18 Windows ICO frames; do not apply later rejected variants.
- Freeze per-frame and assembled-resource hashes. Replace the retired metallic renderer with validation/assembly so normal builds cannot silently redraw the old border.
- Keep UI layout, preferences, application behavior and user libraries unchanged. Windows native taskbar/Explorer/mixed-DPI appearance remains unverified on this macOS build host.

### build 20 / Cavalry 材质语言的边框调整
- Add an independently generated satin-silver perimeter, rounded relief and inner depth to CavalryKit's own approved file/check/plugin artwork, guided by the supplied Cavalry icon screenshot without copying the C symbol.
- Keep Mac Dock/window/in-app masters distinct from 18 optical-size Windows frames and the Windows in-app icon; preserve the original approved A art as the source.
- Verify each frame's source checksum and installed platform bundle, keeping browser appearance separate from native macOS/Windows rendering.

### build 19 / 恢复已确认的 macOS A 视觉锚点
- Restore the exact approved first-round A master as the macOS Dock/window icon, and derive each smaller ICNS frame once from that master.
- Use the same approved A artwork for the macOS in-app brand; preserve the Windows 18-frame ICO and Windows in-app artwork byte-for-byte.
- Pin source hashes in icon tests so a later vector redraw cannot silently replace the macOS visual direction.

### build 18 / macOS 图标登记修正
- Rebuild the macOS package with a content-addressed `CFBundleIconFile` resource name and reinstall it from the local DMG so Launch Services does not retain the previous same-name rendition.
- Keep the approved icon bytes unchanged; this is an installation/resource-identity correction, not a visual redesign.

### build 17 / 已确认图标方向与 Windows 高清化
- Apply the approved A “Graphite Halo” icon direction to CavalryKit’s app icon, in-app brand tile and platform resources.
- Replace the single-master Windows icon pipeline with 18 independently rendered optical-size frames; preserve a separate macOS ICNS and app/window image path.
- Add deterministic approved-frame checks, ICNS decoding checks and stale-source rejection before packaging.

### build 16 / 协同反馈与浅色材质
- Coordinate row-end reveal/remove icons and decorative dots with the row hover timeline while keeping hit targets fixed.
- Use one line-icon family for Program Location navigation and title; remove the old host-icon bitmap and texture.
- Tune Light secondary buttons/status surfaces and count badge density against the dark theme's geometry and hierarchy; retain explicit keyboard focus and readable semantic colors.
- Extend actual-paint and motion regressions; verify Light in the unobscured installed main window, not only inside settings preview.

### build 15 / 细节与一致性
- Add an inset rounded sidebar, ink-centered controls and balanced status padding with a bounded foreground-only breathing indicator.
- Align search/refresh with resting list cards; repair the circular refresh arrow and soften the inset row accent.
- Restore separate reveal/remove icon actions; trailing dots are decorative only, superseding the build14 menu.
- Keep reveal input gating separate from disabled colors to avoid a gray flash during entry.
- Use a complete Noto-derived content face for arbitrary mixed-script filenames, with shared baseline metrics instead of mixed system fallback.
- Remove the Save icon and reset hover plate, simplify safety copy, rename Cavalry light and Program Location, and align the path/browser controls.


### build 14 — 2026-10-13 / 后一轮界面要求
- Replace always-visible row actions with a three-dot menu, safe separation and removal confirmation.
- Add whole-card staggered upward reveal, fixed interaction targets, reference-style expanding row surfaces and interruptible modal entrance/exit.
- Center the actual search glyphs, add sixteen localized occasional hints (20s default / 4s suggestion), a green hover surface and a refresh tile.
- Put status beside the brand, make all four shortcut accents green, clarify drop-surface contrast and use the soft import material for Save.
- Close disabled background row popups when a modal opens; preserve reduced motion, filtering caches and virtualized rendering.

### UI and settings / 界面与设置（此前实现，部分由 build 14 调整）
- Integrate brand/status into the left rail; put the file action below the soft drop surface, without ellipsis.
- Stack four shortcut cards vertically with consistent type-colored accents.
- Use green scripts / blue plugins across row icons, accent bars, hover outlines and count badges.
- Match reveal/remove as two icon actions, keeping separate hit targets and safe spacing.
- Use adaptive segmented filters and an adjacent result badge; place refresh beside search.
- Coordinate import title/format/icon feedback; use an even drop outline without a thick left stripe.
- Keep staged list reveals stable while scrolling, skip empty-state repaint loops, and avoid repeated idle-query normalization allocations.

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
- Preserve the original icon artwork; add macOS optical margins with full Retina ICNS and 18 Windows ICO frames.
- Separate in-app branding from macOS Dock padding; retain the icon revision and advance the redesigned macOS bundle to build 14.
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
