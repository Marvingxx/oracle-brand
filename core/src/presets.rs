//! The export presets a new project starts with. Every one is ordinary data in `brand.json`: the
//! designer edits, removes or adds presets; nothing here is fixed.

use crate::model::{ExportPreset, Format, Role};
use crate::recolor::Treatment;

const NAMING: &str = "{marque}_{role}_{variante}";

fn preset(id: &str, name: &str, format: Format, folder: &str) -> ExportPreset {
    ExportPreset {
        id: id.into(),
        name: name.into(),
        format,
        width: None,
        height: None,
        scale: 1.0,
        margin: 0.0,
        background: None,
        dpi: None,
        quality: None,
        folder: folder.into(),
        naming: NAMING.into(),
        roles: Vec::new(),
        treatments: Vec::new(),
    }
}

pub fn defaults() -> Vec<ExportPreset> {
    vec![
        // The vector master as a clean SVG.
        preset("preset-svg", "Original vectoriel (SVG)", Format::Svg, "Logos/SVG"),
        preset("preset-pdf", "PDF vectoriel", Format::Pdf, "Logos/PDF"),
        ExportPreset { naming: format!("{NAMING}_1x"), ..preset("preset-web-1x", "Web 1×", Format::Png, "Logos/PNG") },
        ExportPreset { scale: 2.0, naming: format!("{NAMING}_2x"), ..preset("preset-web-2x", "Web 2×", Format::Png, "Logos/PNG") },
        ExportPreset { scale: 2.0, naming: format!("{NAMING}_2x"), ..preset("preset-webp", "Web WebP 2×", Format::Webp, "Logos/WebP") },
        ExportPreset {
            width: Some(2000),
            background: Some("#FFFFFF".into()),
            quality: Some(92),
            naming: format!("{NAMING}_fond-blanc"),
            ..preset("preset-jpeg", "JPEG fond blanc", Format::Jpeg, "Logos/JPEG")
        },
        ExportPreset {
            width: Some(1080),
            height: Some(1080),
            margin: 0.15,
            naming: format!("{NAMING}_avatar-{{w}}"),
            roles: vec![Role::Symbol, Role::PrimaryLogo],
            ..preset("preset-avatar", "Avatar carré (réseaux)", Format::Png, "Web-et-Reseaux")
        },
        ExportPreset {
            width: Some(512),
            height: Some(512),
            margin: 0.06,
            naming: "favicon-{w}".into(),
            roles: vec![Role::Favicon],
            ..preset("preset-favicon", "Favicon 512", Format::Png, "Web-et-Reseaux/Favicon")
        },
        ExportPreset {
            width: Some(4000),
            dpi: Some(300),
            naming: format!("{NAMING}_impression-300dpi"),
            ..preset("preset-print", "Impression PNG 300 dpi", Format::Png, "Impression")
        },
    ]
}

/// The whole delivery as Akrivi's Logo Exports hands it over: every colour treatment (colour,
/// inverted, black, white, greyscale) in SVG, PDF, EPS, AI, PNG at two sizes and JPEG, filed by
/// logo, treatment and format. JPEG has a white background, so it leaves out the treatments made
/// for dark backgrounds.
pub fn delivery_matrix() -> Vec<ExportPreset> {
    const FOLDER: &str = "Logos/{role}/{traitement}/{format}";
    const NAME: &str = "{marque}_{role}_{traitement}";
    let all = Treatment::ALL.to_vec();
    let light = vec![Treatment::Original, Treatment::Black, Treatment::Greyscale];
    let m = |id: &str, name: &str, format: Format| ExportPreset { treatments: all.clone(), naming: NAME.into(), ..preset(id, name, format, FOLDER) };
    vec![
        m("matrix-svg", "Livraison — SVG", Format::Svg),
        m("matrix-pdf", "Livraison — PDF", Format::Pdf),
        m("matrix-eps", "Livraison — EPS", Format::Eps),
        m("matrix-ai", "Livraison — AI", Format::Ai),
        ExportPreset { width: Some(1000), naming: format!("{NAME}_{{w}}px"), ..m("matrix-png-1000", "Livraison — PNG 1000 px", Format::Png) },
        ExportPreset { width: Some(3000), naming: format!("{NAME}_{{w}}px"), ..m("matrix-png-3000", "Livraison — PNG 3000 px", Format::Png) },
        ExportPreset {
            width: Some(2000),
            background: Some("#FFFFFF".into()),
            quality: Some(92),
            treatments: light,
            naming: format!("{NAME}_fond-blanc"),
            ..m("matrix-jpeg", "Livraison — JPEG fond blanc", Format::Jpeg)
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_presets_have_unique_ids_and_a_jpeg_background() {
        let p = defaults();
        let mut ids: Vec<_> = p.iter().map(|x| x.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), p.len());
        assert!(p.iter().filter(|x| x.format == Format::Jpeg).all(|x| x.background.is_some()));
    }
}
