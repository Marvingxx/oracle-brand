//! The brand as design tokens, in the W3C Design Tokens Format Module 2025.10 (the first stable
//! version of the Community Group's specification): colours, font families and weights, readable by
//! Penpot, Figma (Tokens Studio), Style Dictionary and developers' tools. Written into every
//! delivery as `<marque>.tokens.json`.
//!
//! Spec: <https://www.w3.org/community/reports/design-tokens/CG-FINAL-format-20251028/> and the
//! colour module <https://www.w3.org/community/reports/design-tokens/CG-FINAL-color-20251028/>.
//! CMYK, Pantone and RAL are not token types: they travel in `$extensions` under
//! `app.oracle.brand`, as typed in the Brand Kit (never computed).

use serde_json::{Map, Value, json};

use crate::model::{BrandProject, parse_hex};

/// The extension key for what the specification has no type for.
pub const EXTENSION: &str = "app.oracle.brand";

/// A token name: lower-case words joined by `-`, never starting with `$` and without the `{`, `}`
/// and `.` the format reserves. Empty names become `token`.
pub fn token_name(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        let c = match c {
            'à' | 'â' | 'ä' | 'á' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' => 'i',
            'ô' | 'ö' | 'ó' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            'ç' => 'c',
            c => c,
        };
        if c.is_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() { "token".into() } else { out }
}

/// A weight name as the specification's number (Regular 400, Bold 700…); `None` when unknown.
pub fn weight_value(name: &str) -> Option<u16> {
    let key: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    Some(match key.as_str() {
        "thin" | "hairline" => 100,
        "extralight" | "ultralight" => 200,
        "light" => 300,
        "regular" | "normal" | "book" | "roman" => 400,
        "medium" => 500,
        "semibold" | "demibold" => 600,
        "bold" => 700,
        "extrabold" | "ultrabold" => 800,
        "black" | "heavy" => 900,
        "extrablack" | "ultrablack" => 950,
        _ => return key.parse::<u16>().ok().filter(|n| (1..=1000).contains(n)),
    })
}

/// Inserts `value` under a name not yet taken (`name`, `name-2`…).
fn insert_unique(map: &mut Map<String, Value>, name: String, value: Value) {
    let mut key = name.clone();
    let mut n = 2;
    while map.contains_key(&key) {
        key = format!("{name}-{n}");
        n += 1;
    }
    map.insert(key, value);
}

/// The tokens file for the brand.
pub fn tokens(data: &BrandProject) -> Value {
    let mut colors = Map::new();
    colors.insert("$type".into(), json!("color"));
    for c in &data.colors {
        let Some([r, g, b]) = parse_hex(&c.hex) else {
            continue;
        };
        let round = |v: u8| (f64::from(v) / 255.0 * 10_000.0).round() / 10_000.0;
        let mut token = json!({
            "$value": {"colorSpace": "srgb", "components": [round(r), round(g), round(b)], "hex": format!("#{r:02x}{g:02x}{b:02x}")},
            "$description": c.name,
        });
        let mut ext = Map::new();
        if let Some(k) = &c.cmyk {
            ext.insert(
                "cmyk".into(),
                json!({"c": k.c, "m": k.m, "y": k.y, "k": k.k, "profile": k.profile}),
            );
        }
        if !c.pantone.is_empty() {
            ext.insert("pantone".into(), json!(c.pantone));
        }
        if !c.ral.is_empty() {
            ext.insert("ral".into(), json!(c.ral));
        }
        if !ext.is_empty() {
            token["$extensions"] = json!({ EXTENSION: ext });
        }
        insert_unique(&mut colors, token_name(&c.name), token);
    }
    let mut families = Map::new();
    families.insert("$type".into(), json!("fontFamily"));
    let mut weights = Map::new();
    weights.insert("$type".into(), json!("fontWeight"));
    for f in data.fonts.iter().filter(|f| !f.family.trim().is_empty()) {
        let name = token_name(if f.usage.trim().is_empty() {
            &f.family
        } else {
            &f.usage
        });
        insert_unique(
            &mut families,
            name.clone(),
            json!({"$value": [f.family], "$description": format!("{} — {}", f.family, f.usage)}),
        );
        for w in &f.weights {
            if let Some(v) = weight_value(w) {
                insert_unique(
                    &mut weights,
                    format!("{name}-{}", token_name(w)),
                    json!({"$value": v, "$description": format!("{} {w}", f.family)}),
                );
            }
        }
    }
    let mut root = Map::new();
    root.insert(
        "$description".into(),
        json!(format!(
            "{} — tokens de marque {} (Oracle Brand Kit)",
            data.brand_name, data.version
        )),
    );
    if colors.len() > 1 {
        root.insert("color".into(), Value::Object(colors));
    }
    let mut font = Map::new();
    if families.len() > 1 {
        font.insert("family".into(), Value::Object(families));
    }
    if weights.len() > 1 {
        font.insert("weight".into(), Value::Object(weights));
    }
    if !font.is_empty() {
        root.insert("font".into(), Value::Object(font));
    }
    Value::Object(root)
}

/// The file name in a delivery: `<marque>.tokens.json` (the format's recommended extension).
pub fn file_name(data: &BrandProject) -> String {
    format!("{}.tokens.json", token_name(&data.brand_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BrandColor, BrandFont, Cmyk};

    fn brand() -> BrandProject {
        let mut b = BrandProject::new("Marque Été");
        b.version = "v1".into();
        b.colors = vec![
            BrandColor {
                id: "1".into(),
                name: "Rouge principal".into(),
                hex: "#D31C3B".into(),
                cmyk: Some(Cmyk {
                    c: 0.0,
                    m: 95.0,
                    y: 75.0,
                    k: 5.0,
                    profile: "Coated FOGRA39".into(),
                }),
                pantone: "186 C".into(),
                ..Default::default()
            },
            BrandColor {
                id: "2".into(),
                name: "Rouge principal".into(),
                hex: "#1b1a17".into(),
                ..Default::default()
            },
            BrandColor {
                id: "3".into(),
                name: "Sans valeur".into(),
                hex: "pas une couleur".into(),
                ..Default::default()
            },
        ];
        b.fonts = vec![BrandFont {
            id: "f".into(),
            family: "Inter".into(),
            usage: "Texte courant".into(),
            weights: vec!["Regular".into(), "SemiBold".into(), "Fantaisie".into()],
            ..Default::default()
        }];
        b
    }

    #[test]
    fn colours_follow_the_stable_format_with_print_values_in_extensions() {
        let t = tokens(&brand());
        let red = &t["color"]["rouge-principal"];
        assert_eq!(t["color"]["$type"], "color");
        assert_eq!(red["$value"]["colorSpace"], "srgb");
        assert_eq!(red["$value"]["hex"], "#d31c3b");
        assert_eq!(red["$value"]["components"][0], 0.8275);
        assert_eq!(red["$extensions"][EXTENSION]["pantone"], "186 C");
        assert_eq!(
            red["$extensions"][EXTENSION]["cmyk"]["profile"],
            "Coated FOGRA39"
        );
        // Two colours with one name both stay; an invalid one is left out.
        assert_eq!(t["color"]["rouge-principal-2"]["$value"]["hex"], "#1b1a17");
        assert!(t["color"].get("sans-valeur").is_none());
    }

    #[test]
    fn fonts_become_family_and_weight_tokens() {
        let t = tokens(&brand());
        assert_eq!(
            t["font"]["family"]["texte-courant"]["$value"],
            json!(["Inter"])
        );
        assert_eq!(t["font"]["weight"]["texte-courant-regular"]["$value"], 400);
        assert_eq!(t["font"]["weight"]["texte-courant-semibold"]["$value"], 600);
        // A weight with no number in the specification is not invented.
        assert_eq!(t["font"]["weight"].as_object().unwrap().len(), 3);
    }

    #[test]
    fn names_are_valid_token_names() {
        assert_eq!(
            token_name("Rouge Été {principal}.v2"),
            "rouge-ete-principal-v2"
        );
        assert_eq!(token_name("$secret"), "secret");
        assert_eq!(token_name("—"), "token");
        assert_eq!(file_name(&brand()), "marque-ete.tokens.json");
        for k in tokens(&brand())["color"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|k| *k != "$type")
        {
            assert!(!k.starts_with('$') && !k.contains(['{', '}', '.']));
        }
    }
}
