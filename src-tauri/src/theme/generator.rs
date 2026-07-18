use material_colors::color::Argb;
use material_colors::scheme::SchemeFromPalette;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ThemeColors {
    pub light: HashMap<String, String>,
    pub dark: HashMap<String, String>,
}

pub fn from_color(argb: u32) -> ThemeColors {
    let c = Argb::from_u32(argb);
    let light = SchemeFromPalette::light(c);
    let dark = SchemeFromPalette::dark(c);

    ThemeColors {
        light: scheme_to_map(&light),
        dark: scheme_to_map(&dark),
    }
}

fn scheme_to_map(s: &SchemeFromPalette) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("primary".into(), s.primary.to_hex_with_pound());
    m.insert("on_primary".into(), s.on_primary.to_hex_with_pound());
    m.insert("primary_container".into(), s.primary_container.to_hex_with_pound());
    m.insert("on_primary_container".into(), s.on_primary_container.to_hex_with_pound());
    m.insert("secondary".into(), s.secondary.to_hex_with_pound());
    m.insert("on_secondary".into(), s.on_secondary.to_hex_with_pound());
    m.insert("tertiary".into(), s.tertiary.to_hex_with_pound());
    m.insert("error".into(), s.error.to_hex_with_pound());
    m.insert("surface".into(), s.surface.to_hex_with_pound());
    m.insert("on_surface".into(), s.on_surface.to_hex_with_pound());
    m.insert("background".into(), s.background.to_hex_with_pound());
    m.insert("on_background".into(), s.on_background.to_hex_with_pound());
    m.insert("outline".into(), s.outline.to_hex_with_pound());
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_blue() {
        let t = from_color(0xff0000ff);
        assert!(t.light.get("primary").unwrap().starts_with('#'));
        assert_eq!(t.light.get("primary").unwrap(), "#343dff");
    }

    #[test]
    fn test_from_red() {
        let t = from_color(0xffff0000);
        assert!(t.dark.get("primary").unwrap().starts_with('#'));
    }

    #[test]
    fn test_light_vs_dark_different() {
        let t = from_color(0xff00ff00);
        assert_ne!(t.light.get("surface"), t.dark.get("surface"));
    }
}
