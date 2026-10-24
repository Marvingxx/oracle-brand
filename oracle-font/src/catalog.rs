//! Every font Oracle Font knows: the system's, the user's, the library's and the brands' (Brand Kit),
//! read once on a worker thread. A font file is read whole only to look at its tables (names,
//! weight, axes, glyph count, scripts); nothing is kept but what the interface shows.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Where a font was found.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// `C:\Windows\Fonts`.
    System,
    /// The user's own fonts (`%LOCALAPPDATA%\Microsoft\Windows\Fonts`): what Oracle Font activates.
    User,
    /// The Oracle Library folder.
    Library,
    /// A brand of Oracle Brand Kit (its name).
    Brand(String),
}

impl Source {
    pub fn label(&self) -> String {
        match self {
            Source::System => "Système".into(),
            Source::User => "Activées (utilisateur)".into(),
            Source::Library => "Bibliothèque".into(),
            Source::Brand(b) => format!("Marque · {b}"),
        }
    }
}

/// A variation axis of a variable font.
#[derive(Clone, Debug, PartialEq)]
pub struct Axis {
    pub tag: [u8; 4],
    pub min: f32,
    pub default: f32,
    pub max: f32,
}

impl Axis {
    pub fn name(&self) -> String {
        match &self.tag {
            b"wght" => "Graisse".into(),
            b"wdth" => "Chasse".into(),
            b"slnt" => "Inclinaison".into(),
            b"ital" => "Italique".into(),
            b"opsz" => "Taille optique".into(),
            t => String::from_utf8_lossy(t).into_owned(),
        }
    }
}

/// One face (a style of a family) in one file.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub path: PathBuf,
    /// Index in a collection (`.ttc`), 0 otherwise.
    pub index: u32,
    pub family: String,
    pub style: String,
    pub weight: u16,
    pub italic: bool,
    pub monospaced: bool,
    pub axes: Vec<Axis>,
    pub glyphs: u16,
    pub scripts: Vec<&'static str>,
    pub designer: String,
    pub license: String,
    pub version: String,
    pub sources: Vec<Source>,
}

impl Face {
    /// A stable key for this face (its file and index).
    pub fn key(&self) -> String {
        format!("{}#{}", self.path.display(), self.index)
    }
}

/// A family and its faces, the regular one first.
#[derive(Clone, Debug, PartialEq)]
pub struct Family {
    pub name: String,
    pub faces: Vec<Face>,
}

impl Family {
    /// The face to show the family with: Regular (400, upright) if there is one.
    pub fn regular(&self) -> Option<&Face> {
        self.faces.iter().min_by_key(|f| ((f.weight as i32 - 400).abs(), f.italic))
    }
    pub fn sources(&self) -> Vec<Source> {
        let mut s: Vec<Source> = self.faces.iter().flat_map(|f| f.sources.iter().cloned()).collect();
        s.sort();
        s.dedup();
        s
    }
    pub fn is_variable(&self) -> bool {
        self.faces.iter().any(|f| !f.axes.is_empty())
    }
    pub fn is_monospaced(&self) -> bool {
        self.faces.iter().any(|f| f.monospaced)
    }
}

/// Font files' extensions.
pub fn is_font(path: &Path) -> bool {
    path.extension().is_some_and(|e| matches!(e.to_string_lossy().to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc"))
}

/// The largest font file read (CJK collections are tens of megabytes).
const MAX_BYTES: u64 = 96 << 20;

/// The faces of one file (several in a collection). An unreadable file has none.
pub fn read_file(path: &Path, source: Source) -> Vec<Face> {
    let Ok(meta) = std::fs::metadata(path) else { return Vec::new() };
    if meta.len() > MAX_BYTES {
        return Vec::new();
    }
    let Ok(data) = std::fs::read(path) else { return Vec::new() };
    let count = ttf_parser::fonts_in_collection(&data).unwrap_or(1).min(64);
    (0..count).filter_map(|i| face(&data, i, path, source.clone())).collect()
}

fn name(face: &ttf_parser::Face, ids: &[u16]) -> String {
    for id in ids {
        let found = face.names().into_iter().filter(|n| n.name_id == *id).find_map(|n| n.to_string().filter(|s| !s.trim().is_empty()));
        if let Some(s) = found {
            return s.trim().to_string();
        }
    }
    String::new()
}

fn face(data: &[u8], index: u32, path: &Path, source: Source) -> Option<Face> {
    use ttf_parser::name_id;
    let f = ttf_parser::Face::parse(data, index).ok()?;
    let family = name(&f, &[name_id::TYPOGRAPHIC_FAMILY, name_id::FAMILY]);
    if family.is_empty() {
        return None;
    }
    let style = name(&f, &[name_id::TYPOGRAPHIC_SUBFAMILY, name_id::SUBFAMILY]);
    let axes = f
        .variation_axes()
        .into_iter()
        .filter(|a| !a.hidden)
        .map(|a| Axis { tag: a.tag.to_bytes(), min: a.min_value, default: a.def_value, max: a.max_value })
        .collect();
    let has = |c: char| f.glyph_index(c).is_some();
    let mut scripts = Vec::new();
    for (label, probe) in [
        ("Latin", 'A'),
        ("Latin étendu", 'Ő'),
        ("Grec", 'Ω'),
        ("Cyrillique", 'Ж'),
        ("Hébreu", 'א'),
        ("Arabe", 'ب'),
        ("Devanagari", 'क'),
        ("Thaï", 'ก'),
        ("Chinois", '中'),
        ("Japonais", 'あ'),
        ("Coréen", '한'),
    ] {
        if has(probe) {
            scripts.push(label);
        }
    }
    Some(Face {
        path: path.to_path_buf(),
        index,
        family,
        style: if style.is_empty() { "Regular".into() } else { style },
        weight: f.weight().to_number(),
        italic: f.is_italic(),
        monospaced: f.is_monospaced(),
        axes,
        glyphs: f.number_of_glyphs(),
        scripts,
        designer: name(&f, &[name_id::DESIGNER, name_id::MANUFACTURER]),
        license: name(&f, &[name_id::LICENSE, name_id::COPYRIGHT_NOTICE]),
        version: name(&f, &[name_id::VERSION]),
        sources: vec![source],
    })
}

/// The font files under `dir` (two levels deep: the library keeps fonts in folders).
fn files_in(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if depth < 6 {
                files_in(&p, depth + 1, out);
            }
        } else if is_font(&p) {
            out.push(p);
        }
    }
}

/// Where each kind of font is looked for.
pub fn system_dir() -> Option<PathBuf> {
    std::env::var_os("WINDIR").map(|w| PathBuf::from(w).join("Fonts"))
}

pub fn user_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("Microsoft").join("Windows").join("Fonts"))
}

/// Every font, grouped into families sorted by name. `library` is the library folder, if any.
pub fn scan(library: Option<&Path>) -> Vec<Family> {
    let mut found: Vec<(PathBuf, Source)> = Vec::new();
    for (dir, source) in [(system_dir(), Source::System), (user_dir(), Source::User), (library.map(Path::to_path_buf), Source::Library)] {
        let Some(dir) = dir else { continue };
        let mut files = Vec::new();
        files_in(&dir, if source == Source::Library { 0 } else { 5 }, &mut files);
        found.extend(files.into_iter().map(|f| (f, source.clone())));
    }
    for b in oracle_suite_link::brands::brands() {
        for font in &b.fonts {
            found.extend(font.files.iter().filter(|f| is_font(f)).map(|f| (f.clone(), Source::Brand(b.name.clone()))));
        }
    }
    // A file found twice (a brand's font kept in the library) is read once, with both sources.
    let mut by_file: BTreeMap<String, (PathBuf, Vec<Source>)> = BTreeMap::new();
    for (path, source) in found {
        let key = path.to_string_lossy().to_lowercase();
        by_file.entry(key).or_insert_with(|| (path.clone(), Vec::new())).1.push(source);
    }
    let mut families: BTreeMap<String, Family> = BTreeMap::new();
    for (_, (path, sources)) in by_file {
        let Some(first) = sources.first().cloned() else { continue };
        for mut f in read_file(&path, first) {
            f.sources = sources.clone();
            let key = f.family.to_lowercase();
            families.entry(key).or_insert_with(|| Family { name: f.family.clone(), faces: Vec::new() }).faces.push(f);
        }
    }
    let mut out: Vec<Family> = families.into_values().collect();
    for fam in &mut out {
        fam.faces.sort_by(|a, b| (a.italic, a.weight, &a.style).cmp(&(b.italic, b.weight, &b.style)));
        fam.faces.dedup_by(|a, b| a.style == b.style && a.weight == b.weight && a.italic == b.italic);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bundled Lexend: a variable family with a weight axis and Latin glyphs.
    #[test]
    fn a_variable_font_is_read_with_its_axes() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../style/assets/Lexend-Variable.ttf");
        let faces = read_file(&path, Source::Library);
        assert_eq!(faces.len(), 1);
        let f = &faces[0];
        assert_eq!(f.family, "Lexend");
        assert!(f.axes.iter().any(|a| &a.tag == b"wght" && a.min < a.max), "{:?}", f.axes);
        assert!(f.scripts.contains(&"Latin"));
        assert!(!f.monospaced);
    }

    #[test]
    fn what_is_not_a_font_is_left_alone() {
        assert!(!is_font(Path::new("notes.txt")));
        assert!(is_font(Path::new("A.OTF")));
        let tmp = std::env::temp_dir().join(format!("oracle-font-bad-{}.ttf", std::process::id()));
        std::fs::write(&tmp, b"not a font").unwrap();
        assert!(read_file(&tmp, Source::User).is_empty());
    }
}
