//! 窗口图标：从打包进二进制的 PNG 解码为 egui IconData。

/// Native window icon with platform-specific optical margins.
pub fn window_icon() -> egui::IconData {
    #[cfg(target_os = "macos")]
    const PNG: &[u8] = include_bytes!("../../assets/icon-macos-window.png");
    #[cfg(not(target_os = "macos"))]
    const PNG: &[u8] = include_bytes!("../../assets/icon-window.png");
    decode(PNG)
}

/// UI brand tile intentionally keeps its existing in-panel size, without Dock-specific padding.
pub fn brand_icon() -> egui::IconData {
    #[cfg(target_os = "macos")]
    const PNG: &[u8] = include_bytes!("../../assets/icon-macos-brand.png");
    #[cfg(not(target_os = "macos"))]
    const PNG: &[u8] = include_bytes!("../../assets/icon.png");
    decode(PNG)
}

fn decode(png: &[u8]) -> egui::IconData {
    match image::load_from_memory(png) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            }
        }
        Err(e) => {
            eprintln!("CavalryKit: 图标解码失败 ({e})，使用空图标");
            egui::IconData::default()
        }
    }
}
