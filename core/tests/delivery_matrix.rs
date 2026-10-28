//! The delivery matrix (as Logo Exports): colour treatments computed from the colour master,
//! filed by logo, treatment and format, with the client's file guide. Raster and SVG only, so no
//! external program is needed.
#![allow(clippy::unwrap_used)]
use std::sync::atomic::AtomicBool;

use oracle_brand::export;
use oracle_brand::model::{Asset, Format, Role, Variant};
use oracle_brand::presets::delivery_matrix;
use oracle_brand::project::Project;
use oracle_brand::recolor::Treatment;
use oracle_brand::render::Engines;

const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 100"><rect width="100" height="100" fill="#D31C3B"/><rect x="110" y="30" width="90" height="40" fill="#1B1A17"/></svg>"##;

fn project() -> (tempfile::TempDir, Project) {
    let tmp = tempfile::tempdir().unwrap();
    let mut p = Project::create(&tmp.path().join("project"), "Orbe").unwrap();
    p.data.version = "v1".into();
    std::fs::write(p.dir.join("logo.svg"), LOGO).unwrap();
    p.data.assets.push(Asset {
        id: "logo".into(),
        role: Role::PrimaryLogo,
        label: "Logo principal".into(),
        variant: Variant::Color,
        source: "logo.svg".into(),
        source_hash: String::new(),
    });
    // SVG, PNG and JPEG presets of the matrix: what runs without Oracle Vector.
    p.data.export_presets = delivery_matrix().into_iter().filter(|x| matches!(x.format, Format::Svg | Format::Png | Format::Jpeg)).collect();
    (tmp, p)
}

#[test]
fn every_treatment_is_filed_by_logo_treatment_and_format() {
    let (_tmp, mut p) = project();
    let plan = export::plan(&p, &[], &[]);
    // SVG 5 + PNG 2 × 5 + JPEG 3 (no white or inverted on a white background).
    assert_eq!(plan.outputs.len(), 18, "{:#?}", plan.outputs.iter().map(|o| &o.path).collect::<Vec<_>>());
    assert!(plan.outputs.iter().all(|o| o.blocked.is_none()), "{:#?}", plan.outputs.iter().filter_map(|o| o.blocked.as_ref().map(|b| (&o.path, b))).collect::<Vec<_>>());
    assert!(plan.outputs.iter().any(|o| o.path == "Logos/logo-principal/noir/SVG/Orbe_logo-principal_noir.svg"), "{:#?}", plan.outputs.iter().map(|o| &o.path).collect::<Vec<_>>());
    let report = export::run(&mut p, &Engines::discover(), &plan, None, &AtomicBool::new(false), |_, _, _| {});
    assert_eq!(report.succeeded(), 18, "{:?}", report.results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());
    let root = p.deliveries_root().join(&plan.folder);
    // The black SVG has no colour left but black (and the root's explicit fill).
    let black = std::fs::read_to_string(root.join("Logos/logo-principal/noir/SVG/Orbe_logo-principal_noir.svg")).unwrap();
    assert!(!black.to_lowercase().contains("#d31c3b") && black.contains("#000000"));
    // The white PNG: every opaque pixel white.
    let white = image::open(root.join("Logos/logo-principal/blanc/PNG/Orbe_logo-principal_blanc_1000px.png")).unwrap().to_rgba8();
    assert!(white.pixels().filter(|p| p.0[3] == 255).all(|p| p.0[..3] == [255, 255, 255]));
    assert!(white.pixels().any(|p| p.0[3] == 255));
    // The colour master is delivered byte for byte.
    assert_eq!(std::fs::read_to_string(root.join("Logos/logo-principal/couleur/SVG/Orbe_logo-principal_couleur.svg")).unwrap(), LOGO);
    // The file guide names each version's use and lists every file.
    let guide = std::fs::read_to_string(root.join(export::FILE_GUIDE)).unwrap();
    for t in Treatment::ALL {
        assert!(guide.contains(t.label_fr()), "{t:?}");
    }
    assert!(guide.contains("Orbe_logo-principal_niveaux-de-gris_3000px.png"));
}

#[test]
fn a_version_the_designer_drew_replaces_the_computed_one() {
    let (_tmp, mut p) = project();
    std::fs::write(p.dir.join("logo-noir.svg"), LOGO.replace("#D31C3B", "#000000").replace("#1B1A17", "#000000")).unwrap();
    p.data.assets.push(Asset {
        id: "logo-noir".into(),
        role: Role::PrimaryLogo,
        label: "Logo noir".into(),
        variant: Variant::Black,
        source: "logo-noir.svg".into(),
        source_hash: String::new(),
    });
    let plan = export::plan(&p, &["logo".into()], &["matrix-svg".into()]);
    let treatments: Vec<Treatment> = plan.outputs.iter().map(|o| o.treatment).collect();
    assert!(!treatments.contains(&Treatment::Black), "the drawn black version wins: {treatments:?}");
    assert_eq!(treatments.len(), 4);
}
