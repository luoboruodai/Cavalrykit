use cavalrykit::icon_assets::{resize_square, MAC_CANVAS, MAC_OPAQUE_TILE, WINDOWS_SIZES};
use image::RgbaImage;

fn rgba(bytes: &[u8]) -> RgbaImage {
    image::load_from_memory(bytes).unwrap().to_rgba8()
}
fn frame(kind: &str, size: u32) -> Vec<u8> {
    std::fs::read(format!(
        "{}/assets/icon-frames/{kind}/{size}.png",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}
fn bounds(im: &RgbaImage, threshold: u8) -> (u32, u32, u32, u32) {
    let (mut l, mut t, mut r, mut b) = (im.width(), im.height(), 0, 0);
    for (x, y, p) in im.enumerate_pixels() {
        if p[3] >= threshold {
            l = l.min(x);
            t = t.min(y);
            r = r.max(x + 1);
            b = b.max(y + 1);
        }
    }
    (l, t, r, b)
}

#[test]
fn mac_canvas_and_opaque_tile_have_separate_sizes() {
    let mac = rgba(include_bytes!("../assets/icon-macos.png"));
    assert_eq!(mac.dimensions(), (MAC_CANVAS, MAC_CANVAS));
    let margin = (MAC_CANVAS - MAC_OPAQUE_TILE) / 2;
    assert_eq!(
        bounds(&mac, 128),
        (
            margin,
            margin,
            margin + MAC_OPAQUE_TILE,
            margin + MAC_OPAQUE_TILE
        )
    );
    for n in 0..MAC_CANVAS {
        assert_eq!(mac.get_pixel(n, 0)[3], 0);
        assert_eq!(mac.get_pixel(n, MAC_CANVAS - 1)[3], 0);
        assert_eq!(mac.get_pixel(0, n)[3], 0);
        assert_eq!(mac.get_pixel(MAC_CANVAS - 1, n)[3], 0);
    }
    let (left, top, right, bottom) = bounds(&mac, 1);
    assert!(
        left < margin
            && top < margin
            && right > margin + MAC_OPAQUE_TILE
            && bottom > margin + MAC_OPAQUE_TILE
    );
}

#[test]
fn runtime_images_match_the_approved_platform_sources() {
    for (bytes, kind, size) in [
        (include_bytes!("../assets/icon.png").as_slice(), "app", 256),
        (
            include_bytes!("../assets/icon-window.png").as_slice(),
            "windows",
            256,
        ),
        (
            include_bytes!("../assets/icon-macos.png").as_slice(),
            "macos",
            1024,
        ),
        (
            include_bytes!("../assets/icon-macos-brand.png").as_slice(),
            "macos-brand",
            256,
        ),
        (
            include_bytes!("../assets/icon-macos-window.png").as_slice(),
            "macos",
            256,
        ),
    ] {
        assert_eq!(bytes, frame(kind, size));
    }
}

#[test]
fn windows_ico_contains_all_target_sizes_with_correct_rgba_pixels() {
    let bytes = include_bytes!("../assets/icon.ico");
    let u16_at = |i| u16::from_le_bytes(bytes[i..i + 2].try_into().unwrap()) as usize;
    let u32_at = |i| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
    assert_eq!((u16_at(0), u16_at(2)), (0, 1));
    assert_eq!(u16_at(4), WINDOWS_SIZES.len());
    for (i, size) in WINDOWS_SIZES.into_iter().enumerate() {
        let entry = 6 + i * 16;
        let dimension = |n| if bytes[n] == 0 { 256 } else { bytes[n] as u32 };
        assert_eq!((dimension(entry), dimension(entry + 1)), (size, size));
        assert_eq!(u16_at(entry + 6), 32);
        let length = u32_at(entry + 8);
        let offset = u32_at(entry + 12);
        let frame = rgba(&bytes[offset..offset + length]);
        assert_eq!(
            frame,
            rgba(&self::frame("windows", size)),
            "wrong approved pixels at {size}px"
        );
    }
}

#[test]
fn mac_dock_padding_does_not_shrink_the_in_app_brand_tile() {
    let brand = cavalrykit::ui::icon::brand_icon();
    assert_eq!((brand.width, brand.height), (256, 256));
    #[cfg(target_os = "macos")]
    let brand_png = include_bytes!("../assets/icon-macos-brand.png").as_slice();
    #[cfg(not(target_os = "macos"))]
    let brand_png = include_bytes!("../assets/icon.png").as_slice();
    assert_eq!(brand.rgba, rgba(brand_png).into_raw());
    let padded = rgba(include_bytes!("../assets/icon-macos-window.png"));
    assert_eq!(bounds(&padded, 128), (25, 25, 231, 231));
    assert_ne!(brand.rgba, padded.into_raw());
}

#[test]
fn mac_icns_keeps_every_approved_frame_without_resampling() {
    let bytes = include_bytes!("../assets/icon.icns");
    assert_eq!(&bytes[..4], b"icns");
    let len = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
    assert_eq!(len, bytes.len());
    let mut offset = 8;
    for (tag, size) in [
        (b"icp4", 16),
        (b"icp5", 32),
        (b"icp6", 64),
        (b"ic07", 128),
        (b"ic08", 256),
        (b"ic09", 512),
        (b"ic10", 1024),
        (b"ic11", 32),
        (b"ic12", 64),
        (b"ic13", 256),
        (b"ic14", 512),
    ] {
        assert_eq!(&bytes[offset..offset + 4], tag);
        let size_bytes =
            u32::from_be_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        assert_eq!(
            &bytes[offset + 8..offset + size_bytes],
            frame("macos", size)
        );
        offset += size_bytes;
    }
    assert_eq!(offset, bytes.len());
}

#[test]
fn small_windows_frames_have_optical_coverage_not_a_master_resize() {
    let master = rgba(&frame("windows", 256));
    for size in [16, 20, 24, 32, 48] {
        let optical = rgba(&frame("windows", size));
        let resized = resize_square(&master, size);
        assert_ne!(
            optical, resized,
            "{size}px lost independent optical rendering"
        );
        let bright = |image: &RgbaImage| {
            image
                .enumerate_pixels()
                .filter(|(x, y, p)| {
                    *x > size / 8
                        && *x < size * 7 / 8
                        && *y > size / 8
                        && *y < size * 7 / 8
                        && p[0] > 210
                        && p[3] == 255
                })
                .count()
        };
        assert!(
            bright(&optical) > bright(&resized),
            "{size}px needs visible core stroke coverage"
        );
    }
}

#[test]
fn metallic_frames_track_approved_source_and_each_canonical_output() {
    use sha2::{Digest, Sha256};
    let design: serde_json::Value =
        serde_json::from_slice(include_bytes!("../assets/icon-design.json")).unwrap();
    let digest = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));
    let source = include_bytes!("../assets/icon-source-approved-a-mac.png");
    let source_app = include_bytes!("../assets/icon-source-approved-a-app.png");
    assert_eq!(
        digest(source),
        design["approved_mac_preview_sha256"].as_str().unwrap()
    );
    assert_eq!(
        digest(source_app),
        design["approved_mac_inapp_master_sha256"].as_str().unwrap()
    );
    let original = rgba(source);
    let final_master = rgba(include_bytes!("../assets/icon-macos.png"));
    for (x, y) in [(230, 500), (512, 512), (740, 400), (512, 700), (620, 250)] {
        assert_eq!(
            original.get_pixel(x, y),
            final_master.get_pixel(x, y),
            "glyph/interior drift at ({x}, {y})"
        );
    }
    let frames = design["canonical_frames_sha256"].as_object().unwrap();
    for (file, expected) in frames {
        let bytes = std::fs::read(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), file)).unwrap();
        assert_eq!(
            digest(&bytes),
            expected.as_str().unwrap(),
            "wrong frame: {file}"
        );
    }
    assert_ne!(original, final_master, "new satin material not applied");
}

#[test]
fn metallic_bezel_matches_material_request_without_erasing_small_icon_glyphs() {
    let before = rgba(include_bytes!("../assets/icon-source-approved-a-mac.png"));
    let after = rgba(include_bytes!("../assets/icon-macos.png"));
    assert!(
        after.get_pixel(512, 112)[0] >= before.get_pixel(512, 112)[0] + 40,
        "macOS satin border must be visibly silver, not the old almost-black bevel"
    );
    assert_eq!(after.get_pixel(512, 180), before.get_pixel(512, 180));
    for n in [16, 20, 24, 30, 32, 48, 64, 128, 256] {
        let source = rgba(&frame("windows-source", n));
        let styled = rgba(&frame("windows", n));
        assert_ne!(source, styled, "metallic material missing at {n}px");
        let bright_glyph = |image: &RgbaImage| {
            image
                .enumerate_pixels()
                .filter(|(x, y, p)| {
                    *x > n / 8
                        && *x < 7 * n / 8
                        && *y > n / 8
                        && *y < 7 * n / 8
                        && p[0] > 185
                        && p[3] > 190
                })
                .count()
        };
        assert_eq!(
            bright_glyph(&styled),
            bright_glyph(&source),
            "small symbol coverage changed at {n}px"
        );
    }
}
