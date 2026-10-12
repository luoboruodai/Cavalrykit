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
