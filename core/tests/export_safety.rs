//! Regression tests use real files and the raster exporter, without external CLIs.
use std::sync::atomic::AtomicBool;

use oracle_brand::export::{self, Plan, Report};
use oracle_brand::model::{Asset, Delivery, ExportPreset, Role, Variant};
use oracle_brand::project::{Project, hash_file};
use oracle_brand::render::Engines;

fn preset(id: &str, folder: &str, name: &str, size: u32) -> ExportPreset {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "format": "png", "width": size, "height": size,
        "folder": folder, "naming": name
    })).unwrap()
}

fn setup() -> (tempfile::TempDir, Project) {
    let tmp = tempfile::tempdir().unwrap();
    let mut p = Project::create(&tmp.path().join("project"), "Audit").unwrap();
    image::RgbaImage::from_pixel(32, 24, image::Rgba([20, 100, 200, 255]))
        .save(p.dir.join("source.png")).unwrap();
    p.data.assets.push(Asset {
        id: "logo".into(), role: Role::PrimaryLogo, label: "Logo".into(),
        variant: Variant::Color, source: "source.png".into(), source_hash: String::new(),
    });
    p.data.export_presets = vec![preset("a", "Logos", "logo", 16)];
    (tmp, p)
}

fn run(p: &mut Project, plan: &Plan) -> Report {
    export::run(p, &Engines::discover(), plan, None, &AtomicBool::new(false), |_, _, _| {})
}

#[test]
fn rejects_unsafe_preset_folders_and_preserves_master() {
    for folder in ["../../..", "..\\..\\..", "/absolute", "\\absolute", "C:/absolute", "C:relative", "Logos/../..", "Logos:stream", "CON", "Logos. "] {
        let (_tmp, mut p) = setup();
        p.data.export_presets[0] = preset("a", folder, "source", 16);
        let before = hash_file(&p.dir.join("source.png")).unwrap();
        let plan = export::plan(&p, &[], &[]);
        assert!(plan.outputs[0].blocked.is_some(), "unsafe folder accepted: {folder}");
        let report = run(&mut p, &plan);
        assert_eq!(report.succeeded(), 0, "{folder}");
        assert_eq!(hash_file(&p.dir.join("source.png")).unwrap(), before);
    }
}

#[test]
fn validates_hand_built_plans_before_writing() {
    let (_tmp, mut p) = setup();
    let before = hash_file(&p.dir.join("source.png")).unwrap();
    let mut plan = export::plan(&p, &[], &[]);
    plan.outputs[0].path = "../../../source.png".into();
    assert_eq!(run(&mut p, &plan).succeeded(), 0);
    assert_eq!(hash_file(&p.dir.join("source.png")).unwrap(), before);
}

#[test]
fn rejects_unsafe_delivery_folder_without_writing_metadata() {
    let (_tmp, mut p) = setup();
    let mut plan = export::plan(&p, &[], &[]);
    plan.folder = "../..".into();
    let report = run(&mut p, &plan);
    assert!(report.failures().next().is_some());
    assert!(!p.dir.join("manifest.json").exists());
    assert!(!p.dir.join("README.txt").exists());
    assert!(p.data.deliveries.is_empty());
}

#[test]
fn allocates_unique_final_names_including_numbered_and_case_variants() {
    let (_tmp, mut p) = setup();
    p.data.export_presets = vec![
        preset("a", "Logos", "logo", 10), preset("b", "Logos", "logo", 20),
        preset("c", "Logos", "logo-2", 30), preset("d", "logos", "LOGO", 40),
    ];
    let plan = export::plan(&p, &[], &[]);
    let paths: std::collections::BTreeSet<_> = plan.outputs.iter().map(|o| o.path.to_lowercase()).collect();
    assert_eq!(paths.len(), 4, "{:?}", plan.outputs);
    let report = run(&mut p, &plan);
    assert_eq!(report.succeeded(), 4, "{report:?}");
    for (output, expected) in plan.outputs.iter().zip([10, 20, 30, 40]) {
        assert_eq!(image::image_dimensions(report.folder.join(&output.path)).unwrap(), (expected, expected));
    }
}

#[test]
fn supports_root_exports_and_windows_subfolder_separators() {
    for folder in ["", "Logos\\PNG"] {
        let (_tmp, mut p) = setup();
        p.data.export_presets[0].folder = folder.into();
        let plan = export::plan(&p, &[], &[]);
        assert!(!plan.outputs[0].path.starts_with('/'));
        assert!(!plan.outputs[0].path.contains('\\'));
        let report = run(&mut p, &plan);
        assert_eq!(report.succeeded(), 1, "{report:?}");
    }
}

#[test]
fn cannot_overwrite_a_master_inside_the_delivery() {
    let (_tmp, mut p) = setup();
    let dest = p.deliveries_root().join("v1/Logos/logo.png");
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::copy(p.dir.join("source.png"), &dest).unwrap();
    p.data.assets[0].source = p.store_path(&dest);
    let before = hash_file(&dest).unwrap();
    let plan = export::plan(&p, &[], &[]);
    assert_eq!(run(&mut p, &plan).succeeded(), 0);
    assert_eq!(hash_file(&dest).unwrap(), before);
}

#[test]
fn cannot_replay_a_plan_into_a_frozen_delivery() {
    let (_tmp, mut p) = setup();
    let plan = export::plan(&p, &[], &[]);
    p.data.deliveries.push(Delivery { folder: plan.folder.clone(), created_at: String::new(), delivered: true, outputs: 0 });
    let report = run(&mut p, &plan);
    assert_eq!(report.succeeded(), 0);
    assert!(report.failures().next().is_some());
    assert!(!report.folder.exists());
}

#[cfg(windows)]
#[test]
fn rejects_a_directory_junction_leading_outside_the_delivery() {
    let (tmp, mut p) = setup();
    let outside = tmp.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    let root = p.deliveries_root().join("v1");
    std::fs::create_dir_all(&root).unwrap();
    let junction = root.join("Logos");
    let status = std::process::Command::new("cmd").args(["/c", "mklink", "/J"])
        .arg(&junction).arg(&outside).output().unwrap();
    assert!(status.status.success(), "{:?}", status);
    let plan = export::plan(&p, &[], &[]);
    let report = run(&mut p, &plan);
    assert_eq!(report.succeeded(), 0, "{report:?}");
    assert!(!outside.join("logo.png").exists());
    // Remove only the junction, never recursively traverse the target.
    std::fs::remove_dir(&junction).unwrap();
}

#[test]
fn rejects_duplicate_hand_built_destinations_before_writing() {
    let (_tmp, mut p) = setup();
    let mut plan = export::plan(&p, &[], &[]);
    let mut duplicate = plan.outputs[0].clone();
    duplicate.path = duplicate.path.to_uppercase();
    plan.outputs.push(duplicate);
    let report = run(&mut p, &plan);
    assert_eq!(report.succeeded(), 0);
    assert!(!report.folder.exists());
}

#[test]
fn carried_files_cannot_escape_or_be_counted_twice() {
    let (_tmp, mut p) = setup();
    let first = export::plan(&p, &[], &[]);
    let report = run(&mut p, &first);
    assert_eq!(report.succeeded(), 1);
    let manifest = report.folder.join("manifest.json");
    let mut data: serde_json::Value = serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    let original = data["outputs"][0].clone();
    let mut escaped = original.clone();
    escaped["path"] = "../../../source.png".into();
    let mut alias = original.clone();
    alias["path"] = "logos/LOGO.PNG".into();
    data["outputs"] = serde_json::json!([original, alias, escaped]);
    std::fs::write(&manifest, serde_json::to_vec(&data).unwrap()).unwrap();
    p.data.deliveries[0].delivered = true;
    let before = hash_file(&p.dir.join("source.png")).unwrap();
    let mut next = export::plan(&p, &[], &[]);
    next.outputs.clear();
    let report = run(&mut p, &next);
    assert_eq!(report.succeeded(), 1, "{report:?}");
    assert_eq!(report.failures().count(), 1, "unsafe carried path must be reported");
    assert_eq!(hash_file(&p.dir.join("source.png")).unwrap(), before);
    assert!(report.folder.join("Logos/logo.png").exists());
}

#[test]
fn cannot_target_a_subfolder_of_a_frozen_delivery() {
    let (_tmp, mut p) = setup();
    let mut plan = export::plan(&p, &[], &[]);
    plan.folder = "v1/new".into();
    p.data.deliveries.push(Delivery { folder: "v1".into(), created_at: String::new(), delivered: true, outputs: 0 });
    assert_eq!(run(&mut p, &plan).succeeded(), 0);
    assert!(!p.deliveries_root().join("v1").exists());
}
