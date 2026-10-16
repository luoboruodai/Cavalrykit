# CavalryKit 0.1.2

English · [简体中文](README.md)

A local installer and library manager for Cavalry scripts and plugins, for **Apple Silicon macOS and Windows x64**. The interface supports Simplified Chinese, English, Japanese, and Korean. CavalryKit does not upload imported content or execute imported scripts.

<img src="assets/sponsor-wechat.svg" alt="WeChat sponsorship" width="232" height="64">

![Main interface with synthetic example data](assets/screenshots/main.png)

<details><summary>Light theme, same layout</summary>

![Light interface with synthetic example data](assets/screenshots/main-light.png)

</details>

## Download and upgrade from 0.1.1

Download the **installable attachment** for your platform from the [v0.1.2 release](https://github.com/luoboruodai/Cavalrykit/releases/tag/v0.1.2). GitHub's automatic Source code archives are not installers.

| Platform | Attachment | Install / upgrade |
|---|---|---|
| Apple Silicon macOS | `CavalryKit-0.1.2-mac-arm64.dmg` | Quit CavalryKit, drag `CavalryKit.app` into Applications, replace the existing app, then eject the image |
| Windows x64 | `CavalryKit-0.1.2-win-x64.zip` | Extract the entire archive, close CavalryKit, then run `Install-CavalryKit.cmd` |
| Integrity | `SHA256SUMS.txt` | Compare the downloaded file's SHA-256, not just its name |

Both packages include **`INSTALL.zh-CN.txt`, `INSTALL.en.txt`, and bilingual `RELEASE_NOTES.md`**. Installation, upgrade, and recovery instructions are available offline.

- **Keep your data:** in-place upgrades do not migrate or delete Cavalry's Scripts/Plugins or reset CavalryKit preferences. Back up your own library and settings before upgrading.
- **macOS:** use `/Applications/CavalryKit.app`. Avoid launching renamed, older copies. The internal macOS build number for this release is **32**.
- **Windows:** the fixed entry point is `%LOCALAPPDATA%\Programs\CavalryKit\CavalryKit.exe`. Desktop/Start-menu shortcuts target that path. Updates back up the old EXE to `%LOCALAPPDATA%\CavalryKit\Backups`. Other portable copies are not deleted. If a pinned taskbar icon remains stale, unpin it and pin the updated shortcut again.
- **Signing:** macOS is ad-hoc signed, not notarized; Windows has no commercial code-signing certificate. Your system may warn about an unverified developer. The tool does not disable system protections.

On macOS, run `shasum -a 256 CavalryKit-0.1.2-mac-arm64.dmg` and compare the corresponding checksum line. On Windows, run `Get-FileHash .\CavalryKit-0.1.2-win-x64.zip -Algorithm SHA256` and compare the corresponding checksum line.

## What's new in 0.1.2

- **Quieter duplicate imports:** same-named scripts and plugins are marked Identical and skipped only when their full contents match. Different contents still offer Replace / Keep both / Skip.
- **More natural empty search:** the central search card and Script/Plugin cards have real separation at rest, while the side cards use opposite small angles and different heights with icons rotated to match. The panel outline now uses the sidebar's low-contrast divider gray. The 32/16/24pt content rhythm, fixed helper, and Close-icon Clear search remain without a left rail or moving hit regions.
- **Richer shortcut feedback:** all four shortcuts retain 40pt spacing while the whole card, icon, and label scale on hover/focus without clipping. Rails grow between 52% and 76% of the inner height without green backplates, and click targets stay fixed.
- **Lower settled-list rendering overhead:** shortcut geometry is computed directly and row visual transforms are limited to their entrance phase, so settled rows avoid repeated identity transforms. This does not replace GPU/system-specific stall diagnosis.
- **Smooth global appearance changes:** theme changes and Cancel restoration use smoothstep easing for a more natural transition. Reduced motion is immediate, and settled themes stop repainting.
- **Updated font subsets:** all six regional/weight UI fonts include the new duplicate-status strings.

## Usage

1. Drop `.jsc`, `.js`, `.jsx`, a plugin folder, or a supported archive into the window, or use Select file. Install content only from sources you trust.
2. For name conflicts, choose Replace / Keep both / Skip, or cancel the batch. Replacement is staged with recovery on failure; destructive choices are not accepted by a default Enter keypress.
3. Review installed, skipped, and failed items in a separate dialog. Expand technical details when needed.
4. If plugins require restarting, **save your Cavalry project first**. CavalryKit only requests a normal exit. Handle Cavalry's save dialog there. Stop waiting does not withdraw an exit request already sent to the host.
5. Filter the library by type or search names/groups/authors. Reveal opens the file location; Remove asks for confirmation before moving items to Trash/Recycle Bin.
6. Refresh after external library changes, or enable refresh when returning to the window.

Supported archives: ZIP, 7z, TAR, TAR.GZ/TGZ, TAR.BZ2/TBZ2. A directory with `definitions.json` is a plugin; its internal `.js` resources are not installed as standalone scripts. Common macOS metadata is filtered and archive path traversal is rejected.

### Settings and shortcuts

![Settings with synthetic example data](assets/screenshots/settings.png)

- UI language: Chinese / English / Japanese / Korean; filenames and script contents are not rewritten.
- Appearance: Cavalry dark, light, custom colors; 90%–125% UI scale; reduced motion.
- Installation: restart-prompt scope and refresh-on-focus; optional Cavalry application path.
- Preferences are stored in `CavalryKit/preferences.json` under the system configuration directory. Save is required to persist previews; failed writes preserve the original file.

`Cmd/Ctrl+F` Search · `Cmd/Ctrl+O` Select file · `Cmd/Ctrl+R` Refresh · `Cmd/Ctrl+,` Settings · `Esc` Cancel/close a dialog.

| Platform | Scripts | Plugins |
|---|---|---|
| macOS | `~/Library/Application Support/Cavalry/Scripts` | `~/Library/Application Support/Cavalry/Third-Party/Plugins` |
| Windows | `%APPDATA%\Cavalry\Scripts` | `%APPDATA%\Cavalry\Third-Party\Plugins` |

### Graphics compatibility mode

If an older graphics driver cannot initialize the default renderer, try this in Windows Command Prompt:

```bat
set CAVALRYKIT_RENDERER=glow
CavalryKit.exe
```

This is not a performance guarantee or a substitute for diagnosing driver/hardware issues.

## Verification scope

- Local tests cover installation/extraction, recovery, job state, preferences, multilingual fonts, layout, icons, and restart logic. Default tests use temporary directories/synthetic fixtures; the real-user-directory write test remains ignored.
- [Desktop checks](https://github.com/luoboruodai/Cavalrykit/actions/workflows/desktop.yml) runs formatting, Clippy, tests, packaging, and public-source audits on macOS and Windows. Windows also runs installer/rollback fixtures. Check the exact release-tag commit's result.
- The installed macOS main window and in-app icon have been observed. Dock appearance was checked from a user-provided real screenshot; resource bytes were validated separately.
- **Physical Windows/Cavalry, GPU/multiple-monitor dragging, mixed DPI, and taskbar-cache interaction remain manually unverified.** Hosted CI does not replace those observations. Synthetic screenshots/frame sequences are not native-interaction recordings.

## Build and project references

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
python3 -m unittest discover -s packaging -p 'test_release_packaging.py'
cargo run --locked --example import_icon -- --check
```

`assets/icon-design.json` freezes approved frame identities. Despite its historical name, `scripts/render_platform_icons.py` validates/assembles frames rather than redrawing old artwork. UI-preview code is excluded from normal release builds. Source archives contain no user library, private logs, approval records, or client material.

- [Bilingual release notes](RELEASE_NOTES.md)
- [Change history](CHANGELOG.md)
- [Packaging guide, Chinese](packaging/README-打包.md)
- [Validation checklist](packaging/VALIDATION.md) · [UI review](packaging/UI_REVIEW.md)
- [Third-party notices](packaging/THIRD-PARTY-NOTICES.txt)
