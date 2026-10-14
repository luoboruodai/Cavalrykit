# Human-approved V6 icon frames

The current design is **D Ink Spotlight / V6**, explicitly approved on
2026-10-14. The folder/check/arrow/plugin artwork comes from the user's original
CavalryKit raster. No third-party C logo or private reference screenshot is
part of these distributed assets.

- `macos/`: approved 16–1024 px representations, including the project-specific
  optical margin and transparent shadow.
- `windows/`: 18 independently prepared optical-size PNGs, embedded unchanged
  into ICO and the Windows executable; never resize one master to replace them.
- `app/`, `macos-brand/`: approved unpadded in-app artwork, 32–1024 px. Runtime
  currently embeds the exact 256 px frame on both platforms.
- `windows-source/`, `app-source/`, `macos-source/`: prior V3 source frames kept
  only for preservation/regression checks, not current build output.

The approved bitmap glyph originated at 256 px. Larger resources do not claim
new vector detail. The 824 px opaque plate within the macOS 1024 canvas is this
project's optical choice, not an Apple-prescribed measurement. The subtle rim
and shadow have separate alpha bounds.

`assets/icon-design.json` freezes canonical/source/output hashes and the exact
approved candidate identity. `cargo run --locked --example import_icon`
assembles PNG payloads without resampling. Add `-- --check` for a read-only
verification. `python3 scripts/render_platform_icons.py --check` additionally
checks the frozen hashes; despite its historical name, it **does not redraw**
the retired metallic-border style. Any future design change requires new
review and approved source frames.
