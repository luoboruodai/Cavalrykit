use cavalrykit::icon_assets::{
    mac_icon, resize_square, trim_transparent, MAC_CANVAS, MAC_OPAQUE_TILE, WINDOWS_SIZES,
};
use image::RgbaImage;

fn rgba(bytes: &[u8]) -> RgbaImage {
    image::load_from_memory(bytes).unwrap().to_rgba8()
}
fn source() -> RgbaImage {
    trim_transparent(&rgba(include_bytes!("../assets/icon-source.ico")))
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
fn generated_assets_match_original_artwork_reproducibly() {
    let art = source();
    assert_eq!(
        resize_square(&art, 256),
        rgba(include_bytes!("../assets/icon.png"))
    );
    let mac = mac_icon(&art);
    assert_eq!(mac, rgba(include_bytes!("../assets/icon-macos.png")));
    assert_eq!(
        resize_square(&mac, 256),
        rgba(include_bytes!("../assets/icon-macos-window.png"))
    );
}

#[test]
fn windows_ico_contains_all_target_sizes_with_correct_rgba_pixels() {
    let bytes = include_bytes!("../assets/icon.ico");
    let u16_at = |i| u16::from_le_bytes(bytes[i..i + 2].try_into().unwrap()) as usize;
    let u32_at = |i| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
    assert_eq!((u16_at(0), u16_at(2)), (0, 1));
    assert_eq!(u16_at(4), WINDOWS_SIZES.len());
    let art = source();
    for (i, size) in WINDOWS_SIZES.into_iter().enumerate() {
        let entry = 6 + i * 16;
        let dimension = |n| if bytes[n] == 0 { 256 } else { bytes[n] as u32 };
        assert_eq!((dimension(entry), dimension(entry + 1)), (size, size));
        assert_eq!(u16_at(entry + 6), 32);
        let length = u32_at(entry + 8);
        let offset = u32_at(entry + 12);
        let frame = rgba(&bytes[offset..offset + length]);
        assert_eq!(frame, resize_square(&art, size), "wrong pixels at {size}px");
    }
}

#[test]
fn mac_dock_padding_does_not_shrink_the_in_app_brand_tile() {
    let brand = cavalrykit::ui::icon::brand_icon();
    assert_eq!((brand.width, brand.height), (256, 256));
    assert_eq!(
        brand.rgba,
        rgba(include_bytes!("../assets/icon.png")).into_raw()
    );
    let padded = rgba(include_bytes!("../assets/icon-macos-window.png"));
    assert_eq!(bounds(&padded, 128), (25, 25, 231, 231));
    assert_ne!(brand.rgba, padded.into_raw());
}
