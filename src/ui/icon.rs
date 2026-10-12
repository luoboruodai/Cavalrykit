//! 窗口图标：从打包进二进制的 PNG 解码为 egui IconData。

/// 应用图标（assets/icon.png，由提供的 assets/icon.ico 转换）。
pub fn window_icon() -> egui::IconData {
    const PNG: &[u8] = include_bytes!("../../assets/icon.png");
    match image::load_from_memory(PNG) {
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
