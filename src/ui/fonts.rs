//! Roboto for Latin; language-specific Noto-derived SC/JP/KR UI glyphs at real 400/500 weights.
//! UI strings never rely on a system CJK font. System fallback is for arbitrary user filenames.
use crate::settings::Language;
use std::sync::{Arc, OnceLock};
pub const FAMILY_MEDIUM: &str = "ui-medium";
pub const FAMILY_SEMIBOLD: &str = "ui-bold";
pub fn setup(ctx: &egui::Context) {
    setup_for_language(ctx, Language::Zh);
}
pub fn setup_for_language(ctx: &egui::Context, language: Language) {
    static BASE: OnceLock<egui::FontDefinitions> = OnceLock::new();
    let mut fonts = BASE.get_or_init(definitions).clone();
    let region = match language {
        Language::Zh | Language::En => "sc",
        Language::Ja => "jp",
        Language::Ko => "kr",
    };
    let defaults = fonts.families[&egui::FontFamily::Proportional].clone();
    for (family, latin, weight) in [
        (egui::FontFamily::Proportional, "roboto", "regular"),
        (
            egui::FontFamily::Name(FAMILY_MEDIUM.into()),
            "roboto-medium",
            "medium",
        ),
        (
            egui::FontFamily::Name(FAMILY_SEMIBOLD.into()),
            "roboto-bold",
            "medium",
        ),
    ] {
        let mut list = vec![latin.to_owned(), format!("ui-{region}-{weight}")];
        for fallback in ["system-cjk", "system-unicode"] {
            if fonts.font_data.contains_key(fallback) {
                list.push(fallback.into());
            }
        }
        list.extend(defaults.clone());
        fonts.families.insert(family, list);
    }
    ctx.set_fonts(fonts);
}
fn definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    for (name, bytes) in [
        (
            "roboto",
            include_bytes!("../../assets/fonts/Roboto-400.ttf").as_slice(),
        ),
        (
            "roboto-medium",
            include_bytes!("../../assets/fonts/Roboto-500.ttf").as_slice(),
        ),
        (
            "roboto-bold",
            include_bytes!("../../assets/fonts/Roboto-700.ttf").as_slice(),
        ),
        (
            "ui-sc-regular",
            include_bytes!("../../assets/fonts/CavalryKit-UI-SC-Regular.otf").as_slice(),
        ),
        (
            "ui-sc-medium",
            include_bytes!("../../assets/fonts/CavalryKit-UI-SC-Medium.otf").as_slice(),
        ),
        (
            "ui-jp-regular",
            include_bytes!("../../assets/fonts/CavalryKit-UI-JP-Regular.otf").as_slice(),
        ),
        (
            "ui-jp-medium",
            include_bytes!("../../assets/fonts/CavalryKit-UI-JP-Medium.otf").as_slice(),
        ),
        (
            "ui-kr-regular",
            include_bytes!("../../assets/fonts/CavalryKit-UI-KR-Regular.otf").as_slice(),
        ),
        (
            "ui-kr-medium",
            include_bytes!("../../assets/fonts/CavalryKit-UI-KR-Medium.otf").as_slice(),
        ),
    ] {
        let mut data = egui::FontData::from_static(bytes);
        if name.starts_with("ui-") {
            // egui 0.31 sizes by ascent-descent, not em. Noto's taller metrics otherwise
            // make CJK visibly smaller than Latin. Optical normalization is tested at 1x/2x.
            data.tweak.scale = 1.18;
        }
        fonts.font_data.insert(name.into(), Arc::new(data));
    }
    if let Some(bytes) = load_cjk() {
        fonts.font_data.insert(
            "system-cjk".into(),
            Arc::new(egui::FontData::from_owned(bytes)),
        );
    }
    #[cfg(target_os = "macos")]
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/Supplemental/Arial Unicode.ttf") {
        fonts.font_data.insert(
            "system-unicode".into(),
            Arc::new(egui::FontData::from_owned(bytes)),
        );
    }
    fonts
}
#[cfg(target_os = "macos")]
fn load_cjk() -> Option<Vec<u8>> {
    [
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
    ]
    .iter()
    .find_map(|p| std::fs::read(p).ok())
}
#[cfg(target_os = "windows")]
fn load_cjk() -> Option<Vec<u8>> {
    let fonts = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
        .join("Fonts");
    ["msyh.ttc", "Deng.ttf", "simhei.ttf", "simsun.ttc"]
        .iter()
        .find_map(|name| std::fs::read(fonts.join(name)).ok())
}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn load_cjk() -> Option<Vec<u8>> {
    [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ]
    .iter()
    .find_map(|p| std::fs::read(p).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_cjk_latin_ink_stays_within_line_height_at_desktop_scales() {
        for language in Language::ALL {
            let ctx = egui::Context::default();
            setup_for_language(&ctx, language);
            for size in [11.0, 12.0, 14.0, 19.0, 24.0] {
                for dpi in [1.0, 1.25, 1.5, 2.0] {
                    ctx.set_pixels_per_point(dpi);
                    let _ = ctx.run(egui::RawInput::default(), |ctx| {
                        for font in [
                            super::super::theme::font(size),
                            super::super::theme::medium(size),
                        ] {
                            let g = ctx.fonts(|f| {
                                f.layout_no_wrap(
                                    "Cavalry 关于设置 日本語 한국어 0123".into(),
                                    font,
                                    egui::Color32::WHITE,
                                )
                            });
                            assert!(
                                g.mesh_bounds.top() >= g.rect.top() - 2.0
                                    && g.mesh_bounds.bottom() <= g.rect.bottom() + 2.0,
                                "ink clipped: {language:?} {size} {dpi}: {:?} {:?}",
                                g.rect,
                                g.mesh_bounds
                            );
                        }
                    });
                }
            }
        }
    }
}
