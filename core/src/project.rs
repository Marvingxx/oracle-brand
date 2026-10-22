//! A brand project on disk: a folder holding `brand.json`, next to (or around) the master files.
//!
//! Opening migrates older schemas forward and never loses unknown data silently: a file from a
//! newer Oracle is refused rather than rewritten. Saving is atomic (temporary file, then rename) and
//! keeps the previous file as `brand.json.bak`.

use std::path::{Component, Path, PathBuf};

use crate::model::{Asset, BrandProject, SCHEMA_VERSION};

pub const FILE_NAME: &str = "brand.json";

#[derive(Debug, Clone)]
pub struct Project {
    /// The project folder.
    pub dir: PathBuf,
    pub data: BrandProject,
}

/// Where a master file stands compared with the hash recorded at its last export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceState {
    /// Same bytes as recorded.
    Unchanged,
    /// Different bytes: derived files are out of date.
    Modified,
    /// No file at the path.
    Missing,
    /// Never exported or checked yet.
    New,
}

impl SourceState {
    pub fn label_fr(self) -> &'static str {
        match self {
            SourceState::Unchanged => "À jour",
            SourceState::Modified => "Modifié depuis le dernier export",
            SourceState::Missing => "Fichier introuvable",
            SourceState::New => "Jamais exporté",
        }
    }
}

impl Project {
    /// Creates a new project folder (or uses an existing empty one) and saves `brand.json`.
    pub fn create(dir: &Path, brand_name: &str) -> Result<Project, String> {
        if dir.join(FILE_NAME).exists() {
            return Err(format!("{} contient déjà un projet de marque", dir.display()));
        }
        std::fs::create_dir_all(dir).map_err(|e| format!("impossible de créer {} : {e}", dir.display()))?;
        let p = Project { dir: dir.to_path_buf(), data: BrandProject::new(brand_name) };
        p.save()?;
        Ok(p)
    }

    /// Opens the project in `dir` (or the folder of a `brand.json` path).
    pub fn open(path: &Path) -> Result<Project, String> {
        let file = if path.is_dir() { path.join(FILE_NAME) } else { path.to_path_buf() };
        let dir = file.parent().map(Path::to_path_buf).ok_or_else(|| format!("{} n'a pas de dossier parent", file.display()))?;
        let bytes = std::fs::read(&file).map_err(|e| format!("impossible de lire {} : {e}", file.display()))?;
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| format!("{} n'est pas un JSON valide : {e}", file.display()))?;
        migrate(&mut value)?;
        let data: BrandProject = serde_json::from_value(value).map_err(|e| format!("{} est illisible : {e}", file.display()))?;
        let dupes = data.duplicate_ids();
        if !dupes.is_empty() {
            return Err(format!("identifiants en double dans {} : {}", file.display(), dupes.join(", ")));
        }
        Ok(Project { dir, data })
    }

    /// Writes `brand.json` atomically, keeping the previous one as `brand.json.bak`.
    pub fn save(&self) -> Result<(), String> {
        let file = self.dir.join(FILE_NAME);
        let json = serde_json::to_vec_pretty(&self.data).map_err(|e| e.to_string())?;
        let tmp = self.dir.join(format!("{FILE_NAME}.tmp"));
        std::fs::write(&tmp, &json).map_err(|e| format!("impossible d'écrire {} : {e}", tmp.display()))?;
        if file.exists() {
            let _ = std::fs::copy(&file, self.dir.join(format!("{FILE_NAME}.bak")));
        }
        std::fs::rename(&tmp, &file).map_err(|e| format!("impossible d'enregistrer {} : {e}", file.display()))
    }

    /// A source path as stored: relative (with `/`) when inside the project folder.
    pub fn store_path(&self, path: &Path) -> String {
        let abs = absolute(path);
        match abs.strip_prefix(absolute(&self.dir)) {
            Ok(rel) => rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"),
            Err(_) => abs.display().to_string(),
        }
    }

    /// The file a stored path points at.
    pub fn resolve(&self, stored: &str) -> PathBuf {
        resolve_with(&self.dir, stored)
    }

    /// Where `asset`'s master file stands.
    pub fn source_state(&self, asset: &Asset) -> SourceState {
        let path = self.resolve(&asset.source);
        let Ok(hash) = hash_file(&path) else { return SourceState::Missing };
        if asset.source_hash.is_empty() {
            SourceState::New
        } else if asset.source_hash == hash {
            SourceState::Unchanged
        } else {
            SourceState::Modified
        }
    }

    /// Records each master's current hash (after a successful export).
    pub fn record_hashes(&mut self, asset_ids: &[String]) {
        for a in self.data.assets.iter_mut() {
            if asset_ids.contains(&a.id) {
                let path = resolve_with(&self.dir, &a.source);
                if let Ok(h) = hash_file(&path) {
                    a.source_hash = h;
                }
            }
        }
    }

    /// The delivery root: `Livrables/<marque>/`.
    pub fn deliveries_root(&self) -> PathBuf {
        self.dir.join("Livrables").join(safe_name(&self.data.brand_name))
    }
}

/// The file a path stored in a project in `dir` points at.
pub fn resolve_with(dir: &Path, stored: &str) -> PathBuf {
    let p = Path::new(stored);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let mut out = dir.to_path_buf();
    for part in stored.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            p => out.push(p),
        }
    }
    out
}

/// Brings an older `brand.json` up to the current schema. Each step is tested.
fn migrate(value: &mut serde_json::Value) -> Result<(), String> {
    let version = value.get("schemaVersion").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
    if version > SCHEMA_VERSION {
        return Err(format!(
            "ce projet a été enregistré par une version plus récente d'Oracle (schéma {version}, celle-ci lit jusqu'au {SCHEMA_VERSION}) : il n'est pas modifié"
        ));
    }
    if version == 0 {
        // Schema 0 (drafts before versioning) had no id and called the brand `name`.
        let obj = value.as_object_mut().ok_or("le projet n'est pas un objet JSON")?;
        if let Some(name) = obj.remove("name") {
            obj.entry("brandName").or_insert(name);
        }
        obj.entry("projectId").or_insert_with(|| serde_json::Value::String(uuid::Uuid::new_v4().to_string()));
        obj.insert("schemaVersion".into(), serde_json::Value::from(1));
    }
    Ok(())
}

/// SHA-256 of a file, as lower-case hex.
pub fn hash_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut f = std::fs::File::open(path).map_err(|e| format!("{} : {e}", path.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).map_err(|e| format!("{} : {e}", path.display()))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// A name safe as a file or folder name on Windows, macOS and Linux (accents kept).
pub fn safe_name(name: &str) -> String {
    let cleaned: String = name.trim().chars().map(|c| if c.is_control() || r#"<>:"/\|?*"#.contains(c) { '-' } else { c }).collect();
    let cleaned = cleaned.trim_matches(|c: char| c == '.' || c == ' ').to_string();
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "LPT1", "LPT2", "LPT3"];
    if cleaned.is_empty() {
        "sans-nom".into()
    } else if reserved.contains(&cleaned.to_ascii_uppercase().as_str()) {
        format!("{cleaned}_")
    } else {
        cleaned
    }
}

fn absolute(p: &Path) -> PathBuf {
    let p = std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
    // Normalise `..` without touching the disk (the file may not exist yet).
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Role, Variant};

    #[test]
    fn create_save_open_round_trip_with_accents_and_spaces() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("Projets clients").join("Marque Été");
        let mut p = Project::create(&dir, "Marque Été").unwrap();
        std::fs::create_dir_all(dir.join("Sources")).unwrap();
        std::fs::write(dir.join("Sources").join("logo é.svg"), b"<svg/>").unwrap();
        let stored = p.store_path(&dir.join("Sources").join("logo é.svg"));
        assert_eq!(stored, "Sources/logo é.svg");
        let id = p.data.new_id("asset");
        p.data.assets.push(Asset { id, role: Role::PrimaryLogo, label: String::new(), variant: Variant::Color, source: stored, source_hash: String::new() });
        p.save().unwrap();
        let back = Project::open(&dir).unwrap();
        assert_eq!(back.data, p.data);
        assert!(back.resolve(&back.data.assets[0].source).is_file());
    }

    #[test]
    fn a_moved_project_keeps_its_relative_links() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let mut p = Project::create(&a, "M").unwrap();
        std::fs::write(a.join("logo.svg"), b"<svg/>").unwrap();
        let id = p.data.new_id("asset");
        p.data.assets.push(Asset { id, role: Role::Symbol, label: String::new(), variant: Variant::Color, source: "logo.svg".into(), source_hash: String::new() });
        p.save().unwrap();
        let b = tmp.path().join("ailleurs").join("b");
        std::fs::create_dir_all(b.parent().unwrap()).unwrap();
        std::fs::rename(&a, &b).unwrap();
        let moved = Project::open(&b).unwrap();
        assert_eq!(moved.source_state(&moved.data.assets[0]), SourceState::New);
    }

    #[test]
    fn source_changes_are_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let mut p = Project::create(tmp.path(), "M").unwrap();
        std::fs::write(tmp.path().join("logo.svg"), b"<svg>1</svg>").unwrap();
        let id = p.data.new_id("asset");
        p.data.assets.push(Asset { id: id.clone(), role: Role::Symbol, label: String::new(), variant: Variant::Color, source: "logo.svg".into(), source_hash: String::new() });
        assert_eq!(p.source_state(&p.data.assets[0]), SourceState::New);
        p.record_hashes(&[id]);
        assert_eq!(p.source_state(&p.data.assets[0]), SourceState::Unchanged);
        std::fs::write(tmp.path().join("logo.svg"), b"<svg>2</svg>").unwrap();
        assert_eq!(p.source_state(&p.data.assets[0]), SourceState::Modified);
        std::fs::remove_file(tmp.path().join("logo.svg")).unwrap();
        assert_eq!(p.source_state(&p.data.assets[0]), SourceState::Missing);
    }

    #[test]
    fn schema_0_migrates_and_newer_schemas_are_refused() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(FILE_NAME), br#"{"name":"Ancienne marque"}"#).unwrap();
        let p = Project::open(tmp.path()).unwrap();
        assert_eq!(p.data.brand_name, "Ancienne marque");
        assert_eq!(p.data.schema_version, 1);
        std::fs::write(tmp.path().join(FILE_NAME), br#"{"schemaVersion":99,"projectId":"x","brandName":"Futur"}"#).unwrap();
        assert!(Project::open(tmp.path()).unwrap_err().contains("plus récente"));
    }

    #[test]
    fn a_corrupt_project_is_an_error_not_a_crash() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(FILE_NAME), b"{pas du json").unwrap();
        assert!(Project::open(tmp.path()).unwrap_err().contains("JSON"));
    }

    #[test]
    fn safe_names() {
        assert_eq!(safe_name("Marque: Été / 2026"), "Marque- Été - 2026");
        assert_eq!(safe_name("CON"), "CON_");
        assert_eq!(safe_name("  "), "sans-nom");
    }
}
