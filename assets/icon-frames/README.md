# Approved metallic icon sources

The user-provided Cavalry example supplies a **material direction**: rounded dark
face, satin silver border, top-edge highlight and shadow. The CavalryKit folder,
check, arrow and plugin mark come from the original approved CavalryKit assets,
not from the third-party C icon. `../icon-source-approved-a-mac.png` and
`../icon-source-approved-a-app.png` preserve the approved A artwork.

`windows-source/` stores the prior 18 independently rendered optical-size PNGs.
`app-source/` stores the prior in-app Windows artwork. `macos/`,
`macos-brand/`, `windows/`, and `app/` contain the newly styled canonical
output frames. The Windows small-size symbol pixels are preserved; each frame
gets its own proportional border rather than being resized from a 256px icon.

When the approved style changes, run `scripts/render_platform_icons.py` with
Pillow and NumPy to regenerate each canonical frame. Normal builds need no
Python graphics runtime: `cargo run --locked --example import_icon -- --check`
checks and assembles the approved PNGs into ICO/ICNS and platform images.
Metadata and per-frame hashes live in `../icon-design.json`. Do not overwrite
the sources or quietly reuse an older icon's frame sequence.
