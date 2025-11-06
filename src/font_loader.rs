use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use std::collections::BTreeSet;

/// Get list of all available system font families
pub fn get_system_fonts() -> Vec<String> {
    let source = SystemSource::new();
    let mut fonts = BTreeSet::new();

    // Try to get all fonts from system
    if let Ok(families) = source.all_families() {
        for family in families {
            fonts.insert(family);
        }
    }

    // Convert to sorted vector
    fonts.into_iter().collect()
}

/// Load a font by family name and return the font data
pub fn load_font_data(family_name: &str) -> Option<Vec<u8>> {
    let source = SystemSource::new();

    // Try to select the font
    let handle = source
        .select_best_match(
            &[FamilyName::Title(family_name.to_string())],
            &Properties::default(),
        )
        .ok()?;

    // Load the font data and convert Arc to Vec
    handle.load().ok()?.copy_font_data().map(|arc| (*arc).clone())
}
