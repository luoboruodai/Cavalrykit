//! Assemble approved optical-size sources without resampling the supplied ICO.
use cavalrykit::icon_assets::WINDOWS_SIZES;
use std::{error::Error, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn png(path: impl AsRef<Path>, size: u32) -> Result<Vec<u8>> {
    let bytes = std::fs::read(path)?;
    let image = image::load_from_memory(&bytes)?;
    if (image.width(), image.height()) != (size, size) || image.color() != image::ColorType::Rgba8 {
        return Err(format!("Expected an approved {size}px RGBA source").into());
    }
    Ok(bytes)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let check = args.as_slice() == ["--check"];
    if !args.is_empty() && !check {
        return Err("Usage: import_icon [--check]".into());
    }
    let windows: Vec<_> = WINDOWS_SIZES
        .into_iter()
        .map(|size| {
            Ok((
                size,
                png(format!("assets/icon-frames/windows/{size}.png"), size)?,
            ))
        })
        .collect::<Result<_>>()?;
    let mut ico = Vec::new();
    ico.extend_from_slice(&[0, 0, 1, 0]);
    ico.extend_from_slice(&(windows.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * windows.len() as u32;
    for (size, data) in &windows {
        let dimension = if *size == 256 { 0 } else { *size as u8 };
        let length = u32::try_from(data.len())?;
        ico.extend_from_slice(&[dimension, dimension, 0, 0, 1, 0, 32, 0]);
        ico.extend_from_slice(&length.to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset = offset.checked_add(length).ok_or("ICO too large")?;
    }
    for (_, data) in windows {
        ico.extend_from_slice(&data);
    }
    let mut chunks = Vec::new();
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
        let data = png(format!("assets/icon-frames/macos/{size}.png"), size)?;
        chunks.extend_from_slice(tag);
        chunks.extend_from_slice(&(u32::try_from(data.len())? + 8).to_be_bytes());
        chunks.extend_from_slice(&data);
    }
    let mut icns = b"icns".to_vec();
    icns.extend_from_slice(&(u32::try_from(chunks.len())? + 8).to_be_bytes());
    icns.extend_from_slice(&chunks);
    let outputs = [
        ("assets/icon.ico", ico),
        ("assets/icon.icns", icns),
        (
            "assets/icon.png",
            png("assets/icon-frames/app/256.png", 256)?,
        ),
        (
            "assets/icon-macos-brand.png",
            png("assets/icon-frames/macos-brand/256.png", 256)?,
        ),
        (
            "assets/icon-window.png",
            png("assets/icon-frames/windows/256.png", 256)?,
        ),
        (
            "assets/icon-macos.png",
            png("assets/icon-frames/macos/1024.png", 1024)?,
        ),
        (
            "assets/icon-macos-window.png",
            png("assets/icon-frames/macos/256.png", 256)?,
        ),
    ];
    for (path, data) in outputs {
        if check {
            if std::fs::read(path)? != data {
                return Err(format!("{path} is stale; run cargo run --example import_icon").into());
            }
        } else {
            std::fs::write(path, data)?;
        }
    }
    println!(
        "Approved source frames {} (18-frame ICO + ICNS + brand/window PNGs)",
        if check { "verified" } else { "assembled" }
    );
    Ok(())
}
