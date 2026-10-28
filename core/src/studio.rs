//! Independent presentation tools. Recipes reference the shared identity; no master is edited.
use crate::{Project, project::hash_file, render::Engines};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Studio {
    pub asset_id: String,
    /// Presentation units, not a claim about the source's geometric construction.
    pub grid_unit: u32,
    pub protection_units: u32,
    pub bento: Bento,
    pub headline: String,
    pub concepts: Vec<Concept>,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}
impl Default for Studio {
    fn default() -> Self {
        Self {
            asset_id: String::new(),
            grid_unit: 40,
            protection_units: 1,
            bento: Bento::Editorial,
            headline: String::new(),
            concepts: Vec::new(),
            extensions: BTreeMap::new(),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bento {
    #[default]
    Editorial,
    Mosaic,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Concept {
    pub id: String,
    pub title: String,
    pub asset_id: String,
    pub rationale: String,
    pub decision: Decision,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}
impl Concept {
    pub fn new(asset_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Nouvelle piste".into(),
            asset_id,
            ..Default::default()
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    #[default]
    Proposed,
    Retained,
    Rejected,
}
impl Decision {
    pub fn label(self) -> &'static str {
        match self {
            Self::Proposed => "À discuter",
            Self::Retained => "Retenue en interne",
            Self::Rejected => "Écartée en interne",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Grid,
    Bento,
    Presentations,
}
impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Self::Grid => "Grille de logo",
            Self::Bento => "Planche Bento",
            Self::Presentations => "Présentation de pistes",
        }
    }
    fn slug(self) -> &'static str {
        match self {
            Self::Grid => "grille",
            Self::Bento => "bento",
            Self::Presentations => "presentations",
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn short(s: &str, limit: usize) -> String {
    let mut text: String = s.chars().take(limit).collect();
    if s.chars().count() > limit {
        text.push('…');
    }
    text
}
fn text(x: u32, y: u32, size: u32, value: &str, color: &str) -> String {
    format!(
        "<text x='{x}' y='{y}' font-family='Arial' font-size='{size}' fill='{color}'>{}</text>",
        esc(value)
    )
}
fn rect(x: u32, y: u32, w: u32, h: u32, color: &str) -> String {
    format!("<rect x='{x}' y='{y}' width='{w}' height='{h}' rx='18' fill='{color}'/>")
}
fn image(
    project: &Project,
    engines: &Engines,
    id: &str,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<String, String> {
    let asset = project
        .data
        .assets
        .iter()
        .find(|a| a.id == id)
        .ok_or("Choisis un logo existant dans la bibliothèque de la marque.")?;
    let path = project.resolve(&asset.source);
    let (iw, ih) = crate::render::intrinsic_size(engines, &path)?;
    let scale = (1200.0 / iw.max(ih).max(1) as f32).min(1.0);
    let pixels = crate::render::rasterize(engines, &path, scale)?;
    let mut bytes = Cursor::new(Vec::new());
    pixels
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let data = base64::engine::general_purpose::STANDARD.encode(bytes.into_inner());
    Ok(format!(
        "<image x='{x}' y='{y}' width='{w}' height='{h}' preserveAspectRatio='xMidYMid meet' href='data:image/png;base64,{data}'/>"
    ))
}

/// Editable SVG composition. Logos are embedded PNG previews; text uses local Arial or fallback.
/// All full text is also preserved in HTML and the recipe, without truncation.
pub fn document(
    project: &Project,
    engines: &Engines,
    tool: Tool,
    concept: usize,
) -> Result<String, String> {
    let s = &project.data.studio;
    if tool == Tool::Grid && (!(10..=80).contains(&s.grid_unit) || s.protection_units > 2) {
        return Err("Unité : 10 à 80 ; protection : 0 à 2 unités.".into());
    }
    let mut out = String::from(
        "<svg xmlns='http://www.w3.org/2000/svg' width='1200' height='800' viewBox='0 0 1200 800'><rect width='1200' height='800' fill='#1B1A17'/>",
    );
    out.push_str(&text(
        48,
        60,
        26,
        &short(&project.data.brand_name, 55),
        "#FFFFFF",
    ));
    out.push_str(&text(48, 95, 17, tool.name(), "#CCCCCC"));
    match tool {
        Tool::Grid => {
            out.push_str(&rect(48, 125, 1104, 590, "#FFFFFF"));
            let unit = s.grid_unit;
            for x in (80..1140).step_by(unit as usize) {
                out.push_str(&format!("<path d='M{x} 145V695' stroke='#dddddd'/>"));
            }
            for y in (145..700).step_by(unit as usize) {
                out.push_str(&format!("<path d='M68 {y}H1132' stroke='#dddddd'/>"));
            }
            let p = unit * s.protection_units;
            out.push_str(&format!("<rect x='{}' y='{}' width='{}' height='{}' fill='none' stroke='#D31C3B' stroke-width='2' stroke-dasharray='8 6'/>", 300-p, 300-p, 600+2*p, 240+2*p));
            out.push_str(&image(project, engines, &s.asset_id, 300, 300, 600, 240)?);
            out.push_str(&format!("<rect x='300' y='300' width='600' height='240' fill='none' stroke='#1B1A17' stroke-width='2'/><path d='M600 180V660M120 420H1080' stroke='#D31C3B' stroke-dasharray='5 5'/>"));
            out.push_str(&text(48, 755, 17, &format!("1 u = {unit} unités de planche · protection = {} u · cadre du fichier, marges incluses", s.protection_units), "#FFFFFF"));
            out.push_str(&text(48, 783, 14, "Grille de présentation réglable ; ne reconstitue pas la géométrie de création du logo.", "#CCCCCC"));
        }
        Tool::Bento => {
            let left = if s.bento == Bento::Editorial {
                700
            } else {
                520
            };
            out.push_str(&rect(48, 128, left, 490, "#FFFFFF"));
            out.push_str(&image(
                project,
                engines,
                &s.asset_id,
                84,
                180,
                left - 72,
                380,
            )?);
            let right = 1200 - left - 120;
            out.push_str(&rect(left + 72, 128, right, 200, "#D31C3B"));
            out.push_str(&text(left + 96, 190, 25, "IDENTITÉ", "#FFFFFF"));
            out.push_str(&text(
                left + 96,
                235,
                17,
                &short(&project.data.version, 22),
                "#FFFFFF",
            ));
            out.push_str(&rect(left + 72, 352, right, 266, "#302E29"));
            for (i, font) in project.data.fonts.iter().take(3).enumerate() {
                out.push_str(&text(
                    left + 96,
                    405 + i as u32 * 65,
                    20,
                    &short(&font.family, 20),
                    "#FFFFFF",
                ));
                out.push_str(&text(
                    left + 96,
                    430 + i as u32 * 65,
                    14,
                    &short(&font.usage, 30),
                    "#CCCCCC",
                ));
            }
            let colors: Vec<_> = project
                .data
                .colors
                .iter()
                .filter(|c| crate::model::parse_hex(&c.hex).is_some())
                .take(8)
                .collect();
            let width = 1104 / colors.len().max(1) as u32;
            for (i, color) in colors.iter().enumerate() {
                out.push_str(&rect(48 + i as u32 * width, 642, width - 8, 48, &color.hex));
                out.push_str(&text(48 + i as u32 * width, 718, 14, &color.hex, "#FFFFFF"));
            }
            out.push_str(&text(48, 770, 23, &short(&s.headline, 72), "#FFFFFF"));
        }
        Tool::Presentations => {
            let c = s
                .concepts
                .get(concept)
                .ok_or("Ajoute au moins une piste de présentation.")?;
            out.push_str(&rect(48, 128, 740, 550, "#FFFFFF"));
            out.push_str(&image(project, engines, &c.asset_id, 90, 190, 656, 420)?);
            out.push_str(&text(824, 185, 28, &short(&c.title, 21), "#FFFFFF"));
            out.push_str(&text(824, 225, 16, c.decision.label(), "#FF6E86"));
            // Keep words intact in this narrow column; HTML preserves the unabridged text.
            let mut lines = Vec::<String>::new();
            for paragraph in c.rationale.lines() {
                let mut line = String::new();
                for word in paragraph.split_whitespace() {
                    let word = short(word, 26);
                    if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > 27 {
                        lines.push(std::mem::take(&mut line));
                    }
                    if !line.is_empty() {
                        line.push(' ');
                    }
                    line.push_str(&word);
                }
                lines.push(line);
            }
            if lines.len() > 12 {
                lines.truncate(12);
                lines[11] = format!("{}…", short(&lines[11], 25));
            }
            for (i, line) in lines.iter().enumerate() {
                out.push_str(&text(824, 280 + i as u32 * 28, 17, line, "#FFFFFF"));
            }
            out.push_str(&text(
                48,
                750,
                16,
                "Document de discussion · décision interne, aucun visa client automatique",
                "#CCCCCC",
            ));
        }
    }
    out.push_str("</svg>");
    Ok(out)
}

/// Fonts are resolved locally for the PNG. Font binaries are never bundled.
pub fn raster(svg: &str) -> Result<image::RgbaImage, String> {
    use resvg::{tiny_skia, usvg};
    static FONTS: std::sync::OnceLock<std::sync::Arc<usvg::fontdb::Database>> =
        std::sync::OnceLock::new();
    let fonts = FONTS.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        std::sync::Arc::new(db)
    });
    let options = usvg::Options {
        fontdb: fonts.clone(),
        ..Default::default()
    };
    let tree = usvg::Tree::from_data(svg.as_bytes(), &options).map_err(|e| e.to_string())?;
    let mut pixmap = tiny_skia::Pixmap::new(1200, 800).ok_or("Image vide")?;
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let mut out = image::RgbaImage::new(1200, 800);
    for (dst, px) in out.pixels_mut().zip(pixmap.pixels()) {
        let c = px.demultiply();
        *dst = image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Ok(out)
}

/// Export authorized source snapshots only, to a new revision. No modification of source files.
pub fn export(project: &Project, engines: &Engines, tool: Tool) -> Result<PathBuf, String> {
    let ids: Vec<String> = if tool == Tool::Presentations {
        if project.data.studio.concepts.is_empty() {
            return Err("Ajoute au moins une piste.".into());
        }
        if project.data.studio.concepts.len() > 50 {
            return Err("Maximum 50 pistes par présentation.".into());
        }
        project
            .data
            .studio
            .concepts
            .iter()
            .map(|c| c.asset_id.clone())
            .collect()
    } else {
        vec![project.data.studio.asset_id.clone()]
    };
    let root = project.dir.join("Livrables").join("Studio");
    crate::guide::no_links(&root)?;
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4();
    let stage = root.join(format!(".preparation-{id}"));
    std::fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let result = (|| {
        let mut snapshot = project.clone();
        let mut sources = BTreeMap::new();
        let mut temporary = Vec::new();
        for (i, asset_id) in ids.iter().enumerate() {
            if sources.contains_key(asset_id) {
                continue;
            }
            if !crate::guide::authorized(project, asset_id) {
                return Err(format!(
                    "Ressource {asset_id} : autorisation de remise absente ou périmée. Renseigne les droits dans Brand guide."
                ));
            }
            let a = snapshot
                .data
                .assets
                .iter_mut()
                .find(|a| &a.id == asset_id)
                .ok_or("Logo supprimé")?;
            let source = project.resolve(&a.source);
            let ext = source.extension().and_then(|v| v.to_str()).unwrap_or("");
            if ![
                "svg", "png", "jpg", "jpeg", "webp", "tif", "tiff", "bmp", "gif",
            ]
            .contains(&ext.to_ascii_lowercase().as_str())
            {
                return Err("Studio : aperçu autonome pris en charge pour SVG, PNG, JPEG, WebP, TIFF, BMP et première image GIF. Convertis une copie des autres maîtres avec les outils existants.".into());
            }
            let copy = stage.join(format!(".source-{i}.{ext}"));
            std::fs::copy(source, &copy).map_err(|e| e.to_string())?;
            let hash = hash_file(&copy)?;
            if project
                .data
                .guide
                .rights
                .get(asset_id)
                .is_none_or(|r| r.source_hash != hash)
            {
                return Err(
                    "Source modifiée pendant la préparation ; renouvelle les droits.".into(),
                );
            }
            sources.insert(asset_id.clone(), hash);
            a.source = std::fs::canonicalize(&copy)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            temporary.push(copy);
        }
        let count = if tool == Tool::Presentations {
            project.data.studio.concepts.len()
        } else {
            1
        };
        let mut hashes = BTreeMap::new();
        let mut page = format!(
            "<!doctype html><html lang=fr><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><meta http-equiv=Content-Security-Policy content=\"default-src 'none'; img-src 'self'; style-src 'unsafe-inline'\"><title>{}</title><style>body{{background:#1B1A17;color:white;font:18px system-ui;max-width:1200px;margin:auto;padding:24px}}a{{color:#ff6e86}}img{{width:100%}}section{{margin:32px 0}}p{{white-space:pre-wrap}}@media print{{section{{break-after:page}}}}</style><h1>{} — {}</h1><p>Document de travail · version {} · {}</p><p>Composition SVG avec aperçus de logos incorporés en PNG ; les logos ne sont pas vectorisés. Texte en Arial ou police locale de repli, sans redistribution de police.</p>",
            esc(tool.name()),
            esc(&project.data.brand_name),
            esc(tool.name()),
            esc(&project.data.version),
            esc(&crate::now_rfc3339())
        );
        for n in 0..count {
            let svg = document(&snapshot, engines, tool, n)?;
            let svg_name = format!("planche-{:02}.svg", n + 1);
            let png_name = format!("planche-{:02}.png", n + 1);
            std::fs::write(stage.join(&svg_name), &svg).map_err(|e| e.to_string())?;
            raster(&svg)?
                .save(stage.join(&png_name))
                .map_err(|e| e.to_string())?;
            page.push_str(&format!("<section><img src='{png_name}' alt='{} {}'><p><a download href='{svg_name}'>Composition SVG</a> · <a download href='{png_name}'>Image PNG 1200 × 800</a></p>", esc(tool.name()), n+1));
            if tool == Tool::Presentations {
                let c = &project.data.studio.concepts[n];
                page.push_str(&format!(
                    "<h2>{}</h2><p>{}</p><p>{}</p>",
                    esc(&c.title),
                    esc(&c.rationale),
                    esc(c.decision.label())
                ));
            } else if tool == Tool::Bento {
                page.push_str(&format!("<p>{}</p>", esc(&project.data.studio.headline)));
            }
            page.push_str("</section>");
            for name in [svg_name, png_name] {
                hashes.insert(name.clone(), hash_file(&stage.join(name))?);
            }
        }
        page.push_str("<footer>Oracle Studio · révision indépendante · aucune validation client automatique. Les recettes restent dans le projet Oracle.</footer></html>");
        std::fs::write(stage.join("index.html"), page).map_err(|e| e.to_string())?;
        let s = &project.data.studio;
        let recipe = match tool {
            Tool::Grid => {
                serde_json::json!({ "assetId":s.asset_id, "gridUnit":s.grid_unit, "protectionUnits":s.protection_units })
            }
            Tool::Bento => {
                serde_json::json!({ "assetId":s.asset_id, "layout":s.bento, "headline":s.headline })
            }
            Tool::Presentations => serde_json::json!({ "concepts":s.concepts }),
        };
        std::fs::write(
            stage.join("recette.json"),
            serde_json::to_vec_pretty(&recipe).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        for name in ["index.html", "recette.json"] {
            hashes.insert(name.into(), hash_file(&stage.join(name))?);
        }
        for file in temporary {
            std::fs::remove_file(file).map_err(|e| e.to_string())?;
        }
        let manifest = serde_json::json!({ "schemaVersion":1, "projectId":project.data.project_id, "version":project.data.version, "tool":tool.slug(), "createdAt":crate::now_rfc3339(), "sourceSha256ById":sources, "sha256":hashes, "clientApproval":false, "embeddedLogos":"png-previews", "fontFilesIncluded":false });
        std::fs::write(
            stage.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let folder = root.join(format!("{}-{id}", tool.slug()));
        std::fs::rename(&stage, &folder).map_err(|e| e.to_string())?;
        Ok(folder)
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&stage);
    }
    result
}
