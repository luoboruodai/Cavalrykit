#[test]
fn every_regional_weight_covers_all_ui_languages_without_system_fonts() {
    for (bytes, weight) in [
        (
            include_bytes!("../assets/fonts/CavalryKit-UI-SC-Regular.otf").as_slice(),
            400,
        ),
        (
            include_bytes!("../assets/fonts/CavalryKit-UI-JP-Regular.otf").as_slice(),
            400,
        ),
        (
            include_bytes!("../assets/fonts/CavalryKit-UI-KR-Regular.otf").as_slice(),
            400,
        ),
        (
            include_bytes!("../assets/fonts/CavalryKit-UI-SC-Medium.otf").as_slice(),
            500,
        ),
        (
            include_bytes!("../assets/fonts/CavalryKit-UI-JP-Medium.otf").as_slice(),
            500,
        ),
        (
            include_bytes!("../assets/fonts/CavalryKit-UI-KR-Medium.otf").as_slice(),
            500,
        ),
    ] {
        let font = ttf_parser::Face::parse(bytes, 0).unwrap();
        assert_eq!(font.weight().to_number(), weight);
        let mut missing = std::collections::BTreeSet::new();
        for language in cavalrykit::settings::Language::ALL {
            for text in cavalrykit::i18n::ALL
                .iter()
                .map(|key| language.tr(*key))
                .chain([language.name()])
            {
                for c in text.chars().filter(|c| !c.is_control()) {
                    if font.glyph_index(c).is_none() {
                        missing.insert(c);
                    }
                }
            }
        }
        assert!(missing.is_empty(), "Missing UI glyphs: {missing:?}");
    }
}

#[test]
fn arbitrary_mixed_filenames_share_complete_content_font_coverage() {
    let face = ttf_parser::Face::parse(
        include_bytes!("../assets/fonts/CavalryKit-Content-Regular.otf"),
        0,
    )
    .unwrap();
    // Synthetic multilingual phrases, not a subset sampled from a user's library.
    for ch in "Simulate动态素材归档 替换未使用资源 插值仿射关键帧Bezier摄像机 标题日本語ひらがな한글테스트 ABCxyz123._-".chars() {assert!(face.glyph_index(ch).is_some(),"missing content glyph: {ch}");}
}
