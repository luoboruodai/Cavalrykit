# Approved optical icon sources

These PNGs are the canonical, independently rendered frames of the approved
Graphite Halo icon. `../icon-glyph.svg` preserves the scalable geometric tracing;
`../icon-source.ico` preserves the supplied raster reference, and
`../icon-design.json` records the material and source checksums.

- `windows/`: 18 distinct physical-pixel sizes. Small frames have optical stroke
  coverage and pixel alignment; do not replace them with resizes of the 256px icon.
- `macos/`: separate Dock/canvas margins and shadows, from 16 to 1024px.
- `app/`: unpadded brand artwork, independent of Dock margins.

`cargo run --locked --example import_icon` assembles ICO/ICNS containers from these
approved PNG bytes and copies the appropriate brand/window PNGs. It does not redraw
or sharpen the icon, and does not regenerate small frames from the uploaded ICO.
A material/geometry change requires a new source render and visual approval before
updating this canonical set. Normal app builds require no extra graphics runtime.
