use oracle_brand::guide::{Chapter, ContrastPair, Rights};
use oracle_brand::model::{Asset, BrandColor, BrandFont, ForbiddenUse, Role, Variant};
use oracle_brand::project::{FILE_NAME, hash_file};
use oracle_brand::render::Engines;
use oracle_brand::{Project, guide};

const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#D31C3B"/></svg>"##;

fn fixture() -> (tempfile::TempDir, Project) {
    let tmp = tempfile::tempdir().unwrap();
    let mut p = Project::create(tmp.path(), "DEMONSTRATION FICTIVE").unwrap();
    std::fs::write(p.dir.join("logo.svg"), SVG).unwrap();
    std::fs::write(p.dir.join("private.ttf"), b"not-for-delivery").unwrap();
    p.data.assets.push(Asset {
        id: "logo".into(),
        role: Role::PrimaryLogo,
        label: "Logo de test".into(),
        variant: Variant::Color,
        source: "logo.svg".into(),
        source_hash: String::new(),
    });
    for (id, hex) in [("ink", "#1B1A17"), ("paper", "#FFFFFF")] {
        p.data.colors.push(BrandColor {
            id: id.into(),
            name: id.into(),
            hex: hex.into(),
            ..Default::default()
        });
    }
    p.data.fonts.push(BrandFont {
        id: "font".into(),
        family: "Police fictive".into(),
        usage: "Texte".into(),
        weights: vec!["Regular".into()],
        license_url: "https://example.org/license".into(),
        local_files: vec!["private.ttf".into()],
        redistributable: true,
    });
    p.data.rules.clear_space = "1 unité".into();
    p.data.rules.min_size = "32 px".into();
    p.data.rules.allowed_backgrounds = "Blanc".into();
    p.data.rules.forbidden.push(ForbiddenUse {
        text: "Ne pas déformer".into(),
        example: String::new(),
    });
    p.data.guide.audience = "Test automatique, écran uniquement".into();
    p.data.guide.owner = "Projet fictif".into();
    p.data.guide.reviewer = "Simulation automatique — aucun visa client".into();
    p.data.guide.rights.insert(
        "logo".into(),
        Rights {
            allow_delivery: true,
            evidence: "SVG créé pour cet essai".into(),
            reviewer: "Test".into(),
            reviewed_at: oracle_brand::now_rfc3339(),
            source_hash: hash_file(&p.dir.join("logo.svg")).unwrap(),
        },
    );
    p.data.guide.contrast_pairs.push(ContrastPair {
        foreground_id: "ink".into(),
        background_id: "paper".into(),
        context: "Texte courant sur fond blanc".into(),
        large_text_or_graphic: false,
    });
    for spec in guide::CHAPTERS.iter().filter(|s| !s.optional) {
        let mut c = Chapter::default();
        for field in spec.fields {
            c.fields.insert(
                (*field).into(),
                format!("Décision fictive de test : {field}."),
            );
        }
        c.asset_ids.push("logo".into());
        p.data.guide.chapters.insert(spec.id.into(), c);
    }
    approve_all(&mut p);
    (tmp, p)
}

fn approve_all(p: &mut Project) {
    let ids: Vec<_> = guide::applicable(&p.data.guide).map(|s| s.id).collect();
    for id in ids {
        guide::approve(&mut p.data, id, false).unwrap();
    }
}

#[test]
fn empty_entries_do_not_count_as_typography_or_usage_decisions() {
    let (_tmp, mut p) = fixture();
    p.data.fonts[0].weights = vec!["   ".into()];
    p.data.rules.forbidden[0].text = "  ".into();
    approve_all(&mut p);
    let report = guide::check(&p);
    assert!(report.blockers.iter().any(|b| b.code == "fonts"));
    assert!(report.blockers.iter().any(|b| b.code == "rules"));
}

#[test]
fn legacy_migration_preserves_extensions_without_fabricating_approvals() {
    let tmp = tempfile::tempdir().unwrap();
    let p = Project::create(tmp.path(), "Ancienne marque").unwrap();
    let mut value = serde_json::to_value(&p.data).unwrap();
    value["schemaVersion"] = 1.into();
    value["foreignTool"] = serde_json::json!({"keep": [1,2,3]});
    value.as_object_mut().unwrap().remove("guide");
    std::fs::write(p.dir.join(FILE_NAME), serde_json::to_vec(&value).unwrap()).unwrap();
    let migrated = Project::open(tmp.path()).unwrap();
    assert_eq!(
        migrated.data.schema_version,
        oracle_brand::model::SCHEMA_VERSION
    );
    assert!(migrated.data.guide.chapters.is_empty());
    assert_eq!(guide::check(&migrated).approved_chapters, 0);
    migrated.save().unwrap();
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(p.dir.join(FILE_NAME)).unwrap()).unwrap();
    assert_eq!(saved["foreignTool"], value["foreignTool"]);
    for version in [
        serde_json::json!(u64::MAX),
        serde_json::json!(-1),
        serde_json::json!("2"),
    ] {
        value["schemaVersion"] = version;
        let bytes = serde_json::to_vec(&value).unwrap();
        std::fs::write(p.dir.join(FILE_NAME), &bytes).unwrap();
        assert!(Project::open(tmp.path()).is_err());
        assert_eq!(std::fs::read(p.dir.join(FILE_NAME)).unwrap(), bytes);
    }
}

#[test]
fn incomplete_final_stops_but_draft_is_explicit_and_excludes_unlicensed_files() {
    let (_tmp, mut p) = fixture();
    p.data.guide.rights.clear();
    assert!(guide::export(&p, &Engines::discover(), false).is_err());
    assert!(!p.dir.join("Livrables").exists());
    let out = guide::export(&p, &Engines::discover(), true).unwrap();
    let html = std::fs::read_to_string(out.join("guide.html")).unwrap();
    assert!(html.contains("BROUILLON"));
    let m: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(m["completeForScope"], false);
    assert_eq!(m["excluded"][0], "logo");
    assert!(m["resourcesById"].as_object().unwrap().is_empty());
    assert_eq!(std::fs::read_dir(out.join("assets")).unwrap().count(), 0);
}

#[test]
fn final_is_offline_hashed_immutable_and_never_bundles_fonts() {
    let (_tmp, p) = fixture();
    assert!(guide::check(&p).complete(), "{:?}", guide::check(&p));
    let before = hash_file(&p.dir.join("logo.svg")).unwrap();
    let a = guide::export(&p, &Engines::discover(), false).unwrap();
    let b = guide::export(&p, &Engines::discover(), false).unwrap();
    assert_ne!(a, b);
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(a.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["completeForScope"], true);
    assert_eq!(manifest["fontFilesIncluded"], false);
    for (path, hash) in manifest["sha256"].as_object().unwrap() {
        assert_eq!(
            &serde_json::Value::String(hash_file(&a.join(path)).unwrap()),
            hash
        );
        assert!(!path.ends_with(".ttf"));
    }
    assert_eq!(hash_file(&p.dir.join("logo.svg")).unwrap(), before);
    assert!(a.join("assets/001-master.svg").is_file());
    assert!(a.join("assets/001-preview.png").is_file());
    assert!(a.join("fiche-reference.html").is_file());
    let html = std::fs::read_to_string(a.join("guide.html")).unwrap();
    assert!(!html.contains("<script"));
    assert!(!html.contains("src='http"));
    assert_eq!(html.matches("<section id='BG-").count(), 12);
}

#[test]
fn shared_edits_and_deleted_links_require_review_again() {
    let (_tmp, mut p) = fixture();
    p.data.colors[0].name = "Nouvelle encre".into();
    assert_eq!(guide::check(&p).approved_chapters, 0);
    approve_all(&mut p);
    assert!(guide::check(&p).complete());
    p.data
        .guide
        .chapters
        .get_mut("BG-03")
        .unwrap()
        .asset_ids
        .push("deleted".into());
    approve_all(&mut p);
    assert!(
        guide::check(&p)
            .blockers
            .iter()
            .any(|i| i.message.contains("supprimée"))
    );
}

#[test]
fn replaced_source_revokes_permission_and_is_excluded_from_drafts() {
    let (_tmp, p) = fixture();
    std::fs::write(p.dir.join("logo.svg"), SVG.replace("#D31C3B", "#000000")).unwrap();
    assert!(!guide::authorized(&p, "logo"));
    assert!(guide::check(&p).blockers.iter().any(|i| i.code == "rights"));
    let out = guide::export(&p, &Engines::discover(), true).unwrap();
    assert!(!out.join("assets/001-master.svg").exists());
}

#[test]
fn thresholds_are_unrounded_and_print_never_invents_a_conversion() {
    let (_tmp, mut p) = fixture();
    let ratio = guide::contrast("#777777", "#FFFFFF").unwrap();
    assert!(ratio > 4.47 && ratio < 4.5);
    p.data.colors[0].hex = "#777777".into();
    approve_all(&mut p);
    assert!(
        guide::check(&p)
            .blockers
            .iter()
            .any(|i| i.code == "contrast")
    );
    p.data.guide.contrast_pairs[0].large_text_or_graphic = true;
    approve_all(&mut p);
    assert!(guide::check(&p).complete());
    p.data.guide.print_required = true;
    approve_all(&mut p);
    assert!(guide::check(&p).blockers.iter().any(|i| i.code == "print"));
    assert!(p.data.colors.iter().all(|c| c.cmyk.is_none()));
}

#[test]
fn modules_require_justification_and_unknown_modules_block_delivery() {
    let (_tmp, mut p) = fixture();
    assert!(guide::approve(&mut p.data, "BG-01", true).is_err());
    p.data.guide.modules.push("audio".into());
    p.data
        .guide
        .chapters
        .insert("audio".into(), Chapter::default());
    assert!(guide::approve(&mut p.data, "audio", true).is_err());
    p.data.guide.chapters.get_mut("audio").unwrap().reason =
        "Signature sonore exclue de ce mandat.".into();
    let ids: Vec<_> = guide::applicable(&p.data.guide).map(|s| s.id).collect();
    for id in ids {
        guide::approve(&mut p.data, id, id == "audio").unwrap();
    }
    assert!(guide::check(&p).complete());
    p.data.guide.modules.push("future-unknown".into());
    assert!(guide::check(&p).blockers.iter().any(|i| i.code == "module"));
}

#[test]
fn malformed_preview_rolls_back_and_untrusted_text_is_escaped() {
    let (_tmp, mut p) = fixture();
    p.data.brand_name = "<script>alert('x')</script>".into();
    p.data.fonts[0].license_url = "javascript:alert(1)".into();
    let out = guide::export(&p, &Engines::discover(), true).unwrap();
    let html = std::fs::read_to_string(out.join("guide.html")).unwrap();
    assert!(!html.contains("<script>"));
    assert!(!html.contains("href='javascript:"));
    assert!(html.contains("&lt;script&gt;"));
    p.data.fonts[0].license_url = "https://example.org".into();
    std::fs::write(p.dir.join("logo.svg"), "not svg").unwrap();
    p.data.guide.rights.get_mut("logo").unwrap().source_hash =
        hash_file(&p.dir.join("logo.svg")).unwrap();
    approve_all(&mut p);
    assert!(guide::export(&p, &Engines::discover(), false).is_err());
    let children: Vec<_> = std::fs::read_dir(p.dir.join("Livrables/Brand-guides"))
        .unwrap()
        .map(|v| v.unwrap().file_name())
        .collect();
    assert_eq!(
        children.len(),
        1,
        "failed export leaked preparation directory"
    );
}
