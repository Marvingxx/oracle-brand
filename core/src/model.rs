//! The brand project file (`brand.json`): what the designer defines, never generated.
//!
//! Schema version 2. Paths to sources are stored relative to the project folder when the source
//! sits inside it (portable when the folder moves), absolute otherwise. Every list item has a stable
//! id; ids are unique within their list.

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandProject {
    pub schema_version: u32,
    pub project_id: String,
    pub brand_name: String,
    #[serde(default)]
    pub client: String,
    /// The identity's version, as the designer names it (`v1`).
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub notes: String,
    /// RFC 3339.
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub colors: Vec<BrandColor>,
    #[serde(default)]
    pub fonts: Vec<BrandFont>,
    #[serde(default)]
    pub rules: Rules,
    #[serde(default)]
    pub export_presets: Vec<ExportPreset>,
    #[serde(default)]
    pub deliveries: Vec<Delivery>,
    /// Brand Guide documents and Mockup Studio scenes that use this brand (paths, as for assets).
    #[serde(default)]
    pub guide_documents: Vec<String>,
    #[serde(default)]
    pub mockup_scenes: Vec<String>,
    #[serde(default)]
    pub guide: crate::guide::Guide,
    #[serde(default)]
    pub studio: crate::studio::Studio,
    /// Retain extensions written by other suite tools when saving this project.
    #[serde(flatten)]
    pub extensions: std::collections::BTreeMap<String, serde_json::Value>,
}

fn default_version() -> String {
    "v1".into()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    #[default]
    InProgress,
    Delivered,
    Archived,
}

impl Status {
    pub const ALL: [Status; 3] = [Status::InProgress, Status::Delivered, Status::Archived];
    pub fn label_fr(self) -> &'static str {
        match self {
            Status::InProgress => "En cours",
            Status::Delivered => "Livré",
            Status::Archived => "Archivé",
        }
    }
}

/// A master file the designer made (in Oracle Vector or Oracle Edit), in one role.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: String,
    pub role: Role,
    /// The designer's own name for it ("Logo principal — noir").
    #[serde(default)]
    pub label: String,
    /// The variant as the designer declares it: a variant is never derived by Oracle.
    #[serde(default)]
    pub variant: Variant,
    /// Path to the master file (relative to the project folder when inside it).
    pub source: String,
    /// SHA-256 of the master when it was last exported or checked.
    #[serde(default)]
    pub source_hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    PrimaryLogo,
    HorizontalLogo,
    VerticalLogo,
    Symbol,
    Favicon,
    Signature,
    SecondaryLogo,
    Other,
}

impl Role {
    pub const ALL: [Role; 8] =
        [Role::PrimaryLogo, Role::HorizontalLogo, Role::VerticalLogo, Role::Symbol, Role::Favicon, Role::Signature, Role::SecondaryLogo, Role::Other];
    pub fn label_fr(self) -> &'static str {
        match self {
            Role::PrimaryLogo => "Logo principal",
            Role::HorizontalLogo => "Logo horizontal",
            Role::VerticalLogo => "Logo vertical",
            Role::Symbol => "Symbole",
            Role::Favicon => "Favicon",
            Role::Signature => "Signature",
            Role::SecondaryLogo => "Version secondaire",
            Role::Other => "Autre",
        }
    }
    /// For file names: `logo-principal`.
    pub fn slug(self) -> &'static str {
        match self {
            Role::PrimaryLogo => "logo-principal",
            Role::HorizontalLogo => "logo-horizontal",
            Role::VerticalLogo => "logo-vertical",
            Role::Symbol => "symbole",
            Role::Favicon => "favicon",
            Role::Signature => "signature",
            Role::SecondaryLogo => "logo-secondaire",
            Role::Other => "autre",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Variant {
    #[default]
    Color,
    Black,
    White,
    Monochrome,
    Inverted,
    Simplified,
}

impl Variant {
    pub const ALL: [Variant; 6] = [Variant::Color, Variant::Black, Variant::White, Variant::Monochrome, Variant::Inverted, Variant::Simplified];
    pub fn label_fr(self) -> &'static str {
        match self {
            Variant::Color => "Couleur",
            Variant::Black => "Noir",
            Variant::White => "Blanc",
            Variant::Monochrome => "Monochrome",
            Variant::Inverted => "Inversée",
            Variant::Simplified => "Simplifiée",
        }
    }
    pub fn slug(self) -> &'static str {
        match self {
            Variant::Color => "couleur",
            Variant::Black => "noir",
            Variant::White => "blanc",
            Variant::Monochrome => "mono",
            Variant::Inverted => "inverse",
            Variant::Simplified => "simplifie",
        }
    }
}

/// A palette colour: the designer's values. CMYK, Pantone and RAL are only ever typed in, never
/// guessed; a CMYK value names the ICC profile it was converted with, when there was one.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandColor {
    pub id: String,
    pub name: String,
    /// `#RRGGBB`.
    pub hex: String,
    #[serde(default)]
    pub cmyk: Option<Cmyk>,
    #[serde(default)]
    pub pantone: String,
    #[serde(default)]
    pub ral: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cmyk {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
    /// The ICC profile the values come from (`Coated FOGRA39`); empty when typed in by hand.
    #[serde(default)]
    pub profile: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandFont {
    pub id: String,
    pub family: String,
    #[serde(default)]
    pub weights: Vec<String>,
    /// "Titres", "Texte courant"…
    #[serde(default)]
    pub usage: String,
    /// Local font files, for the designer's own use.
    #[serde(default)]
    pub local_files: Vec<String>,
    #[serde(default)]
    pub license_url: String,
    /// Whether the font may be handed to the client. Font files are never put in a delivery unless
    /// this is set.
    #[serde(default)]
    pub redistributable: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rules {
    /// Clear space around the logo, in a unit the designer defines ("hauteur du symbole").
    #[serde(default)]
    pub clear_space: String,
    #[serde(default)]
    pub min_size: String,
    #[serde(default)]
    pub allowed_backgrounds: String,
    /// Forbidden uses: the designer's text, with an example file they prepared (optional).
    #[serde(default)]
    pub forbidden: Vec<ForbiddenUse>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForbiddenUse {
    pub text: String,
    #[serde(default)]
    pub example: String,
}

/// How one kind of deliverable is made.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreset {
    pub id: String,
    pub name: String,
    pub format: Format,
    /// Output size in pixels (raster); `None` keeps the source's proportions from `scale`.
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    /// Pixels per source unit when no size is set (1 = the source's own size).
    #[serde(default = "one")]
    pub scale: f32,
    /// Margin around the logo, as a share of the output's shorter side (0.1 = 10 %).
    #[serde(default)]
    pub margin: f32,
    /// `None`: transparent (where the format has transparency); `Some("#RRGGBB")` otherwise.
    #[serde(default)]
    pub background: Option<String>,
    /// Pixel density written in the file's metadata, for print (dots per inch).
    #[serde(default)]
    pub dpi: Option<u32>,
    /// JPEG/WebP quality 1–100 (JPEG only; WebP is lossless).
    #[serde(default)]
    pub quality: Option<u8>,
    /// Sub-folder of the delivery (`Logos/PNG`).
    pub folder: String,
    /// File name, with `{marque}`, `{role}`, `{variante}`, `{preset}`, `{w}`, `{h}` and `{version}`.
    pub naming: String,
    /// Roles this preset applies to (empty: all).
    #[serde(default)]
    pub roles: Vec<Role>,
    /// Colour treatments to deliver, each a file of its own (empty: the master as drawn).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub treatments: Vec<crate::recolor::Treatment>,
}

fn one() -> f32 {
    1.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Svg,
    Png,
    Jpeg,
    Webp,
    Pdf,
    /// Encapsulated PostScript, for printers and older software (written by Oracle Vector).
    Eps,
    /// Illustrator-compatible document (written by Oracle Vector).
    Ai,
}

impl Format {
    pub const ALL: [Format; 7] = [Format::Svg, Format::Png, Format::Jpeg, Format::Webp, Format::Pdf, Format::Eps, Format::Ai];
    pub fn ext(self) -> &'static str {
        match self {
            Format::Svg => "svg",
            Format::Png => "png",
            Format::Jpeg => "jpg",
            Format::Webp => "webp",
            Format::Pdf => "pdf",
            Format::Eps => "eps",
            Format::Ai => "ai",
        }
    }
    pub fn is_raster(self) -> bool {
        matches!(self, Format::Png | Format::Jpeg | Format::Webp)
    }
    pub fn has_alpha(self) -> bool {
        matches!(self, Format::Png | Format::Webp | Format::Svg | Format::Pdf | Format::Ai)
    }
}

/// One export run kept in the project: where it went and whether it was handed to the client.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    /// Folder name under `Livrables/<marque>/` (`v1`, `v1-2`…).
    pub folder: String,
    pub created_at: String,
    /// Marked by the designer once sent: never written again; a regeneration makes a new folder.
    #[serde(default)]
    pub delivered: bool,
    #[serde(default)]
    pub outputs: usize,
}

impl BrandProject {
    /// A new, empty project.
    pub fn new(brand_name: &str) -> BrandProject {
        BrandProject {
            schema_version: SCHEMA_VERSION,
            project_id: uuid::Uuid::new_v4().to_string(),
            brand_name: brand_name.trim().to_string(),
            client: String::new(),
            version: default_version(),
            status: Status::InProgress,
            notes: String::new(),
            created_at: crate::now_rfc3339(),
            assets: Vec::new(),
            colors: Vec::new(),
            fonts: Vec::new(),
            rules: Rules::default(),
            export_presets: crate::presets::defaults(),
            deliveries: Vec::new(),
            guide_documents: Vec::new(),
            mockup_scenes: Vec::new(),
            guide: crate::guide::Guide::default(),
            studio: crate::studio::Studio::default(),
            extensions: Default::default(),
        }
    }

    /// A fresh id for a list item, unique in this project.
    pub fn new_id(&self, prefix: &str) -> String {
        loop {
            let id = format!("{prefix}-{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
            if !self.all_ids().any(|i| i == id) {
                return id;
            }
        }
    }

    fn all_ids(&self) -> impl Iterator<Item = &str> {
        self.assets
            .iter()
            .map(|a| a.id.as_str())
            .chain(self.colors.iter().map(|c| c.id.as_str()))
            .chain(self.fonts.iter().map(|f| f.id.as_str()))
            .chain(self.export_presets.iter().map(|p| p.id.as_str()))
    }

    /// Ids used twice within a list, which the project must not have.
    pub fn duplicate_ids(&self) -> Vec<String> {
        let mut out = Vec::new();
        for list in [
            self.assets.iter().map(|a| a.id.clone()).collect::<Vec<_>>(),
            self.colors.iter().map(|c| c.id.clone()).collect(),
            self.fonts.iter().map(|f| f.id.clone()).collect(),
            self.export_presets.iter().map(|p| p.id.clone()).collect(),
        ] {
            let mut seen = std::collections::HashSet::new();
            for id in list {
                if !seen.insert(id.clone()) {
                    out.push(id);
                }
            }
        }
        out
    }
}

/// `#RRGGBB` → (r, g, b); `None` for anything else.
pub fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let h = hex.trim().strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

/// HSL (degrees, %, %) of an sRGB colour, for display only.
pub fn hsl(rgb: [u8; 3]) -> (f32, f32, f32) {
    let [r, g, b] = rgb.map(|c| c as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    if d == 0.0 {
        return (0.0, 0.0, l * 100.0);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    ((h + 360.0) % 360.0, s * 100.0, l * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_and_hsl() {
        assert_eq!(parse_hex("#C82029"), Some([0xC8, 0x20, 0x29]));
        assert_eq!(parse_hex("C82029"), None);
        assert_eq!(parse_hex("#C8202"), None);
        let (h, s, l) = hsl([255, 0, 0]);
        assert!((h - 0.0).abs() < 0.01 && (s - 100.0).abs() < 0.01 && (l - 50.0).abs() < 0.01);
    }

    #[test]
    fn new_ids_are_unique_and_duplicates_are_found() {
        let mut p = BrandProject::new("Marque Exemple");
        let a = p.new_id("color");
        p.colors.push(BrandColor { id: a.clone(), name: "Rouge".into(), hex: "#C82029".into(), ..Default::default() });
        assert_ne!(p.new_id("color"), a);
        assert!(p.duplicate_ids().is_empty());
        p.colors.push(BrandColor { id: a.clone(), name: "Copie".into(), hex: "#000000".into(), ..Default::default() });
        assert_eq!(p.duplicate_ids(), [a]);
    }
}
