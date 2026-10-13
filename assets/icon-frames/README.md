# Approved optical icon sources

These PNGs are the canonical frames of the approved Graphite Halo icon.
The macOS master is the exact A-direction preview the user selected, and each
smaller macOS frame is independently resized once from that approved raster;
the Windows optical frames remain individually rendered from their vector paths.
`../icon-glyph.svg` preserves the Windows scalable geometric tracing; the macOS
source is the approved raster A master and is not regenerated from this SVG.
`../icon-source.ico` preserves the supplied raster reference, and
`../icon-design.json` records the material and source checksums.

- `windows/`: 18 distinct physical-pixel sizes. Small frames have optical stroke
  coverage and pixel alignment; do not replace them with resizes of the 256px icon.
- `macos/`: approved A-direction Dock master with separate margins/shadows,
  resized once per target from 16 to 1024px.
- `macos-brand/`: A-direction unpadded macOS in-app branding.
- `app/`: Windows unpadded brand artwork, unchanged by macOS restoration.

`cargo run --locked --example import_icon` assembles ICO/ICNS containers from these
approved PNG bytes and copies the appropriate brand/window PNGs. It does not redraw
or sharpen the icon, and does not regenerate small frames from the uploaded ICO.
A material/geometry change requires a new source render and visual approval before
updating this canonical set. Normal app builds require no extra graphics runtime.
