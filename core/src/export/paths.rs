//! Delivery paths are portable relative paths, even when a project is edited by hand.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::project::Project;

pub(super) fn relative(stored: &str) -> Result<String, String> {
    let parts: Vec<_> = stored.split(['/', '\\']).collect();
    for part in &parts {
        let device = part.split('.').next().unwrap_or_default().to_ascii_uppercase();
        let reserved = matches!(device.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ["COM", "LPT"].iter().any(|prefix| device.strip_prefix(prefix)
                .is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³")));
        if part.is_empty() || matches!(*part, "." | "..") || part.ends_with(['.', ' ']) || reserved
            || part.chars().any(|c| c.is_control() || "<>:\"|?*".contains(c)) {
            return Err(format!("chemin de livraison non sûr : {stored}"));
        }
    }
    Ok(parts.join("/"))
}

pub(super) fn key(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

/// Reserve the final name, including generated suffixes and Windows case aliases.
pub(super) fn unique(path: &str, taken: &mut BTreeSet<String>) -> String {
    let (stem, ext) = path.rsplit_once('.').unwrap_or((path, ""));
    let mut candidate = path.to_string();
    let mut n = 2;
    while !taken.insert(key(&candidate)) {
        candidate = format!("{stem}-{n}.{ext}");
        n += 1;
    }
    candidate
}

fn no_links(project: &Project, path: &Path) -> Result<(), String> {
    let suffix = path.strip_prefix(&project.dir).map_err(|_| "destination hors du projet".to_string())?;
    let mut current = project.dir.clone();
    for part in suffix.components() {
        current.push(part);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                #[cfg(windows)]
                let linked = {
                    use std::os::windows::fs::MetadataExt;
                    meta.file_attributes() & 0x400 != 0 // includes directory junctions
                };
                #[cfg(not(windows))]
                let linked = meta.file_type().is_symlink();
                if linked {
                    return Err(format!("lien ou jonction interdit dans une livraison : {}", current.display()));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
            Err(e) => return Err(format!("{} : {e}", current.display())),
        }
    }
    Ok(())
}

pub(super) fn root(project: &Project, folder: &str) -> Result<PathBuf, String> {
    let folder = relative(folder)?;
    if folder.contains('/') {
        return Err("le dossier de livraison doit être un nom simple".into());
    }
    let root = project.deliveries_root().join(folder);
    no_links(project, &root)?;
    Ok(root)
}

pub(super) fn destination(project: &Project, root: &Path, stored: &str) -> Result<PathBuf, String> {
    let dest = root.join(relative(stored)?);
    no_links(project, &dest)?;
    if let Ok(canonical) = dest.canonicalize() {
        let target = key(&canonical.to_string_lossy());
        for asset in &project.data.assets {
            if project.resolve(&asset.source).canonicalize().ok()
                .is_some_and(|source| key(&source.to_string_lossy()) == target) {
                return Err(format!("la destination est un fichier maître : {}", dest.display()));
            }
        }
    }
    Ok(dest)
}
