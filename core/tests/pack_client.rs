//! End-to-end: a client pack from two masters, then a revision of the main logo.
//!
//! Uses the real Oracle Vector engine (`oracle-vector-cli`); without it the test says so and only
//! checks what doesn't need it.

use std::sync::atomic::AtomicBool;

use oracle_brand::export::{self, plan};
use oracle_brand::model::{Asset, BrandColor, BrandFont, Role, Variant};
use oracle_brand::project::Project;
use oracle_brand::render::Engines;
use oracle_brand::SourceState;

const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="160" viewBox="0 0 400 160">
  <rect x="10" y="20" width="120" height="120" rx="24" fill="#C82029"/>
  <circle cx="70" cy="80" r="32" fill="#FFFFFF"/>
  <rect x="150" y="50" width="230" height="22" fill="#1D1D1B"/>
</svg>"##;
const SYMBOL: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="160" height="160" viewBox="0 0 160 160">
  <rect x="20" y="20" width="120" height="120" rx="24" fill="#C82029"/>
</svg>"##;

fn setup(dir: &std::path::Path) -> Project {
    // Unicode, accents and spaces in every path.
    let mut p = Project::create(dir, "Marque Été").unwrap();
    std::fs::create_dir_all(dir.join("Sources vectorielles")).unwrap();
    std::fs::write(dir.join("Sources vectorielles").join("logo principal é.svg"), LOGO).unwrap();
    std::fs::write(dir.join("Sources vectorielles").join("symbole.svg"), SYMBOL).unwrap();
    let logo = p.store_path(&dir.join("Sources vectorielles").join("logo principal é.svg"));
    let symbol = p.store_path(&dir.join("Sources vectorielles").join("symbole.svg"));
    let id = p.data.new_id("asset");
    p.data.assets.push(Asset { id, role: Role::PrimaryLogo, label: "Logo principal".into(), variant: Variant::Color, source: logo, source_hash: String::new() });
    let id = p.data.new_id("asset");
    p.data.assets.push(Asset { id, role: Role::Symbol, label: "Symbole".into(), variant: Variant::Color, source: symbol, source_hash: String::new() });
    for (name, hex) in [("Rouge principal", "#C82029"), ("Noir", "#1D1D1B"), ("Bleu", "#2E86AB")] {
        let id = p.data.new_id("color");
        p.data.colors.push(BrandColor { id, name: name.into(), hex: hex.into(), ..Default::default() });
    }
    for (family, usage) in [("Lexend", "Titres"), ("Inter", "Texte courant")] {
        let id = p.data.new_id("font");
        p.data.fonts.push(BrandFont { id, family: family.into(), usage: usage.into(), weights: vec!["Regular".into(), "SemiBold".into()], ..Default::default() });
    }
    p.save().unwrap();
    p
}

#[test]
fn client_pack_then_revision() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("Projets clients").join("Marque Été");
    let mut p = setup(&dir);
    let engines = Engines::discover();
    let Some(_) = engines.vector_cli.as_ref() else {
        eprintln!("oracle-vector-cli introuvable : test d'export ignoré");
        return;
    };

    // Plan first: nothing is written yet.
    let pl = plan(&p, &[], &[]);
    assert_eq!(pl.folder, "v1");
    assert!(!p.deliveries_root().exists(), "planning writes nothing");
    assert!(pl.outputs.iter().all(|o| o.blocked.is_none()), "{:?}", pl.outputs.iter().filter(|o| o.blocked.is_some()).collect::<Vec<_>>());
    // Favicon preset applies to the Favicon role only: no master has it.
    assert!(!pl.outputs.iter().any(|o| o.preset_id == "preset-favicon"));

    let report = export::run(&mut p, &engines, &pl, None, &AtomicBool::new(false), |_, _, _| {});
    let failures: Vec<_> = report.failures().collect();
    assert!(failures.is_empty(), "{failures:?}");
    p.save().unwrap();
    let root = p.deliveries_root().join("v1");
    assert!(root.ends_with("Livrables/Marque Été/v1") || root.ends_with("Livrables\\Marque Été\\v1"));

    // Each format: vector stays vector, PNG/WebP keep transparency, JPEG has none.
    let svg = std::fs::read_to_string(root.join("Logos/SVG/Marque-Été_logo-principal_couleur.svg")).unwrap();
    assert_eq!(svg, LOGO, "an SVG master is delivered byte for byte");
    let pdf = std::fs::read(root.join("Logos/PDF/Marque-Été_logo-principal_couleur.pdf")).unwrap();
    assert!(pdf.starts_with(b"%PDF") && !pdf.windows(13).any(|w| w == b"/Subtype/Image"), "PDF is vector");
    let png1 = image::open(root.join("Logos/PNG/Marque-Été_logo-principal_couleur_1x.png")).unwrap().to_rgba8();
    assert_eq!(png1.dimensions(), (400, 160));
    assert_eq!(png1.get_pixel(0, 0)[3], 0, "PNG keeps transparency");
    let png2 = image::open(root.join("Logos/PNG/Marque-Été_logo-principal_couleur_2x.png")).unwrap().to_rgba8();
    assert_eq!(png2.dimensions(), (800, 320));
    let webp = image::open(root.join("Logos/WebP/Marque-Été_logo-principal_couleur_2x.webp")).unwrap();
    assert!(webp.color().has_alpha());
    let jpg = image::open(root.join("Logos/JPEG/Marque-Été_logo-principal_couleur_fond-blanc.jpg")).unwrap();
    assert!(!jpg.color().has_alpha());
    assert_eq!(jpg.width(), 2000);
    assert_eq!(jpg.to_rgb8().get_pixel(0, 0).0, [255, 255, 255], "JPEG flattened on its white background");
    // The avatar: exact size, centred, margins transparent.
    let avatar = image::open(root.join("Web-et-Reseaux/Marque-Été_symbole_couleur_avatar-1080.png")).unwrap().to_rgba8();
    assert_eq!(avatar.dimensions(), (1080, 1080));
    assert_eq!(avatar.get_pixel(5, 540)[3], 0);
    assert!(avatar.get_pixel(540, 540)[3] > 200);
    // Print: 300 dpi written in the file.
    let print = std::fs::read(root.join("Impression/Marque-Été_logo-principal_couleur_impression-300dpi.png")).unwrap();
    assert!(print.windows(4).any(|w| w == b"pHYs"));

    // The manifest lists every output with its source and settings; the README is there.
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let outputs = manifest["outputs"].as_array().unwrap();
    assert_eq!(outputs.len(), pl.outputs.len());
    assert!(outputs.iter().all(|o| o["sha256"].as_str().is_some_and(|h| h.len() == 64)));
    assert_eq!(manifest["sources"].as_array().unwrap().len(), 2);
    let readme = std::fs::read_to_string(root.join("README.txt")).unwrap();
    assert!(readme.contains("Rouge principal") && readme.contains("Lexend"));

    // Sources untouched, and recorded as exported.
    assert!(p.data.assets.iter().all(|a| p.source_state(a) == SourceState::Unchanged));

    // The designer sends v1 to the client.
    p.data.deliveries[0].delivered = true;
    let v1_svg_hash = oracle_brand::project::hash_file(&root.join("Logos/SVG/Marque-Été_logo-principal_couleur.svg")).unwrap();

    // Revision: the main logo changes. Oracle detects it…
    std::fs::write(dir.join("Sources vectorielles").join("logo principal é.svg"), LOGO.replace("#1D1D1B", "#2E86AB")).unwrap();
    let stale = export::stale(&p);
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].1, SourceState::Modified);
    // …and a regeneration of that logo goes to a new folder, leaving v1 as delivered.
    let changed = vec![stale[0].0.clone()];
    let pl2 = plan(&p, &changed, &[]);
    assert_eq!(pl2.folder, "v1-2");
    let r2 = export::run(&mut p, &engines, &pl2, None, &AtomicBool::new(false), |_, _, _| {});
    assert_eq!(r2.failures().count(), 0);
    assert_eq!(oracle_brand::project::hash_file(&root.join("Logos/SVG/Marque-Été_logo-principal_couleur.svg")).unwrap(), v1_svg_hash, "v1 untouched");
    let v2 = std::fs::read_to_string(p.deliveries_root().join("v1-2/Logos/SVG/Marque-Été_logo-principal_couleur.svg")).unwrap();
    assert!(v2.contains("#2E86AB"));
    // v1-2 is a complete set: the symbol's files, unchanged, come from v1 byte for byte.
    let v12 = p.deliveries_root().join("v1-2");
    let v12_manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(v12.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(v12_manifest["outputs"].as_array().unwrap().len(), outputs.len(), "as many files as v1");
    let symbol = p.data.assets.iter().find(|a| a.role == oracle_brand::Role::Symbol).unwrap().id.clone();
    let copied: Vec<&serde_json::Value> = v12_manifest["outputs"].as_array().unwrap().iter().filter(|o| o["assetId"] == symbol.as_str()).collect();
    assert!(!copied.is_empty());
    for o in copied {
        let path = o["path"].as_str().unwrap();
        assert_eq!(oracle_brand::project::hash_file(&v12.join(path)).unwrap(), oracle_brand::project::hash_file(&root.join(path)).unwrap(), "{path}");
    }
    assert!(p.data.assets.iter().all(|a| p.source_state(a) == SourceState::Unchanged));
}

#[test]
fn broken_inputs_give_clear_errors_and_keep_previous_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("p");
    let mut p = setup(&dir);
    let engines = Engines::discover();
    // A JPEG preset without background is refused at planning.
    let jpeg = p.data.export_presets.iter_mut().find(|x| x.id == "preset-jpeg").unwrap();
    jpeg.background = None;
    let pl = plan(&p, &[], &["preset-jpeg".into()]);
    assert!(pl.outputs.iter().all(|o| o.blocked.as_deref().is_some_and(|b| b.contains("fond"))));
    // An invalid SVG master: an error for its files, the others still made.
    std::fs::write(dir.join("Sources vectorielles").join("symbole.svg"), "<svg pas fini").unwrap();
    let pl = plan(&p, &[], &["preset-web-1x".into()]);
    let r = export::run(&mut p, &engines, &pl, None, &AtomicBool::new(false), |_, _, _| {});
    let failed: Vec<_> = r.failures().map(|f| f.path.clone()).collect();
    assert_eq!(failed.len(), 1, "{:?}", r.results);
    assert!(failed[0].contains("symbole"));
    assert_eq!(r.succeeded(), 1);
    // A missing master is blocked before rendering.
    std::fs::remove_file(dir.join("Sources vectorielles").join("symbole.svg")).unwrap();
    let pl = plan(&p, &[], &["preset-web-1x".into()]);
    assert!(pl.outputs.iter().any(|o| o.blocked.as_deref().is_some_and(|b| b.contains("introuvable"))));
    // Cancelling before the first file writes nothing new.
    let cancel = AtomicBool::new(true);
    let r = export::run(&mut p, &engines, &pl, None, &cancel, |_, _, _| {});
    assert!(r.cancelled);
}
