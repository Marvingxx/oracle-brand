use oracle_brand::model::{Asset, Role, Variant};
use oracle_brand::project::hash_file;
use oracle_brand::render::Engines;
use oracle_brand::{
    Project,
    guide::{self, Rights},
    studio::{self, Concept, Tool},
};

fn fixture() -> (tempfile::TempDir, Project) {
    fixture_in(tempfile::tempdir().unwrap())
}

fn fixture_in(tmp: tempfile::TempDir) -> (tempfile::TempDir, Project) {
    let mut p = Project::create(tmp.path(), "MARQUE FICTIVE <&>").unwrap();
    std::fs::write(p.dir.join("logo.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><rect width="200" height="100" fill="#D31C3B"/><circle cx="100" cy="50" r="30" fill="white"/></svg>"##).unwrap();
    let hash = hash_file(&p.dir.join("logo.svg")).unwrap();
    p.data.assets.push(Asset {
        id: "logo".into(),
        role: Role::PrimaryLogo,
        variant: Variant::Color,
        label: "Logo".into(),
        source: "logo.svg".into(),
        source_hash: hash.clone(),
    });
    p.data.studio.asset_id = "logo".into();
    let mut c = Concept::new("logo".into());
    c.title = "Piste <script>".into();
    c.rationale = "COMMENTAIRE INTERNE & justification".into();
    p.data.studio.concepts.push(c);
    p.data.guide.rights.insert(
        "logo".into(),
        Rights {
            allow_delivery: true,
            evidence: "Création géométrique pour essai".into(),
            reviewer: "Test automatique".into(),
            reviewed_at: oracle_brand::now_rfc3339(),
            source_hash: hash,
        },
    );
    (tmp, p)
}

#[test]
fn exports_accept_a_project_opened_from_a_relative_path() {
    let cwd = std::env::current_dir().unwrap().canonicalize().unwrap();
    let tmp = tempfile::Builder::new()
        .prefix("studio-relative-")
        .tempdir_in(&cwd)
        .unwrap();
    let (_tmp, mut p) = fixture_in(tmp);
    p.dir = p.dir.strip_prefix(&cwd).unwrap().to_path_buf();
    assert!(!p.dir.is_absolute());
    let folder = studio::export(&p, &Engines::discover(), Tool::Grid).unwrap();
    assert!(folder.join("planche-01.png").is_file());
    verify_manifest(&folder);
}

#[test]
fn raster_sources_decode_into_embedded_previews_in_all_advertised_formats() {
    let (_tmp, mut p) = fixture();
    let e = Engines::default();
    let pixels = image::RgbImage::from_fn(64, 32, |x, _| {
        if x < 32 {
            image::Rgb([211, 28, 59])
        } else {
            image::Rgb([27, 26, 23])
        }
    });
    for ext in ["PNG", "jpg", "jpeg", "webp", "tif", "tiff", "bmp", "gif"] {
        let filename = format!("raster.{ext}");
        let source = p.dir.join(&filename);
        pixels.save(&source).unwrap();
        let original = hash_file(&source).unwrap();
        p.data.assets[0].source = filename;
        p.data.assets[0].source_hash = original.clone();
        p.data.guide.rights.get_mut("logo").unwrap().source_hash = original.clone();
        let folder = studio::export(&p, &e, Tool::Bento).unwrap();
        verify_manifest(&folder);
        let board = image::open(folder.join("planche-01.png"))
            .unwrap()
            .to_rgb8();
        let red = board.get_pixel(200, 370);
        assert!(
            red[0] > 180 && red[1] < 65 && red[2] < 90,
            "{ext}: logo absent"
        );
        assert_eq!(hash_file(&source).unwrap(), original);
    }
}

fn verify_manifest(folder: &std::path::Path) -> serde_json::Value {
    let v: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    for (file, expected) in v["sha256"].as_object().unwrap() {
        assert_eq!(
            hash_file(&folder.join(file)).unwrap(),
            expected.as_str().unwrap()
        );
    }
    v
}

#[test]
fn three_tools_export_real_boards_without_editing_sources_or_leaking_other_concepts() {
    let (_tmp, p) = fixture();
    let engines = Engines::discover();
    let original = hash_file(&p.dir.join("logo.svg")).unwrap();
    for tool in [Tool::Grid, Tool::Bento, Tool::Presentations] {
        let folder = studio::export(&p, &engines, tool).unwrap();
        let manifest = verify_manifest(&folder);
        assert_eq!(manifest["clientApproval"], false);
        assert_eq!(manifest["fontFilesIncluded"], false);
        assert_eq!(manifest["sourceSha256ById"]["logo"], original);
        assert_eq!(
            image::open(folder.join("planche-01.png"))
                .unwrap()
                .to_rgba8()
                .dimensions(),
            (1200, 800)
        );
        let svg = std::fs::read_to_string(folder.join("planche-01.svg")).unwrap();
        assert!(svg.contains("data:image/png;base64,"));
        assert!(!svg.contains("<script>"));
        assert!(svg.contains("MARQUE FICTIVE &lt;&amp;&gt;"));
        let recipe = std::fs::read_to_string(folder.join("recette.json")).unwrap();
        if tool != Tool::Presentations {
            assert!(!recipe.contains("COMMENTAIRE INTERNE"));
        }
        for file in std::fs::read_dir(&folder).unwrap() {
            assert!(
                !file
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".source")
            );
        }
        assert_eq!(hash_file(&p.dir.join("logo.svg")).unwrap(), original);
        let second = studio::export(&p, &engines, tool).unwrap();
        assert_ne!(second, folder);
        verify_manifest(&folder);
    }
}

#[test]
fn delivery_requires_current_rights_and_failed_exports_leave_no_partial_revision() {
    let (_tmp, mut p) = fixture();
    let e = Engines::discover();
    p.data.guide.rights.clear();
    assert!(
        studio::export(&p, &e, Tool::Grid)
            .unwrap_err()
            .contains("autorisation")
    );
    assert_eq!(
        std::fs::read_dir(p.dir.join("Livrables/Studio"))
            .unwrap()
            .count(),
        0
    );
    let (_tmp2, p2) = fixture();
    std::fs::write(p2.dir.join("logo.svg"), "modified").unwrap();
    assert!(studio::export(&p2, &e, Tool::Bento).is_err());
    assert_eq!(
        std::fs::read_dir(p2.dir.join("Livrables/Studio"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn grid_boundaries_are_safe_and_other_tools_ignore_irrelevant_grid_settings() {
    let (_tmp, mut p) = fixture();
    let e = Engines::discover();
    for unit in [10, 80] {
        for protection in [0, 2] {
            p.data.studio.grid_unit = unit;
            p.data.studio.protection_units = protection;
            let svg = studio::document(&p, &e, Tool::Grid, 0).unwrap();
            studio::raster(&svg).unwrap();
        }
    }
    p.data.studio.grid_unit = 0;
    assert!(studio::export(&p, &e, Tool::Grid).is_err());
    assert!(studio::document(&p, &e, Tool::Bento, 0).is_ok());
    p.data.studio.concepts.clear();
    assert!(studio::export(&p, &e, Tool::Presentations).is_err());
    p.data.studio.concepts = vec![Concept::new("missing".into())];
    assert!(studio::export(&p, &e, Tool::Presentations).is_err());
}

#[test]
fn studio_recipes_are_additive_persistent_and_do_not_invalidate_editorial_approval() {
    let (_tmp, mut p) = fixture();
    let before = guide::fingerprint(&p.data);
    p.data.studio.headline = "Une accroche".into();
    p.data
        .studio
        .extensions
        .insert("futureLayout".into(), serde_json::json!({"columns":5}));
    p.save().unwrap();
    let opened = Project::open(&p.dir).unwrap();
    assert_eq!(opened.data.studio, p.data.studio);
    assert_eq!(guide::fingerprint(&p.data), before);
    let mut legacy = serde_json::to_value(&p.data).unwrap();
    legacy.as_object_mut().unwrap().remove("studio");
    let parsed: oracle_brand::model::BrandProject = serde_json::from_value(legacy).unwrap();
    assert!(parsed.studio.concepts.is_empty());
    assert_eq!(parsed.studio.grid_unit, 40);
}

#[test]
fn hub_is_a_hashed_offline_revision_and_final_requires_guide_approval() {
    let (_tmp, mut p) = fixture();
    let e = Engines::discover();
    assert!(guide::export_hub(&p, &e, false).is_err());
    let folder = guide::export_hub(&p, &e, true).unwrap();
    let m = verify_manifest(&folder);
    assert_eq!(m["draft"], true);
    assert_eq!(m["offlineClientSpace"], true);
    assert_eq!(m["completeForScope"], false);
    let index = std::fs::read_to_string(folder.join("index.html")).unwrap();
    assert!(index.contains("BROUILLON"));
    assert!(index.contains("href='guide.html'"));
    assert!(index.contains("Télécharger le maître"));
    assert!(index.contains("MARQUE FICTIVE &lt;&amp;&gt;"));
    p.data.guide.rights.clear();
    let excluded = guide::export_hub(&p, &e, true).unwrap();
    assert!(
        verify_manifest(&excluded)["resourcesById"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    assert!(
        !std::fs::read_to_string(excluded.join("index.html"))
            .unwrap()
            .contains("Télécharger le maître")
    );
}
