//! Client deliveries: plan the files (shown before anything is written), make them, and describe
//! them in `manifest.json` and `README.txt`.
//!
//! A delivery lives in `Livrables/<marque>/<dossier>/`. A delivery marked as sent to the client is
//! never written again: regenerating makes a new folder (`v1-2`, `v1-3`…). Each file is written
//! atomically, so a failed export leaves the previous files as they were.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::model::{Asset, Delivery, ExportPreset, Format};
use crate::project::{Project, SourceState, hash_file, safe_name};
use crate::render::{self, Engines};

/// The preset id of a delivered Mockup Studio render (its `asset_id` is the scene's id).
pub const MOCKUP_PRESET: &str = "mockup";

/// One file the delivery will hold.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedOutput {
    pub asset_id: String,
    pub preset_id: String,
    /// Relative to the delivery folder, with `/`.
    pub path: String,
    pub format: Format,
    /// A problem known before rendering (a JPEG without background, a raster master asked as SVG…).
    pub blocked: Option<String>,
}

/// The whole plan: the folder and its files.
#[derive(Clone, Debug)]
pub struct Plan {
    pub folder: String,
    pub outputs: Vec<PlannedOutput>,
}

/// The result of one file.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputResult {
    pub path: String,
    pub asset_id: String,
    pub preset_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub folder: PathBuf,
    pub results: Vec<OutputResult>,
    pub cancelled: bool,
}

impl Report {
    pub fn failures(&self) -> impl Iterator<Item = &OutputResult> {
        self.results.iter().filter(|r| r.error.is_some())
    }
    pub fn succeeded(&self) -> usize {
        self.results.iter().filter(|r| r.error.is_none()).count()
    }
}

/// Fills a preset's naming pattern. Unknown `{fields}` stay as written.
pub fn file_stem(pattern: &str, project: &Project, asset: &Asset, preset: &ExportPreset) -> String {
    let brand = safe_name(&project.data.brand_name).replace(' ', "-");
    let w = preset.width.map(|w| w.to_string()).unwrap_or_default();
    let h = preset.height.map(|h| h.to_string()).unwrap_or_default();
    let filled = pattern
        .replace("{marque}", &brand)
        .replace("{role}", asset.role.slug())
        .replace("{variante}", asset.variant.slug())
        .replace("{preset}", &safe_name(&preset.name).replace(' ', "-"))
        .replace("{version}", &project.data.version)
        .replace("{w}", &w)
        .replace("{h}", &h);
    safe_name(&filled)
}

/// The folder the next export writes to: the latest delivery if it hasn't been sent, else a new one.
pub fn next_folder(project: &Project) -> String {
    let base = safe_name(&project.data.version);
    match project.data.deliveries.last() {
        Some(d) if !d.delivered => d.folder.clone(),
        Some(_) => {
            let mut n = 2;
            loop {
                let candidate = format!("{base}-{n}");
                if !project.data.deliveries.iter().any(|d| d.folder == candidate) && !project.deliveries_root().join(&candidate).exists() {
                    return candidate;
                }
                n += 1;
            }
        }
        None => base,
    }
}

/// The files the given presets make from the given assets (all when the lists are empty).
pub fn plan(project: &Project, asset_ids: &[String], preset_ids: &[String]) -> Plan {
    let folder = next_folder(project);
    let mut outputs = Vec::new();
    let mut taken: BTreeMap<String, usize> = BTreeMap::new();
    for preset in project.data.export_presets.iter().filter(|p| preset_ids.is_empty() || preset_ids.contains(&p.id)) {
        for asset in project.data.assets.iter().filter(|a| asset_ids.is_empty() || asset_ids.contains(&a.id)) {
            if !preset.roles.is_empty() && !preset.roles.contains(&asset.role) {
                continue;
            }
            let stem = file_stem(&preset.naming, project, asset, preset);
            let mut path = format!("{}/{stem}.{}", preset.folder.trim_matches('/'), preset.format.ext());
            // Two outputs with one name: number the later ones rather than overwrite.
            let n = taken.entry(path.clone()).or_insert(0);
            *n += 1;
            if *n > 1 {
                path = format!("{}/{stem}-{}.{}", preset.folder.trim_matches('/'), n, preset.format.ext());
            }
            let src = project.resolve(&asset.source);
            let blocked = if !src.is_file() {
                Some(format!("fichier maître introuvable : {}", src.display()))
            } else if preset.format == Format::Jpeg && preset.background.is_none() {
                Some("un JPEG n'a pas de transparence : choisissez une couleur de fond dans le preset".into())
            } else if !preset.format.is_raster() && render::master_kind(&src) != Some("vector") {
                Some(format!("le maître n'est pas vectoriel : pas d'export {} sans vectorisation", preset.format.ext().to_uppercase()))
            } else if render::master_kind(&src).is_none() {
                Some("format de fichier maître non pris en charge".into())
            } else {
                None
            };
            outputs.push(PlannedOutput { asset_id: asset.id.clone(), preset_id: preset.id.clone(), path, format: preset.format, blocked });
        }
    }
    Plan { folder, outputs }
}

/// Adds the Mockup Studio renders of the project's scenes to a plan, as `Mockups/<file>`: copied
/// byte for byte as rendered. A scene never rendered, or whose render is out of date (a logo, a
/// mask or a setting changed since), is listed as blocked with the reason: never delivered stale.
pub fn plan_mockups(project: &Project, plan: &mut Plan, template_roots: &[PathBuf]) {
    use crate::mockup::{SceneState, scene_state, scenes};
    for (_, _, s) in scenes(project) {
        let Ok(scene) = s else { continue };
        let file = scene.last_render.as_ref().and_then(|r| Path::new(&r.file).file_name().map(|n| n.to_string_lossy().to_string()));
        let name = file.unwrap_or_else(|| format!("{}.{}", safe_name(&scene.name), scene.output.format.ext()));
        let blocked = match scene_state(project, &scene, template_roots) {
            SceneState::UpToDate => None,
            SceneState::NeverRendered => Some("scène jamais rendue dans Mockup Studio".to_string()),
            SceneState::Outdated(why) => Some(format!("rendu à mettre à jour dans Mockup Studio ({})", why.join(", "))),
            SceneState::Broken(e) => Some(e),
        };
        plan.outputs.push(PlannedOutput { asset_id: scene.id, preset_id: MOCKUP_PRESET.into(), path: format!("Mockups/{name}"), format: scene.output.format, blocked });
    }
}

/// Makes the planned files. `only` limits the run to some paths (a retry of failures); `progress`
/// is called after each file with (done, total, path). Setting `cancel` stops after the current file.
pub fn run(
    project: &mut Project,
    engines: &Engines,
    plan: &Plan,
    only: Option<&[String]>,
    cancel: &AtomicBool,
    mut progress: impl FnMut(usize, usize, &str),
) -> Report {
    let root = project.deliveries_root().join(&plan.folder);
    let todo: Vec<&PlannedOutput> = plan.outputs.iter().filter(|o| only.is_none_or(|only| only.contains(&o.path))).collect();
    let mut results = Vec::new();
    let mut cancelled = false;
    for (i, out) in todo.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }
        let result = make_one(project, engines, &root, out);
        progress(i + 1, todo.len(), &out.path);
        results.push(result);
    }
    // Merge with an earlier manifest of this folder, so a retry keeps the files already made.
    let previous = read_manifest_results(&root.join("manifest.json"));
    let mut merged: BTreeMap<String, OutputResult> = previous.into_iter().map(|r| (r.path.clone(), r)).collect();
    for r in &results {
        merged.insert(r.path.clone(), r.clone());
    }
    // A regeneration after a delivery: the unchanged files come along, so the folder is complete.
    if !cancelled {
        let made: std::collections::BTreeSet<String> = merged.keys().cloned().collect();
        for r in complete_from_delivered(project, plan, &root, &made, false) {
            merged.insert(r.path.clone(), r.clone());
            results.push(r);
        }
    }
    let all: Vec<OutputResult> = merged.into_values().collect();
    let made_ids: Vec<String> = results.iter().filter(|r| r.error.is_none()).map(|r| r.asset_id.clone()).collect();
    project.record_hashes(&made_ids);
    if let Err(e) = write_manifest(project, &root, &all) {
        results.push(OutputResult { path: "manifest.json".into(), asset_id: String::new(), preset_id: String::new(), error: Some(e), width: None, height: None, bytes: None, sha256: None });
    }
    if let Err(e) = write_readme(project, &root, &all) {
        results.push(OutputResult { path: "README.txt".into(), asset_id: String::new(), preset_id: String::new(), error: Some(e), width: None, height: None, bytes: None, sha256: None });
    }
    let made = all.iter().filter(|r| r.error.is_none()).count();
    match project.data.deliveries.iter_mut().find(|d| d.folder == plan.folder) {
        Some(d) => d.outputs = made,
        None => project.data.deliveries.push(Delivery { folder: plan.folder.clone(), created_at: crate::now_rfc3339(), delivered: false, outputs: made }),
    }
    Report { folder: root, results, cancelled }
}

/// A new folder made after a delivery holds a complete set: each file of the last delivered
/// folder that this run didn't make is copied from it byte for byte (checked against that
/// folder's manifest), when what it was made from hasn't changed since: its master's hash as
/// recorded then, or, for a mockup, the same render. Anything changed and not regenerated stays
/// out rather than going out stale.
fn complete_from_delivered(project: &Project, plan: &Plan, root: &Path, made: &std::collections::BTreeSet<String>, dry_run: bool) -> Vec<OutputResult> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Source {
        id: String,
        sha256: Option<String>,
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Prev {
        #[serde(default)]
        sources: Vec<Source>,
    }
    let Some(prev) = project.data.deliveries.iter().rev().find(|d| d.delivered && d.folder != plan.folder) else { return Vec::new() };
    let prev_root = project.deliveries_root().join(&prev.folder);
    let manifest = prev_root.join("manifest.json");
    let sources: Vec<Source> = std::fs::read(&manifest).ok().and_then(|b| serde_json::from_slice::<Prev>(&b).ok()).map(|p| p.sources).unwrap_or_default();
    let renders: BTreeMap<String, PathBuf> = crate::mockup::scenes(project)
        .into_iter()
        .filter_map(|(_, _, s)| s.ok())
        .filter_map(|s| s.last_render.map(|r| (s.id, project.resolve(&r.file))))
        .collect();
    let mut out = Vec::new();
    for old in read_manifest_results(&manifest) {
        let Some(sha) = old.sha256.clone().filter(|_| old.error.is_none()) else { continue };
        if made.contains(&old.path) {
            continue;
        }
        let unchanged = if old.preset_id == MOCKUP_PRESET {
            renders.get(&old.asset_id).and_then(|r| hash_file(r).ok()).as_deref() == Some(sha.as_str())
        } else {
            let then = sources.iter().find(|s| s.id == old.asset_id).and_then(|s| s.sha256.clone());
            let now = project.data.assets.iter().find(|a| a.id == old.asset_id).and_then(|a| hash_file(&project.resolve(&a.source)).ok());
            then.is_some() && then == now
        };
        if !unchanged {
            continue;
        }
        if dry_run {
            out.push(old);
            continue;
        }
        let from = crate::project::resolve_with(&prev_root, &old.path);
        let to = crate::project::resolve_with(root, &old.path);
        let mut r = old.clone();
        let copied = std::fs::read(&from).map_err(|e| format!("{} : {e}", from.display())).and_then(|bytes| {
            if hash_file(&from).ok().as_deref() != Some(sha.as_str()) {
                return Err(format!("{} ne correspond plus au manifeste de {} : non repris", old.path, prev.folder));
            }
            if let Some(dir) = to.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("{} : {e}", dir.display()))?;
            }
            write_atomic(&to, &bytes)
        });
        if let Err(e) = copied {
            r.error = Some(e);
            r.sha256 = None;
        }
        out.push(r);
    }
    out
}

/// What an export of `plan` would take over unchanged from the last delivered folder (its name
/// and the files), shown with the plan before anything is written.
pub fn carried_over(project: &Project, plan: &Plan) -> Option<(String, Vec<String>)> {
    let prev = project.data.deliveries.iter().rev().find(|d| d.delivered && d.folder != plan.folder)?;
    let root = project.deliveries_root().join(&plan.folder);
    let mut made: std::collections::BTreeSet<String> = plan.outputs.iter().map(|o| o.path.clone()).collect();
    made.extend(read_manifest_results(&root.join("manifest.json")).into_iter().map(|r| r.path));
    let files: Vec<String> = complete_from_delivered(project, plan, &root, &made, true).into_iter().map(|r| r.path).collect();
    (!files.is_empty()).then(|| (prev.folder.clone(), files))
}

fn make_one(project: &Project, engines: &Engines, root: &Path, out: &PlannedOutput) -> OutputResult {
    let mut r = OutputResult {
        path: out.path.clone(),
        asset_id: out.asset_id.clone(),
        preset_id: out.preset_id.clone(),
        error: None,
        width: None,
        height: None,
        bytes: None,
        sha256: None,
    };
    if let Some(b) = &out.blocked {
        r.error = Some(b.clone());
        return r;
    }
    if out.preset_id == MOCKUP_PRESET {
        let render = crate::mockup::scenes(project)
            .into_iter()
            .filter_map(|(_, _, s)| s.ok())
            .find(|s| s.id == out.asset_id)
            .and_then(|s| s.last_render.map(|rec| project.resolve(&rec.file)));
        let dest = crate::project::resolve_with(root, &out.path);
        let copied = render.ok_or_else(|| "scène ou rendu introuvable".to_string()).and_then(|src| {
            let bytes = std::fs::read(&src).map_err(|e| format!("{} : {e}", src.display()))?;
            if let Some(dir) = dest.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("{} : {e}", dir.display()))?;
            }
            write_atomic(&dest, &bytes)
        });
        match copied {
            Ok(()) => {
                if let Ok((w, h)) = image::image_dimensions(&dest) {
                    r.width = Some(w);
                    r.height = Some(h);
                }
                r.bytes = std::fs::metadata(&dest).ok().map(|m| m.len());
                r.sha256 = hash_file(&dest).ok();
            }
            Err(e) => r.error = Some(e),
        }
        return r;
    }
    let (Some(asset), Some(preset)) = (project.data.assets.iter().find(|a| a.id == out.asset_id), project.data.export_presets.iter().find(|p| p.id == out.preset_id))
    else {
        r.error = Some("logo ou preset supprimé depuis le plan".into());
        return r;
    };
    let src = project.resolve(&asset.source);
    let dest = crate::project::resolve_with(root, &out.path);
    let made = if preset.format.is_raster() {
        render::raster_export(engines, &src, preset).and_then(|img| {
            r.width = Some(img.width());
            r.height = Some(img.height());
            render::write_raster(&img, preset, &dest)
        })
    } else {
        render::vector_export(engines, &src, preset.format, &dest)
    };
    match made {
        Ok(()) => {
            r.bytes = std::fs::metadata(&dest).ok().map(|m| m.len());
            r.sha256 = hash_file(&dest).ok();
        }
        Err(e) => r.error = Some(e),
    }
    r
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest<'a> {
    schema_version: u32,
    generator: &'static str,
    brand: &'a str,
    project_id: &'a str,
    version: &'a str,
    generated_at: String,
    sources: Vec<ManifestSource>,
    presets: &'a [ExportPreset],
    outputs: &'a [OutputResult],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestSource {
    id: String,
    role: &'static str,
    variant: &'static str,
    label: String,
    /// As stored in the project (relative when inside it).
    source: String,
    sha256: Option<String>,
}

fn write_manifest(project: &Project, root: &Path, outputs: &[OutputResult]) -> Result<(), String> {
    let sources = project
        .data
        .assets
        .iter()
        .map(|a| ManifestSource {
            id: a.id.clone(),
            role: a.role.slug(),
            variant: a.variant.slug(),
            label: a.label.clone(),
            source: a.source.clone(),
            sha256: hash_file(&project.resolve(&a.source)).ok(),
        })
        .collect();
    let m = Manifest {
        schema_version: 1,
        generator: concat!("Oracle Brand Kit ", env!("CARGO_PKG_VERSION")),
        brand: &project.data.brand_name,
        project_id: &project.data.project_id,
        version: &project.data.version,
        generated_at: crate::now_rfc3339(),
        sources,
        presets: &project.data.export_presets,
        outputs,
    };
    std::fs::create_dir_all(root).map_err(|e| format!("{} : {e}", root.display()))?;
    let json = serde_json::to_vec_pretty(&m).map_err(|e| e.to_string())?;
    write_atomic(&root.join("manifest.json"), &json)
}

fn read_manifest_results(path: &Path) -> Vec<OutputResult> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Prev {
        #[serde(default)]
        outputs: Vec<PrevOut>,
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PrevOut {
        path: String,
        #[serde(default)]
        asset_id: String,
        #[serde(default)]
        preset_id: String,
        error: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        bytes: Option<u64>,
        sha256: Option<String>,
    }
    let Ok(bytes) = std::fs::read(path) else { return Vec::new() };
    let Ok(prev) = serde_json::from_slice::<Prev>(&bytes) else { return Vec::new() };
    prev.outputs
        .into_iter()
        .map(|p| OutputResult { path: p.path, asset_id: p.asset_id, preset_id: p.preset_id, error: p.error, width: p.width, height: p.height, bytes: p.bytes, sha256: p.sha256 })
        .collect()
}

fn write_readme(project: &Project, root: &Path, outputs: &[OutputResult]) -> Result<(), String> {
    let d = &project.data;
    let mut s = String::new();
    s.push_str(&format!("{} — fichiers de marque ({})\n", d.brand_name, d.version));
    s.push_str(&"=".repeat(60));
    s.push_str("\n\n");
    if !d.client.is_empty() {
        s.push_str(&format!("Client : {}\n", d.client));
    }
    s.push_str(&format!("Généré le : {}\n\n", crate::now_rfc3339()));
    s.push_str("CONTENU\n-------\n");
    let mut by_folder: BTreeMap<String, Vec<&OutputResult>> = BTreeMap::new();
    for o in outputs.iter().filter(|o| o.error.is_none()) {
        let folder = o.path.rsplit_once('/').map(|(f, _)| f.to_string()).unwrap_or_default();
        by_folder.entry(folder).or_default().push(o);
    }
    for (folder, files) in &by_folder {
        s.push_str(&format!("{folder}/ ({} fichier{})\n", files.len(), if files.len() > 1 { "s" } else { "" }));
        for f in files {
            let size = match (f.width, f.height) {
                (Some(w), Some(h)) => format!(" — {w}×{h} px"),
                _ => String::new(),
            };
            s.push_str(&format!("  {}{size}\n", f.path.rsplit('/').next().unwrap_or(&f.path)));
        }
    }
    s.push_str("\nFormats : SVG et PDF sont vectoriels (agrandissables sans perte). PNG et WebP gardent la\n");
    s.push_str("transparence. JPEG n'a pas de transparence (fond indiqué dans le nom du fichier).\n");
    if !d.colors.is_empty() {
        s.push_str("\nCOULEURS\n--------\n");
        for c in &d.colors {
            let rgb = crate::model::parse_hex(&c.hex).map(|[r, g, b]| format!("RVB {r} {g} {b}")).unwrap_or_default();
            s.push_str(&format!("{} — {} {rgb}", c.name, c.hex.to_uppercase()));
            if let Some(k) = &c.cmyk {
                let prof = if k.profile.is_empty() { "valeurs saisies".to_string() } else { k.profile.clone() };
                s.push_str(&format!(" — CMJN {} {} {} {} ({prof})", k.c, k.m, k.y, k.k));
            }
            if !c.pantone.is_empty() {
                s.push_str(&format!(" — Pantone {}", c.pantone));
            }
            if !c.ral.is_empty() {
                s.push_str(&format!(" — RAL {}", c.ral));
            }
            s.push('\n');
        }
    }
    if !d.fonts.is_empty() {
        s.push_str("\nTYPOGRAPHIES\n------------\n");
        for f in &d.fonts {
            s.push_str(&format!("{} {} — {}", f.family, f.weights.join(", "), f.usage));
            if !f.license_url.is_empty() {
                s.push_str(&format!(" — licence : {}", f.license_url));
            }
            s.push('\n');
        }
        s.push_str("Les fichiers de police ne sont pas fournis sauf mention contraire : utilisez votre propre licence.\n");
    }
    let r = &d.rules;
    if !r.clear_space.is_empty() || !r.min_size.is_empty() || !r.allowed_backgrounds.is_empty() || !r.forbidden.is_empty() {
        s.push_str("\nRÈGLES D'USAGE\n--------------\n");
        if !r.clear_space.is_empty() {
            s.push_str(&format!("Zone de protection : {}\n", r.clear_space));
        }
        if !r.min_size.is_empty() {
            s.push_str(&format!("Taille minimale : {}\n", r.min_size));
        }
        if !r.allowed_backgrounds.is_empty() {
            s.push_str(&format!("Fonds autorisés : {}\n", r.allowed_backgrounds));
        }
        for f in &r.forbidden {
            s.push_str(&format!("À éviter : {}\n", f.text));
        }
    }
    s.push_str("\nmanifest.json décrit chaque fichier (source, paramètres, empreinte SHA-256).\n");
    write_atomic(&root.join("README.txt"), s.replace('\n', "\r\n").as_bytes())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes).map_err(|e| format!("{} : {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{} : {e}", path.display()))
}

/// What a regeneration would change: masters modified since the last export, and the outputs that
/// come from them.
pub fn stale(project: &Project) -> Vec<(String, SourceState)> {
    project.data.assets.iter().map(|a| (a.id.clone(), project.source_state(a))).filter(|(_, s)| *s != SourceState::Unchanged).collect()
}
