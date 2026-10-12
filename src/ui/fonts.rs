//! 字体加载:Roboto(MD3 字族,拉丁内嵌)+ CJK 系统回退。
//! MD3 字重映射:Regular 400 = Body/Title,Medium 500 = Label/Title Medium,Bold 700 = 备用强调。

use std::sync::Arc;

pub const FAMILY_MEDIUM: &str = "roboto-medium";
pub const FAMILY_SEMIBOLD: &str = "roboto-bold";

pub fn setup(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "roboto".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Roboto-400.ttf"
        ))),
    );
    fonts.font_data.insert(
        FAMILY_MEDIUM.to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Roboto-500.ttf"
        ))),
    );
    fonts.font_data.insert(
        FAMILY_SEMIBOLD.to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Roboto-700.ttf"
        ))),
    );

    let has_cjk = match load_cjk() {
        Some(bytes) => {
            fonts.font_data.insert(
                "cjk".to_owned(),
                Arc::new(egui::FontData::from_owned(bytes)),
            );
            true
        }
        None => {
            eprintln!("CavalryKit: 未找到系统中文字体,界面文字可能无法正确显示");
            false
        }
    };

    let proportional = fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default();
    proportional.insert(0, "roboto".to_owned());
    let mut medium_families = vec![FAMILY_MEDIUM.to_owned(), "roboto".to_owned()];
    let mut bold_families = vec![
        FAMILY_SEMIBOLD.to_owned(),
        FAMILY_MEDIUM.to_owned(),
        "roboto".to_owned(),
    ];
    if has_cjk {
        proportional.push("cjk".to_owned());
        medium_families.push("cjk".to_owned());
        bold_families.push("cjk".to_owned());
    }
    fonts.families.insert(
        egui::FontFamily::Name(FAMILY_MEDIUM.into()),
        medium_families,
    );
    fonts.families.insert(
        egui::FontFamily::Name(FAMILY_SEMIBOLD.into()),
        bold_families,
    );

    ctx.set_fonts(fonts);
}

#[cfg(target_os = "macos")]
fn load_cjk() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}

#[cfg(target_os = "windows")]
fn load_cjk() -> Option<Vec<u8>> {
    let fonts = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
        .join("Fonts");
    ["msyh.ttc", "simhei.ttf", "simsun.ttc", "Deng.ttf"]
        .iter()
        .find_map(|name| std::fs::read(fonts.join(name)).ok())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn load_cjk() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}
