//! Generate platform assets without redrawing or destructively editing the original ICO.
use cavalrykit::icon_assets::{mac_icon, resize_square, trim_transparent, WINDOWS_SIZES};
use image::codecs::ico::{IcoEncoder, IcoFrame};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = image::open("assets/icon-source.ico")?.to_rgba8();
    let art = trim_transparent(&source);
    let windows = resize_square(&art, 256);
    windows.save("assets/icon.png")?;
    let mac = mac_icon(&art);
    mac.save("assets/icon-macos.png")?;
    resize_square(&mac, 256).save("assets/icon-macos-window.png")?;

    let mut frames = Vec::new();
    for size in WINDOWS_SIZES {
        // Resample each size from the source, never from another small ICO frame.
        let frame = resize_square(&art, size);
        frames.push(IcoFrame::as_png(
            frame.as_raw(),
            size,
            size,
            image::ExtendedColorType::Rgba8,
        )?);
    }
    IcoEncoder::new(std::fs::File::create("assets/icon.ico")?).encode_images(&frames)?;
    println!(
        "Generated macOS 1024px master + 256px window asset; Windows ICO sizes: {WINDOWS_SIZES:?}"
    );
    Ok(())
}
