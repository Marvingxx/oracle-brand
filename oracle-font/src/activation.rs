//! Activating a font for the user, without administrator rights, as Windows does when you choose
//! "Install for me": the file is copied into the user's font folder
//! (`%LOCALAPPDATA%\Microsoft\Windows\Fonts`) and named in the user's font registry key. Apps see it
//! from their next start. Deactivating takes away only what is in the user's folder; the system's
//! fonts are never touched.

use std::path::{Path, PathBuf};

use crate::catalog::{Face, user_dir};

const FONTS_KEY: &str = r"Software\Microsoft\Windows NT\CurrentVersion\Fonts";

/// The registry value name Windows uses for a face (`Lexend Regular (TrueType)`).
pub fn value_name(face: &Face) -> String {
    let kind = if face.path.extension().is_some_and(|e| e.eq_ignore_ascii_case("otf")) { "OpenType" } else { "TrueType" };
    format!("{} {} ({kind})", face.family, face.style)
}

/// Whether `face` is one the user activated (its file is in the user's font folder).
pub fn is_user_font(face: &Face) -> bool {
    user_dir().is_some_and(|d| face.path.starts_with(d))
}

/// Copies `face`'s file into the user's font folder (keeping a file already there) and registers it.
pub fn activate(face: &Face) -> Result<PathBuf, String> {
    let dir = user_dir().ok_or("dossier des polices de l'utilisateur introuvable")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{} : {e}", dir.display()))?;
    let name = face.path.file_name().ok_or("fichier de police sans nom")?;
    let target = dir.join(name);
    if !target.exists() {
        std::fs::copy(&face.path, &target).map_err(|e| format!("{} : {e}", face.path.display()))?;
    }
    register(&value_name(face), &target)?;
    Ok(target)
}

/// Takes `face` off the user's fonts: its registry value and, when it is in the user's folder, its
/// file (a file in use by a running app stays until that app quits).
pub fn deactivate(face: &Face) -> Result<(), String> {
    if !is_user_font(face) {
        return Err("seules les polices activées pour l'utilisateur se désactivent ici".into());
    }
    unregister(&face.path)?;
    std::fs::remove_file(&face.path).map_err(|e| format!("{} : {e} (fermez les logiciels qui l'utilisent)", face.path.display()))
}

#[cfg(windows)]
fn register(name: &str, file: &Path) -> Result<(), String> {
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).create_subkey(FONTS_KEY).map_err(|e| e.to_string())?.0;
    key.set_value(name, &file.to_string_lossy().to_string()).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn unregister(file: &Path) -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    let key = winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(FONTS_KEY, KEY_READ | KEY_WRITE).map_err(|e| e.to_string())?;
    let target = file.to_string_lossy().to_lowercase();
    let names: Vec<String> = key
        .enum_values()
        .flatten()
        .filter(|(_, v)| v.to_string().trim_matches('"').to_lowercase() == target)
        .map(|(n, _)| n)
        .collect();
    for n in names {
        key.delete_value(&n).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn register(_: &str, _: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
fn unregister(_: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_names_follow_windows() {
        let mut f = crate::catalog::read_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../style/assets/Lexend-Variable.ttf"), crate::catalog::Source::Library).remove(0);
        assert_eq!(value_name(&f), format!("Lexend {} (TrueType)", f.style));
        f.path = PathBuf::from("C:/x/a.otf");
        assert!(value_name(&f).ends_with("(OpenType)"));
        assert!(!is_user_font(&f));
        assert!(deactivate(&f).is_err(), "a font outside the user's folder is never removed");
    }
}
