//! Deterministic, platform-specific icon construction from the original raster artwork.
//! Pixel canvas sizes and optical artwork size are deliberately separate.
use image::{imageops, Rgba, RgbaImage};

pub const MAC_CANVAS: u32 = 1024;
// Target the 206/256 opaque tile footprint measured in the local macOS Terminal icon.
// The source has a 1px antialiased fringe: 830px total artwork yields an 824px opaque tile.
// This is an optical alignment choice, not an Apple-mandated pixel measurement.
pub const MAC_OPAQUE_TILE: u32 = 824;
pub const MAC_ARTWORK: u32 = 830;
pub const WINDOWS_SIZES: [u32; 15] = [16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 128, 256];

/// Trim only fully transparent external margins; never crop the logo or black tile.
pub fn trim_transparent(image: &RgbaImage) -> RgbaImage {
    let (mut left, mut top, mut right, mut bottom) = (image.width(), image.height(), 0, 0);
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] != 0 {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x + 1);
            bottom = bottom.max(y + 1);
        }
    }
    if right <= left || bottom <= top {
        return RgbaImage::new(1, 1);
    }
    imageops::crop_imm(image, left, top, right - left, bottom - top).to_image()
}

pub fn resize_square(art: &RgbaImage, size: u32) -> RgbaImage {
    let scale = size as f64 / art.width().max(art.height()) as f64;
    let width = (art.width() as f64 * scale).round().max(1.0) as u32;
    let height = (art.height() as f64 * scale).round().max(1.0) as u32;
    let resized = imageops::resize(art, width, height, imageops::FilterType::Lanczos3);
    let mut canvas = RgbaImage::new(size, size);
    imageops::overlay(
        &mut canvas,
        &resized,
        ((size - width) / 2).into(),
        ((size - height) / 2).into(),
    );
    canvas
}

pub fn mac_icon(art: &RgbaImage) -> RgbaImage {
    let tile = resize_square(art, MAC_ARTWORK);
    let margin = (MAC_CANVAS - MAC_ARTWORK) / 2;
    let mut silhouette = RgbaImage::new(MAC_CANVAS, MAC_CANVAS);
    // A subtle external shadow is separate from the tile, and never changes its opaque bounds.
    for (x, y, pixel) in tile.enumerate_pixels() {
        silhouette.put_pixel(
            x + margin,
            y + margin + 8,
            Rgba([0, 0, 0, (pixel[3] as u16 * 40 / 255) as u8]),
        );
    }
    let mut canvas = imageops::blur(&silhouette, 12.0);
    imageops::overlay(&mut canvas, &tile, margin.into(), margin.into());
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trimming_preserves_all_nontransparent_artwork_and_aspect() {
        let mut image = RgbaImage::new(20, 20);
        image.put_pixel(3, 5, Rgba([255, 255, 255, 255]));
        image.put_pixel(14, 11, Rgba([80, 80, 80, 1]));
        let trimmed = trim_transparent(&image);
        assert_eq!(trimmed.dimensions(), (12, 7));
        assert_eq!(trimmed.get_pixel(0, 0)[3], 255);
        assert_eq!(trimmed.get_pixel(11, 6)[3], 1);
        let fitted = resize_square(&trimmed, 24);
        assert_eq!(fitted.dimensions(), (24, 24));
        assert_eq!(fitted.get_pixel(0, 0)[3], 0);
        assert_eq!(
            trim_transparent(&RgbaImage::new(12, 12)).dimensions(),
            (1, 1)
        );
    }
}
