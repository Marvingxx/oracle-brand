//! Mockup Studio's files: templates and scenes. Strictly 2D, nothing generated.
//!
//! A **template** is a folder holding `template.json`, a copy of the photograph, and the masks and
//! overlays the designer prepared for it (a copy: the library's original is never touched). Its
//! zones are where a design can go: four corners on the photo, the printable area's real
//! proportions, an optional mask, a blend mode and an opacity.
//!
//! A **scene** is a template filled for one brand: each zone holds a master of the brand project
//! (by id, with its role and variant so the scene can be reused for another brand) or any picture,
//! over an optional background colour (a palette colour or a hex value). Scenes live in the brand
//! project (`Mockups/Scènes/*.json`, listed in `brand.json`), and render to `Mockups/<scène>.<ext>`.
//! Each render records the hash of everything that went in, so a changed logo, mask or setting
//! shows which scenes are out of date; nothing is re-rendered unless asked.

use std::path::{Path, PathBuf};

use image::{GrayImage, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::composite::{Quality, TemplateImages, ZoneDesign};
use crate::model::{ExportPreset, Format, Role, Variant, parse_hex};
use crate::project::{Project, hash_file, resolve_with, safe_name};
use crate::render::{Engines, master_kind, rasterize};

pub const SCHEMA_VERSION: u32 = 1;
pub const TEMPLATE_FILE: &str = "template.json";
/// Renders, inside the brand project; scenes go in its `Scènes` sub-folder.
pub const RENDERS_DIR: &str = "Mockups";
pub const SCENES_SUBDIR: &str = "Scènes";
/// The largest design drawn for a zone (px per side).
const MAX_CONTENT_SIDE: f64 = 8192.0;

fn one() -> f32 {
    1.0
}

/// How a zone's design mixes with the photo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Blend {
    #[default]
    Normal,
    /// The photo's shading shows through (ink on paper).
    Multiply,
    /// Light on dark (a reflection, a lit sign).
    Screen,
}

impl Blend {
    pub const ALL: [Blend; 3] = [Blend::Normal, Blend::Multiply, Blend::Screen];
    pub fn label_fr(self) -> &'static str {
        match self {
            Blend::Normal => "Normal",
            Blend::Multiply => "Produit",
            Blend::Screen => "Superposition claire",
        }
    }
}

/// How a design fits its zone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Fit {
    /// Whole design visible, proportions kept.
    #[default]
    Contain,
    /// Zone filled, proportions kept, the excess cropped.
    Cover,
    /// Zone filled, proportions changed (only on request).
    Stretch,
}

impl Fit {
    pub const ALL: [Fit; 3] = [Fit::Contain, Fit::Cover, Fit::Stretch];
    pub fn label_fr(self) -> &'static str {
        match self {
            Fit::Contain => "Contenir",
            Fit::Cover => "Remplir (recadrer)",
            Fit::Stretch => "Étirer",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    /// "Papeterie", "Enseigne"…
    #[serde(default)]
    pub category: String,
    /// The photograph, relative to the template's folder.
    pub photo: String,
    /// Where the photo comes from, and what its licence allows (the designer's words).
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub license: String,
    /// Bottom to top.
    #[serde(default)]
    pub zones: Vec<Zone>,
    /// What stands in front of the designs (white or opaque = in front), framed like the photo.
    #[serde(default)]
    pub occlusion: Option<String>,
    /// Shadows, reflections, grain… drawn over everything, framed like the photo.
    #[serde(default)]
    pub overlays: Vec<Overlay>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub id: String,
    pub name: String,
    /// Photo pixels: top-left, top-right, bottom-right, bottom-left of the printable area.
    pub corners: [[f64; 2]; 4],
    /// The printable area's real proportions (85 × 55 for a business card).
    pub ratio: [f64; 2],
    /// Where the design may show (white or opaque = shown), framed like the photo.
    #[serde(default)]
    pub mask: Option<String>,
    #[serde(default)]
    pub blend: Blend,
    #[serde(default = "one")]
    pub opacity: f32,
}

impl Zone {
    pub fn new(id: &str, name: &str) -> Zone {
        Zone { id: id.into(), name: name.into(), corners: [[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]], ratio: [1.0, 1.0], mask: None, blend: Blend::Normal, opacity: 1.0 }
    }

    /// Width over height of the printable area.
    pub fn ratio_value(&self) -> f64 {
        let [w, h] = self.ratio;
        if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 { w / h } else { 1.0 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Overlay {
    #[serde(default)]
    pub name: String,
    pub file: String,
    #[serde(default)]
    pub blend: Blend,
    #[serde(default = "one")]
    pub opacity: f32,
}

impl Template {
    pub fn new(name: &str, photo: &str) -> Template {
        Template {
            schema_version: SCHEMA_VERSION,
            id: format!("template-{}", &uuid::Uuid::new_v4().simple().to_string()[..12]),
            name: name.trim().to_string(),
            category: String::new(),
            photo: photo.into(),
            source: String::new(),
            license: String::new(),
            zones: Vec::new(),
            occlusion: None,
            overlays: Vec::new(),
        }
    }

    /// Reads `template.json` (or the folder holding it).
    pub fn load(path: &Path) -> Result<(PathBuf, Template), String> {
        let file = if path.is_dir() { path.join(TEMPLATE_FILE) } else { path.to_path_buf() };
        let t: Template = read_json(&file, "modèle")?;
        if t.schema_version > SCHEMA_VERSION {
            return Err(format!("{} vient d'une version plus récente d'Oracle (schéma {}) : il n'est pas modifié", file.display(), t.schema_version));
        }
        let mut ids = std::collections::HashSet::new();
        if let Some(z) = t.zones.iter().find(|z| !ids.insert(z.id.as_str())) {
            return Err(format!("{} : identifiant de zone en double ({})", file.display(), z.id));
        }
        Ok((file, t))
    }

    /// Writes `template.json` into `dir`, atomically.
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        write_json(&dir.join(TEMPLATE_FILE), self)
    }

    /// A new zone id, unique in this template.
    pub fn new_zone_id(&self) -> String {
        (1..).map(|n| format!("zone-{n}")).find(|id| !self.zones.iter().any(|z| &z.id == id)).unwrap_or_default()
    }
}

/// Makes a template from a photograph: a new folder in `root` holding a copy of the photo and a
/// first rectangular zone in its middle. The photo itself is only read.
pub fn create_template(root: &Path, photo: &Path, name: &str) -> Result<PathBuf, String> {
    let ext = photo.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    if !matches!(master_kind(photo), Some("raster" | "document")) {
        return Err(format!("{} : une photo PNG, JPEG, WebP, TIFF ou PSD est attendue", photo.display()));
    }
    let name = if name.trim().is_empty() { photo.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Modèle".into()) } else { name.trim().to_string() };
    let dir = unique_dir(root, &safe_name(&name));
    std::fs::create_dir_all(&dir).map_err(|e| format!("impossible de créer {} : {e}", dir.display()))?;
    let photo_name = format!("photo.{ext}");
    std::fs::copy(photo, dir.join(&photo_name)).map_err(|e| format!("copie de {} impossible : {e}", photo.display()))?;
    let mut t = Template::new(&name, &photo_name);
    t.source = photo.display().to_string();
    if let Ok((w, h)) = image::image_dimensions(dir.join(&photo_name)) {
        let (w, h) = (f64::from(w), f64::from(h));
        let mut z = Zone::new("zone-1", "Zone 1");
        z.corners = [[w * 0.3, h * 0.3], [w * 0.7, h * 0.3], [w * 0.7, h * 0.7], [w * 0.3, h * 0.7]];
        z.ratio = [(w * 0.4).round(), (h * 0.4).round()];
        t.zones.push(z);
    }
    t.save(&dir)?;
    Ok(dir.join(TEMPLATE_FILE))
}

/// Brings a mask or overlay file into the template's folder (a copy, under `name`), and returns the
/// path to store. A file already inside the folder is used where it is.
pub fn adopt_file(template_dir: &Path, file: &Path, name: &str) -> Result<String, String> {
    if image::image_dimensions(file).is_err() {
        return Err(format!("{} n'est pas une image lisible", file.display()));
    }
    if let (Ok(f), Ok(d)) = (std::fs::canonicalize(file), std::fs::canonicalize(template_dir))
        && let Ok(rel) = f.strip_prefix(&d)
    {
        return Ok(rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"));
    }
    let ext = file.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_else(|| "png".into());
    let mut target = format!("{name}.{ext}");
    let mut n = 2;
    while template_dir.join(&target).exists() {
        target = format!("{name}-{n}.{ext}");
        n += 1;
    }
    std::fs::copy(file, template_dir.join(&target)).map_err(|e| format!("copie de {} impossible : {e}", file.display()))?;
    Ok(target)
}

fn unique_dir(root: &Path, base: &str) -> PathBuf {
    let mut dir = root.join(base);
    let mut n = 2;
    while dir.exists() {
        dir = root.join(format!("{base} {n}"));
        n += 1;
    }
    dir
}

/// A palette colour of the brand (by id; by position when reused for another brand), or a value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Paint {
    #[serde(rename_all = "camelCase")]
    BrandColor { color_id: String, index: usize },
    Hex { hex: String },
}

/// What goes in a zone.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Content {
    #[default]
    None,
    /// A master of the brand project. The role and variant find the equivalent master when the
    /// scene is reused for another brand.
    #[serde(rename_all = "camelCase")]
    BrandAsset { asset_id: String, role: Role, variant: Variant },
    /// Any picture or vector file (stored like a project source path).
    File { path: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    pub zone_id: String,
    #[serde(default)]
    pub content: Content,
    #[serde(default)]
    pub background: Option<Paint>,
    #[serde(default)]
    pub fit: Fit,
    #[serde(default = "one")]
    pub scale: f32,
    /// Degrees, clockwise.
    #[serde(default)]
    pub rotation: f32,
    /// Shift of the design's centre, as a share of the zone's width and height.
    #[serde(default)]
    pub offset: [f32; 2],
}

impl Fill {
    pub fn empty(zone_id: &str) -> Fill {
        Fill { zone_id: zone_id.into(), content: Content::None, background: None, fit: Fit::Contain, scale: 0.6, rotation: 0.0, offset: [0.0, 0.0] }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    /// PNG, JPEG or WebP.
    pub format: Format,
    /// Pixels wide; `None`: the photo's own width.
    #[serde(default)]
    pub width: Option<u32>,
    /// JPEG quality.
    #[serde(default)]
    pub quality: Option<u8>,
}

impl Default for Output {
    fn default() -> Output {
        Output { format: Format::Png, width: None, quality: Some(92) }
    }
}

/// One thing a render was made from, and its hash then.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub key: String,
    /// For the designer: "Logo « Logo principal »".
    pub label: String,
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRecord {
    /// Relative to the project.
    pub file: String,
    pub at: String,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    pub inputs: Vec<Input>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    /// The template's `template.json`: relative to the project when inside it, absolute otherwise.
    pub template: String,
    /// Finds the template again in the templates folder when its path no longer exists.
    pub template_id: String,
    #[serde(default)]
    pub fills: Vec<Fill>,
    #[serde(default)]
    pub output: Output,
    #[serde(default)]
    pub last_render: Option<RenderRecord>,
}

impl Scene {
    pub fn load(path: &Path) -> Result<Scene, String> {
        let s: Scene = read_json(path, "scène")?;
        if s.schema_version > SCHEMA_VERSION {
            return Err(format!("{} vient d'une version plus récente d'Oracle (schéma {}) : elle n'est pas modifiée", path.display(), s.schema_version));
        }
        Ok(s)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("impossible de créer {} : {e}", dir.display()))?;
        }
        write_json(path, self)
    }

    /// The fill of a zone, created empty when missing (a zone added to the template later).
    pub fn fill_mut(&mut self, zone_id: &str) -> &mut Fill {
        if let Some(i) = self.fills.iter().position(|f| f.zone_id == zone_id) {
            return &mut self.fills[i];
        }
        self.fills.push(Fill::empty(zone_id));
        let last = self.fills.len() - 1;
        &mut self.fills[last]
    }

    pub fn fill(&self, zone_id: &str) -> Option<&Fill> {
        self.fills.iter().find(|f| f.zone_id == zone_id)
    }

    /// The render's path in the project.
    pub fn render_path(&self, project: &Project) -> PathBuf {
        project.dir.join(RENDERS_DIR).join(format!("{}.{}", safe_name(&self.name), self.output.format.ext()))
    }
}

/// Where a scene's render stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneState {
    NeverRendered,
    UpToDate,
    /// What changed since the render.
    Outdated(Vec<String>),
    /// Can't be drawn (template missing, file unreadable…).
    Broken(String),
}

impl SceneState {
    pub fn label_fr(&self) -> String {
        match self {
            SceneState::NeverRendered => "Jamais rendue".into(),
            SceneState::UpToDate => "À jour".into(),
            SceneState::Outdated(why) => format!("À mettre à jour : {}", why.join(", ")),
            SceneState::Broken(e) => format!("Impossible à rendre : {e}"),
        }
    }
}

// --- Scenes in a brand project -----------------------------------------------------------------

/// Creates a scene from a template in the project, with one empty fill per zone, and lists it in
/// `brand.json` (saved).
pub fn create_scene(project: &mut Project, template_file: &Path, name: &str) -> Result<PathBuf, String> {
    let (template_file, template) = Template::load(template_file)?;
    let name = if name.trim().is_empty() { template.name.clone() } else { name.trim().to_string() };
    let dir = project.dir.join(RENDERS_DIR).join(SCENES_SUBDIR);
    std::fs::create_dir_all(&dir).map_err(|e| format!("impossible de créer {} : {e}", dir.display()))?;
    let mut path = dir.join(format!("{}.json", safe_name(&name)));
    let mut n = 2;
    let mut final_name = name.clone();
    while path.exists() {
        final_name = format!("{name} {n}");
        path = dir.join(format!("{}.json", safe_name(&final_name)));
        n += 1;
    }
    let scene = Scene {
        schema_version: SCHEMA_VERSION,
        id: format!("scene-{}", &uuid::Uuid::new_v4().simple().to_string()[..12]),
        name: final_name,
        template: project.store_path(&template_file),
        template_id: template.id.clone(),
        fills: template.zones.iter().map(|z| Fill::empty(&z.id)).collect(),
        output: Output::default(),
        last_render: None,
    };
    scene.save(&path)?;
    project.data.mockup_scenes.push(project.store_path(&path));
    project.save()?;
    Ok(path)
}

/// The project's scenes: (stored path, file, scene or why it can't be read).
pub fn scenes(project: &Project) -> Vec<(String, PathBuf, Result<Scene, String>)> {
    project.data.mockup_scenes.iter().map(|s| (s.clone(), project.resolve(s), Scene::load(&project.resolve(s)))).collect()
}

/// Copies a scene into another brand project (its logos found again by role and variant, its
/// palette colours by position), and lists it there. The render record is not carried over.
pub fn copy_scene_to(scene_file: &Path, from: &Project, to: &mut Project) -> Result<PathBuf, String> {
    let mut scene = Scene::load(scene_file)?;
    let template_file = resolve_with(&from.dir, &scene.template);
    let template_file = std::path::absolute(&template_file).unwrap_or(template_file);
    let path = create_scene(to, &template_file, &scene.name)?;
    let created = Scene::load(&path)?;
    scene.id = created.id;
    scene.name = created.name;
    scene.template = created.template;
    scene.last_render = None;
    for fill in scene.fills.iter_mut() {
        match &fill.content {
            Content::BrandAsset { role, variant, .. } => {
                fill.content = match find_equivalent(to, *role, *variant) {
                    Some(a) => Content::BrandAsset { asset_id: a.id.clone(), role: a.role, variant: a.variant },
                    None => Content::None,
                };
            }
            Content::File { path } => {
                let abs = resolve_with(&from.dir, path);
                fill.content = Content::File { path: to.store_path(&abs) };
            }
            Content::None => {}
        }
        if let Some(Paint::BrandColor { index, .. }) = &fill.background {
            fill.background = to.data.colors.get(*index).map(|c| Paint::BrandColor { color_id: c.id.clone(), index: *index });
        }
    }
    scene.save(&path)?;
    Ok(path)
}

fn find_equivalent(project: &Project, role: Role, variant: Variant) -> Option<&crate::model::Asset> {
    let assets = &project.data.assets;
    assets.iter().find(|a| a.role == role && a.variant == variant).or_else(|| assets.iter().find(|a| a.role == role))
}

/// The scenes using a master, by name (for "used in N mockups").
pub fn scenes_using_asset(project: &Project, asset_id: &str) -> Vec<String> {
    scenes(project)
        .into_iter()
        .filter_map(|(_, _, s)| s.ok())
        .filter(|s| s.fills.iter().any(|f| matches!(&f.content, Content::BrandAsset { asset_id: a, .. } if a == asset_id)))
        .map(|s| s.name)
        .collect()
}

/// The scene's template file: its stored path, else the template with the same id in one of
/// `template_roots` (a project or templates folder that moved).
pub fn find_template(project: &Project, scene: &Scene, template_roots: &[PathBuf]) -> Result<PathBuf, String> {
    let stored = project.resolve(&scene.template);
    if stored.is_file() {
        return Ok(stored);
    }
    for root in template_roots {
        let Ok(entries) = std::fs::read_dir(root) else { continue };
        for e in entries.flatten() {
            let f = e.path().join(TEMPLATE_FILE);
            if f.is_file() && Template::load(&f).is_ok_and(|(_, t)| t.id == scene.template_id) {
                return Ok(f);
            }
        }
    }
    Err(format!("modèle introuvable : {} (relocalisez-le)", stored.display()))
}

// --- Drawing ------------------------------------------------------------------------------------

/// Reads a template's photo, masks and overlays. Masks and overlays must have the photo's
/// proportions (they are drawn in its frame).
pub fn load_template_images(template_file: &Path, template: &Template, engines: &Engines) -> Result<TemplateImages, String> {
    let dir = template_file.parent().unwrap_or(Path::new("."));
    let photo = rasterize(engines, &resolve_with(dir, &template.photo), 1.0).map_err(|e| format!("photo du modèle : {e}"))?;
    let aspect = f64::from(photo.width()) / f64::from(photo.height().max(1));
    let check = |w: u32, h: u32, what: &str| -> Result<(), String> {
        let a = f64::from(w) / f64::from(h.max(1));
        if (a / aspect - 1.0).abs() > 0.01 {
            return Err(format!("{what} : {w}×{h} n'a pas les proportions de la photo ({}×{})", photo.width(), photo.height()));
        }
        Ok(())
    };
    let mask = |stored: &str, what: &str| -> Result<GrayImage, String> {
        let path = resolve_with(dir, stored);
        let img = image::open(&path).map_err(|e| format!("{what} ({}) illisible : {e}", path.display()))?.to_rgba8();
        check(img.width(), img.height(), what)?;
        // White and opaque = 1; black or transparent = 0.
        Ok(GrayImage::from_fn(img.width(), img.height(), |x, y| {
            let p = img.get_pixel(x, y);
            let luma = (u32::from(p[0]) * 2126 + u32::from(p[1]) * 7152 + u32::from(p[2]) * 722) / 10_000;
            image::Luma([(luma * u32::from(p[3]) / 255) as u8])
        }))
    };
    let mut zone_masks = Vec::new();
    for z in &template.zones {
        zone_masks.push(match &z.mask {
            Some(m) => Some(mask(m, &format!("masque de la zone « {} »", z.name))?),
            None => None,
        });
    }
    let occlusion = match &template.occlusion {
        Some(m) => Some(mask(m, "masque de premier plan")?),
        None => None,
    };
    let mut overlays = Vec::new();
    for (i, o) in template.overlays.iter().enumerate() {
        let path = resolve_with(dir, &o.file);
        let img = image::open(&path).map_err(|e| format!("surcouche {} ({}) illisible : {e}", i + 1, path.display()))?.to_rgba8();
        check(img.width(), img.height(), &format!("surcouche {}", i + 1))?;
        overlays.push(img);
    }
    Ok(TemplateImages { photo, zone_masks, occlusion, overlays })
}

/// The file a fill's content points at.
pub fn content_file(project: &Project, content: &Content) -> Result<Option<(PathBuf, String)>, String> {
    match content {
        Content::None => Ok(None),
        Content::BrandAsset { asset_id, .. } => {
            let a = project.data.assets.iter().find(|a| &a.id == asset_id).ok_or_else(|| format!("le logo {asset_id} n'est plus dans le Brand Kit"))?;
            let label = if a.label.is_empty() { a.role.label_fr().to_string() } else { a.label.clone() };
            Ok(Some((project.resolve(&a.source), label)))
        }
        Content::File { path } => {
            let p = project.resolve(path);
            let label = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone());
            Ok(Some((p, label)))
        }
    }
}

/// A fill's background colour.
pub fn paint_rgb(project: &Project, paint: &Paint) -> Result<[u8; 3], String> {
    match paint {
        Paint::Hex { hex } => parse_hex(hex).ok_or_else(|| format!("couleur invalide : {hex}")),
        Paint::BrandColor { color_id, .. } => {
            let c = project.data.colors.iter().find(|c| &c.id == color_id).ok_or_else(|| format!("la couleur {color_id} n'est plus dans la palette"))?;
            parse_hex(&c.hex).ok_or_else(|| format!("couleur invalide : {}", c.hex))
        }
    }
}

/// Draws a design (logo, picture) at the size it takes in its zone at `out_scale`.
pub fn draw_content(engines: &Engines, file: &Path, template: &Template, zone: usize, fill: &Fill, out_scale: f64) -> Result<RgbaImage, String> {
    let natural = rasterize(engines, file, 1.0)?;
    let (iw, ih) = (f64::from(natural.width().max(1)), f64::from(natural.height().max(1)));
    let probe = ZoneDesign { fit: fill.fit, scale: fill.scale, ..Default::default() };
    let (nw, nh) = crate::composite::content_pixels(template, zone, &probe, iw / ih, out_scale);
    let k = (nw / iw).max(nh / ih).min(MAX_CONTENT_SIDE / iw.max(ih));
    if master_kind(file) == Some("vector") {
        // Vectors are drawn again at the size needed: sharp at any scale.
        if (k - 1.0).abs() < 0.02 { Ok(natural) } else { rasterize(engines, file, k as f32) }
    } else if k < 0.98 {
        // Pictures are only ever reduced (with a proper filter), never enlarged beyond their pixels.
        let (w, h) = ((iw * k).round().max(1.0) as u32, (ih * k).round().max(1.0) as u32);
        Ok(image::imageops::resize(&natural, w, h, image::imageops::FilterType::Lanczos3))
    } else {
        Ok(natural)
    }
}

/// Everything a scene's render depends on, with its current hash.
pub fn current_inputs(project: &Project, scene: &Scene, template_file: &Path, template: &Template) -> Result<Vec<Input>, String> {
    let dir = template_file.parent().unwrap_or(Path::new("."));
    let mut inputs = Vec::new();
    fn push(inputs: &mut Vec<Input>, key: String, label: String, path: &Path) -> Result<(), String> {
        let hash = cached_hash(path).map_err(|_| format!("{label} : fichier introuvable ({})", path.display()))?;
        inputs.push(Input { key, label, hash });
        Ok(())
    }
    push(&mut inputs, "template".into(), format!("modèle « {} »", template.name), template_file)?;
    push(&mut inputs, "photo".into(), "photo du modèle".into(), &resolve_with(dir, &template.photo))?;
    for z in &template.zones {
        if let Some(m) = &z.mask {
            push(&mut inputs, format!("mask:{}", z.id), format!("masque de « {} »", z.name), &resolve_with(dir, m))?;
        }
    }
    if let Some(m) = &template.occlusion {
        push(&mut inputs, "occlusion".into(), "masque de premier plan".into(), &resolve_with(dir, m))?;
    }
    for (i, o) in template.overlays.iter().enumerate() {
        push(&mut inputs, format!("overlay:{i}"), format!("surcouche {}", i + 1), &resolve_with(dir, &o.file))?;
    }
    for z in &template.zones {
        let Some(fill) = scene.fill(&z.id) else { continue };
        if let Some((file, label)) = content_file(project, &fill.content)? {
            let key = match &fill.content {
                Content::BrandAsset { asset_id, .. } => format!("content:{}:{asset_id}", z.id),
                _ => format!("content:{}:file", z.id),
            };
            let what = if matches!(fill.content, Content::BrandAsset { .. }) { "logo" } else { "image" };
            push(&mut inputs, key, format!("{what} « {label} »"), &file)?;
        }
        if let Some(p) = &fill.background {
            let rgb = paint_rgb(project, p)?;
            inputs.push(Input { key: format!("background:{}", z.id), label: format!("couleur de « {} »", z.name), hash: format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]) });
        }
    }
    // The scene's own settings (placement, fit, output), without the record of its last render.
    let mut settings = scene.clone();
    settings.last_render = None;
    let json = serde_json::to_vec(&settings).map_err(|e| e.to_string())?;
    inputs.push(Input { key: "scene".into(), label: "réglages de la scène".into(), hash: sha256_hex(&json) });
    Ok(inputs)
}

/// Where a scene's render stands, from the hashes recorded at its last render.
pub fn scene_state(project: &Project, scene: &Scene, template_roots: &[PathBuf]) -> SceneState {
    let current = find_template(project, scene, template_roots).and_then(|f| {
        let (f, t) = Template::load(&f)?;
        current_inputs(project, scene, &f, &t)
    });
    let current = match current {
        Ok(c) => c,
        Err(e) => return SceneState::Broken(e),
    };
    let Some(rec) = &scene.last_render else { return SceneState::NeverRendered };
    if !project.resolve(&rec.file).is_file() {
        return SceneState::Outdated(vec!["rendu introuvable".into()]);
    }
    let mut why: Vec<String> = current.iter().filter(|c| !rec.inputs.iter().any(|r| r.key == c.key && r.hash == c.hash)).map(|c| c.label.clone()).collect();
    why.extend(rec.inputs.iter().filter(|r| !current.iter().any(|c| c.key == r.key)).map(|r| format!("{} retiré", r.label)));
    if why.is_empty() { SceneState::UpToDate } else { SceneState::Outdated(why) }
}

/// Draws a scene `width` pixels wide (`None`: its output width), from the files as they are now.
pub fn render_scene(project: &Project, scene: &Scene, engines: &Engines, template_roots: &[PathBuf], width: Option<u32>, quality: Quality) -> Result<(RgbaImage, Vec<Input>), String> {
    let template_file = find_template(project, scene, template_roots)?;
    let (template_file, template) = Template::load(&template_file)?;
    let inputs = current_inputs(project, scene, &template_file, &template)?;
    let images = load_template_images(&template_file, &template, engines)?;
    let out_width = width.or(scene.output.width).unwrap_or(images.photo.width());
    let out_scale = f64::from(out_width) / f64::from(images.photo.width().max(1));
    let designs = designs_for(project, scene, &template, engines, out_scale)?;
    let img = crate::composite::compose(&template, &images, &designs, out_width, quality)?;
    Ok((img, inputs))
}

/// Each zone's design, drawn at `out_scale`.
pub fn designs_for(project: &Project, scene: &Scene, template: &Template, engines: &Engines, out_scale: f64) -> Result<Vec<Option<ZoneDesign>>, String> {
    let mut designs = Vec::new();
    for (i, z) in template.zones.iter().enumerate() {
        let Some(fill) = scene.fill(&z.id) else {
            designs.push(None);
            continue;
        };
        let content = match content_file(project, &fill.content)? {
            Some((file, label)) => Some(draw_content(engines, &file, template, i, fill, out_scale).map_err(|e| format!("« {label} » : {e}"))?),
            None => None,
        };
        let background = fill.background.as_ref().map(|p| paint_rgb(project, p)).transpose()?;
        designs.push(Some(ZoneDesign { content, background, fit: fill.fit, scale: fill.scale, rotation: fill.rotation, offset: fill.offset }));
    }
    Ok(designs)
}

/// Renders a scene to its file in the project (`Mockups/<scène>.<ext>`, written atomically: a
/// failed render leaves the previous file), records what went in, and saves the scene.
pub fn export_scene(project: &Project, scene_file: &Path, engines: &Engines, template_roots: &[PathBuf]) -> Result<PathBuf, String> {
    let mut scene = Scene::load(scene_file)?;
    if !scene.output.format.is_raster() {
        return Err("un mockup s'exporte en PNG, JPEG ou WebP".into());
    }
    let (img, inputs) = render_scene(project, &scene, engines, template_roots, None, Quality::FINAL)?;
    let out = scene.render_path(project);
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("impossible de créer {} : {e}", dir.display()))?;
    }
    let preset = ExportPreset {
        id: "mockup".into(),
        name: "Mockup".into(),
        format: scene.output.format,
        width: None,
        height: None,
        scale: 1.0,
        margin: 0.0,
        // JPEG has no transparency: a photo with transparent parts is flattened onto white.
        background: (scene.output.format == Format::Jpeg).then(|| "#FFFFFF".into()),
        dpi: None,
        quality: scene.output.quality,
        folder: String::new(),
        naming: String::new(),
        roles: Vec::new(),
    };
    crate::render::write_raster(&img, &preset, &out)?;
    scene.last_render = Some(RenderRecord {
        file: project.store_path(&out),
        at: crate::now_rfc3339(),
        width: img.width(),
        height: img.height(),
        sha256: hash_file(&out)?,
        inputs,
    });
    scene.save(scene_file)?;
    Ok(out)
}

// --- Files --------------------------------------------------------------------------------------

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, what: &str) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{what} {} illisible : {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{what} {} invalide : {e}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let json = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("impossible d'écrire {} : {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("impossible d'enregistrer {} : {e}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// [`hash_file`], remembered per (path, size, modification time): photos can be large and the
/// state of every scene is checked often.
pub fn cached_hash(path: &Path) -> Result<String, String> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    type Key = (PathBuf, u64, Option<std::time::SystemTime>);
    static CACHE: OnceLock<Mutex<HashMap<Key, String>>> = OnceLock::new();
    let meta = std::fs::metadata(path).map_err(|e| format!("{} : {e}", path.display()))?;
    let key = (path.to_path_buf(), meta.len(), meta.modified().ok());
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(h) = cache.lock().ok().and_then(|c| c.get(&key).cloned()) {
        return Ok(h);
    }
    let h = hash_file(path)?;
    if let Ok(mut c) = cache.lock() {
        c.insert(key, h.clone());
    }
    Ok(h)
}
