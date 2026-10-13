#!/usr/bin/env python3
"""Render CavalryKit's user-requested metallic frame from its own approved art.

This optional design-time script needs Pillow and NumPy. Normal Rust builds
only assemble checked, canonical PNGs and need no image-generation tool.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
FRAMES = ASSETS / "icon-frames"
METAL = {"corner_radius_mac1024": 266, "band_width_mac1024": 48, "peak_gray": 150, "mac_shadow_strength": 0.29}
WINDOWS = [16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 128, 144, 160, 192, 256]
MAC = [16, 32, 64, 128, 256, 512, 1024]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


def signed_rounded_square(n, inset, radius):
    y, x = np.mgrid[0:n, 0:n].astype(np.float32)
    half = n / 2 - inset
    qx = np.abs(x + 0.5 - n / 2) - (half - radius)
    qy = np.abs(y + 0.5 - n / 2) - (half - radius)
    return np.hypot(np.maximum(qx, 0), np.maximum(qy, 0)) + np.minimum(np.maximum(qx, qy), 0) - radius


def metal_frame(source, inset, radius, width, peak, shadow):
    n = source.width
    if source.size != (n, n):
        raise ValueError("icon source is not square")
    a = np.asarray(source.convert("RGBA"), dtype=np.float32)
    distance = signed_rounded_square(n, inset, radius)
    inside = -distance
    y, x = np.mgrid[0:n, 0:n].astype(np.float32)
    xn, yn = x / (n - 1), y / (n - 1)
    coverage = np.clip(0.5 - distance, 0, 1)
    normalized = np.maximum(0, inside) / width
    profile = np.interp(normalized, [0, 0.08, 0.20, 0.36, 0.58, 0.78, 1.0], [0.78, 0.95, 1.0, 0.86, 0.63, 0.31, 0.0])
    light = 0.88 + 0.17 * (1 - yn) + 0.07 * (1 - xn) + 0.09 * np.maximum(0, yn - 0.72)
    sheen = 14 * np.exp(-((normalized - 0.16) / 0.12) ** 2) * (0.75 + 0.25 * (1 - yn))
    metal = 25 + (peak - 25) * profile * light + sheen
    blend = np.clip((width - inside) / max(5.0 * n / 1024, 0.6), 0, 1)
    gray = np.clip(metal[..., None] + np.array([0, 1, 3])[None, None, :], 0, 255)
    rgb = a[:, :, :3] * (1 - blend[..., None]) + gray * blend[..., None]
    rgba = np.uint8(np.clip(np.dstack((rgb, coverage * 255)), 0, 255))
    body = Image.fromarray(rgba, "RGBA")
    if shadow and inset > 0:
        offset = int(round(12 * n / 1024))
        silhouette = Image.new("L", (n, n))
        silhouette.paste(body.getchannel("A"), (0, offset))
        blurred = silhouette.filter(ImageFilter.GaussianBlur(18 * n / 1024))
        blurred = blurred.point(lambda v: int(v * METAL["mac_shadow_strength"]))
        composite = Image.new("RGBA", (n, n))
        composite.putalpha(blurred)
        composite.alpha_composite(body)
        body = composite
    return body


def png_bytes(image):
    stream = io.BytesIO()
    image.save(stream, format="PNG")
    return stream.getvalue()


def render():
    mac_source = ASSETS / "icon-source-approved-a-mac.png"
    app_source = ASSETS / "icon-source-approved-a-app.png"
    design = json.loads((ASSETS / "icon-design.json").read_text())
    if sha(mac_source) != design["approved_mac_preview_sha256"] or sha(app_source) != design["approved_mac_inapp_master_sha256"]:
        raise ValueError("approved CavalryKit source art has changed; do not silently redraw")
    output = {}
    mac = metal_frame(Image.open(mac_source), 100, 266, 48, 150, True)
    for size in MAC:
        image = mac if size == 1024 else mac.resize((size, size), Image.Resampling.LANCZOS)
        output[FRAMES / "macos" / f"{size}.png"] = png_bytes(image)
    mac_app = metal_frame(Image.open(app_source), 0, 266 / 824 * 1024, 48 / 824 * 1024, 150, False)
    for size in [256, 1024]:
        image = mac_app if size == 1024 else mac_app.resize((size, size), Image.Resampling.LANCZOS)
        output[FRAMES / "macos-brand" / f"{size}.png"] = png_bytes(image)
    for size in WINDOWS:
        source = Image.open(FRAMES / "windows-source" / f"{size}.png")
        inset = 0 if size <= 32 else round(size / 64)
        tile = size - inset * 2
        image = metal_frame(source, inset, 266 / 824 * tile, 48 / 824 * tile, 150, False)
        output[FRAMES / "windows" / f"{size}.png"] = png_bytes(image)
    for size in [256, 1024]:
        source = Image.open(FRAMES / "app-source" / f"{size}.png")
        image = metal_frame(source, 0, 266 / 824 * size, 48 / 824 * size, 150, False)
        output[FRAMES / "app" / f"{size}.png"] = png_bytes(image)
    return output


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="compare approved PNG bytes without writing")
    args = parser.parse_args()
    output = render()
    for path, data in output.items():
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                raise SystemExit(f"stale or mismatched metallic icon frame: {path.relative_to(ROOT)}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    print(f"{'Verified' if args.check else 'Rendered'} {len(output)} independent macOS/Windows icon source frames")


if __name__ == "__main__":
    main()
