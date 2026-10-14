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
        bounds(&mac, 225),
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
    assert_eq!(bounds(&padded, 225), (25, 25, 231, 231));
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
        if size <= 32 {
            assert!(
                bright(&optical) > bright(&resized),
                "{size}px needs visible core stroke coverage"
            );
        } else {
            // V6 preserves V3's 48px glyph. Its approved stroke can be thinner
            // than a filtered 256px master; more white pixels is not the goal.
            assert_eq!(
                bright(&optical),
                bright(&rgba(&frame("windows-source", size))),
                "approved glyph coverage changed at {size}px"
            );
        }
    }
}

#[test]
fn approved_v6_manifest_tracks_every_frame_and_output() {
    use sha2::{Digest, Sha256};
    let design: serde_json::Value =
        serde_json::from_slice(include_bytes!("../assets/icon-design.json")).unwrap();
    assert_eq!(design["revision"], "icon-final-inset-v6");
    for section in [
        "canonical_frames_sha256",
        "source_frames_sha256",
        "runtime_assets_sha256",
    ] {
        for (file, expected) in design[section].as_object().unwrap() {
            let bytes = std::fs::read(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), file)).unwrap();
            assert_eq!(
                format!("{:x}", Sha256::digest(&bytes)),
                expected.as_str().unwrap(),
                "wrong approved bytes: {file}"
            );
        }
    }
}

#[test]
fn approved_v6_master_is_the_frozen_human_choice_not_a_redraw() {
    use sha2::{Digest, Sha256};
    // Independent anchors captured from the actual approved V6 candidate.
    for (bytes, expected) in [
        (
            include_bytes!("../assets/icon-macos.png").as_slice(),
            "c3bf2c7fd87cf78e03e73a0609872e267f1c2338945199dcf7d4e44d4228a6c8",
        ),
        (
            include_bytes!("../assets/icon.ico").as_slice(),
            "b281a93e3a3ff4c20fd0af42a2c0940ca8a4a735d0365a5e6e79b3289ef9cd6a",
        ),
        (
            include_bytes!("../assets/icon.png").as_slice(),
            "9b401f8447cb7e7a1794b43e251724b6cc95dd268647a828c563fef4c43c2f22",
        ),
        (
            include_bytes!("../assets/icon.icns").as_slice(),
            "f60ac4fd92cb6ba3a41686b4adb34912f1b9376a2030e30090ca460d514e398f",
        ),
    ] {
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), expected);
    }
}

#[test]
fn v6_preserves_v3_glyph_interior_and_alpha() {
    let before = rgba(&frame("macos-source", 1024));
    let after = rgba(include_bytes!("../assets/icon-macos.png"));
    for (x, y, pixel) in before.enumerate_pixels() {
        assert_eq!(pixel[3], after.get_pixel(x, y)[3], "alpha drift at {x},{y}");
        if (150..874).contains(&x) && (150..874).contains(&y) && pixel[3] == 255 {
            // Exclude rounded-corner edge region, retain the central artwork.
            if (280..744).contains(&x) || (280..744).contains(&y) {
                assert_eq!(pixel, after.get_pixel(x, y), "body/glyph drift at {x},{y}");
            }
        }
    }
    for size in [16, 20, 24, 30, 32] {
        assert_eq!(
            rgba(&frame("windows", size)),
            rgba(&frame("windows-source", size))
        );
    }
}

#[test]
fn v6_has_the_approved_narrow_highlight_not_the_rejected_broad_border() {
    let mac = rgba(include_bytes!("../assets/icon-macos.png"));
    assert_eq!(mac.get_pixel(512, 100).0, [46, 47, 47, 255]);
    assert_eq!(mac.get_pixel(512, 105).0, [169, 169, 169, 255]);
    assert_eq!(mac.get_pixel(512, 116).0, [11, 11, 11, 255]);
    // Alpha-128 includes the subtle rim; the opaque plate is checked separately.
    assert_eq!(bounds(&mac, 128), (98, 98, 926, 926));
}
