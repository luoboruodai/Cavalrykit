#[test]
fn bundled_cjk_subset_covers_every_interface_language_without_system_fonts() {
    // Read the actual cmap: egui::has_glyph has a known false-negative for its '?' fallback.
    let font = ttf_parser::Face::parse(include_bytes!("../assets/fonts/CavalryKit-UI-CJK.otf"), 0)
        .unwrap();
    let mut missing = std::collections::BTreeSet::new();
    for language in cavalrykit::settings::Language::ALL {
        for key in cavalrykit::i18n::ALL {
            for c in language.tr(*key).chars().filter(|c| !c.is_control()) {
                if font.glyph_index(c).is_none() {
                    missing.insert(c);
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "UI glyphs missing without system fonts: {missing:?}"
    );
}
