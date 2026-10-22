//! The Brand Kit window on a demo project, light and dark, through each page and a real export.
//! Writes `tests/snapshots/*.png` to look at.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use crate::theme::ThemeChoice;
use crate::{App, Tab};

const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="160" viewBox="0 0 400 160">
  <rect x="10" y="20" width="120" height="120" rx="24" fill="#C82029"/>
  <circle cx="70" cy="80" r="32" fill="#FFFFFF"/>
  <rect x="150" y="50" width="230" height="22" fill="#1D1D1B"/>
  <rect x="150" y="88" width="160" height="22" fill="#2E86AB"/>
</svg>"##;
const SYMBOL: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="160" height="160" viewBox="0 0 160 160">
  <rect x="20" y="20" width="120" height="120" rx="24" fill="#C82029"/>
  <circle cx="80" cy="80" r="32" fill="#FFFFFF"/>
</svg>"##;

fn demo_project() -> std::path::PathBuf {
    use oracle_brand::model::{Asset, BrandColor, BrandFont, Role, Variant};
    let dir = std::env::temp_dir().join(format!("oracle-brand-kit-ui-{}", std::process::id())).join("Marque Exemple");
    let _ = std::fs::remove_dir_all(&dir);
    let mut p = oracle_brand::Project::create(&dir, "Marque Exemple").unwrap();
    p.data.client = "Client fictif".into();
    std::fs::create_dir_all(dir.join("Sources")).unwrap();
    std::fs::write(dir.join("Sources/logo-principal.svg"), LOGO).unwrap();
    std::fs::write(dir.join("Sources/symbole.svg"), SYMBOL).unwrap();
    for (role, label, file) in [(Role::PrimaryLogo, "Logo principal", "logo-principal.svg"), (Role::Symbol, "Symbole", "symbole.svg")] {
        let id = p.data.new_id("asset");
        let source = p.store_path(&dir.join("Sources").join(file));
        p.data.assets.push(Asset { id, role, label: label.into(), variant: Variant::Color, source, source_hash: String::new() });
    }
    for (name, hex) in [("Rouge principal", "#C82029"), ("Noir", "#1D1D1B"), ("Bleu", "#2E86AB")] {
        let id = p.data.new_id("color");
        p.data.colors.push(BrandColor { id, name: name.into(), hex: hex.into(), ..Default::default() });
    }
    for (family, usage) in [("Lexend", "Titres"), ("Inter", "Texte courant")] {
        let id = p.data.new_id("font");
        p.data.fonts.push(BrandFont { id, family: family.into(), usage: usage.into(), weights: vec!["Regular".into(), "SemiBold".into()], ..Default::default() });
    }
    p.save().unwrap();
    dir
}

fn harness(theme: ThemeChoice, dir: &std::path::Path) -> Harness<'static, App> {
    let mut app = App::new(None);
    app.settings.theme = theme;
    app.open(dir);
    app.status = None; // the message names a temporary folder that changes every run
    Harness::builder().with_size(egui::vec2(1280.0, 820.0)).build_ui_state(|ui, app: &mut App| app.draw(ui), app)
}

fn settle(h: &mut Harness<'_, App>, frames: usize) {
    for _ in 0..frames {
        h.step();
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

#[test]
fn pages_light_and_dark_then_export() {
    let dir = demo_project();
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for (theme, suffix) in [(ThemeChoice::Light, "light"), (ThemeChoice::Dark, "dark")] {
        let mut h = harness(theme, &dir);
        settle(&mut h, 60);
        assert_eq!(h.get_all_by_label_contains("Jamais exporté").count(), 2);
        h.snapshot(format!("logos_{suffix}"));
        for (tab, name, expect) in [(Tab::Colors, "colors", "RVB 200 32 41"), (Tab::Export, "export", "vers Livrables/Marque Exemple/v1"), (Tab::Presets, "presets", "Avatar")] {
            h.state_mut().tab = tab;
            settle(&mut h, 8);
            h.get_by_label_contains(expect);
            h.snapshot(format!("{name}_{suffix}"));
        }
        snapshots.extend_harness(&mut h);
    }

    // A real export from the window (Oracle Vector when installed or built, resvg otherwise for SVG).
    let mut h = harness(ThemeChoice::Light, &dir);
    h.state_mut().tab = Tab::Export;
    settle(&mut h, 6);
    h.get_by_label("Exporter 16 fichiers").click();
    for _ in 0..4 {
        h.step();
    }
    let status = h.state().status.as_ref().map(|s| s.0.clone());
    assert!(h.state().running.is_some() || !h.state().last_results.is_empty(), "the click started nothing ({status:?})");
    for _ in 0..2400 {
        h.step();
        if h.state().running.is_none() && !h.state().last_results.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let results = h.state().last_results.clone();
    assert!(!results.is_empty(), "the export never finished");
    let failed: Vec<_> = results.iter().filter(|r| r.error.is_some()).collect();
    if h.state().engines.vector_cli.is_some() {
        assert!(failed.is_empty(), "{failed:?}");
    }
    settle(&mut h, 6);
    h.get_by_label_contains("DERNIER EXPORT : 16 RÉUSSI(S), 0 ÉCHEC(S)");
    h.snapshot("export_done_light");
    // A fixed date, for a stable picture.
    if let Some(p) = h.state_mut().project.as_mut() {
        p.data.deliveries[0].created_at = "2026-10-22T12:00:00Z".into();
    }
    h.state_mut().tab = Tab::Deliveries;
    settle(&mut h, 6);
    h.get_by_label("Marquer comme livrée").click();
    settle(&mut h, 30);
    h.get_by_label("Livrée");
    h.snapshot("deliveries_light");
    snapshots.extend_harness(&mut h);
    // Saved to brand.json.
    drop(h);
    let saved = oracle_brand::Project::open(&dir).unwrap();
    assert!(saved.data.deliveries.first().is_some_and(|d| d.delivered && d.outputs > 0));
    assert!(dir.join("Livrables/Marque Exemple/v1/manifest.json").is_file());
}
