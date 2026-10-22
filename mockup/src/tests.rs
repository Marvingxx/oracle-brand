//! The Mockup Studio window on a demo brand and the synthetic business-card template, light and
//! dark: the scene, the template editor, then a real render from the window. Writes
//! `tests/snapshots/*.png` to look at.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use oracle_brand::mockup::{self, Content, Paint, Scene, Template};
use oracle_brand::model::{Asset, BrandColor, Role, Variant};

use crate::{App, Mode, theme::ThemeChoice};

const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="160" viewBox="0 0 400 160">
  <rect x="10" y="20" width="120" height="120" rx="24" fill="#C82029"/>
  <circle cx="70" cy="80" r="32" fill="#FFFFFF"/>
  <rect x="150" y="50" width="230" height="22" fill="#1D1D1B"/>
  <rect x="150" y="88" width="160" height="22" fill="#2E86AB"/>
</svg>"##;

/// A fixed folder (not per run): the status bar shows the templates folder in the pictures.
fn root() -> std::path::PathBuf {
    std::env::temp_dir().join("oracle-mockup-studio-ui")
}

fn demo() -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let root = root();
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("Marque Exemple");
    let mut p = oracle_brand::Project::create(&dir, "Marque Exemple").unwrap();
    std::fs::create_dir_all(dir.join("Sources")).unwrap();
    std::fs::write(dir.join("Sources/logo-principal.svg"), LOGO).unwrap();
    let id = p.data.new_id("asset");
    p.data.assets.push(Asset { id: id.clone(), role: Role::PrimaryLogo, label: "Logo principal".into(), variant: Variant::Color, source: "Sources/logo-principal.svg".into(), source_hash: String::new() });
    for (name, hex) in [("Rouge principal", "#C82029"), ("Noir", "#1D1D1B"), ("Papier", "#F4F1EA")] {
        let cid = p.data.new_id("color");
        p.data.colors.push(BrandColor { id: cid, name: name.into(), hex: hex.into(), ..Default::default() });
    }
    p.save().unwrap();

    let photos = root.join("Photos");
    std::fs::create_dir_all(&photos).unwrap();
    let (photo, pen) = oracle_brand::mockup_demo::business_card();
    photo.save(photos.join("carte-oblique.png")).unwrap();
    pen.save(photos.join("stylo.png")).unwrap();
    let templates = root.join("Modèles");
    std::fs::create_dir_all(&templates).unwrap();
    let tfile = mockup::create_template(&templates, &photos.join("carte-oblique.png"), "Carte oblique").unwrap();
    let tdir = tfile.parent().unwrap();
    let (_, mut t) = Template::load(&tfile).unwrap();
    t.category = "Papeterie".into();
    t.zones[0].name = "Recto".into();
    t.zones[0].corners = oracle_brand::mockup_demo::CARD;
    t.zones[0].ratio = [85.0, 55.0];
    t.occlusion = Some(mockup::adopt_file(tdir, &photos.join("stylo.png"), "premier-plan").unwrap());
    t.save(tdir).unwrap();

    let scene_file = mockup::create_scene(&mut p, &tfile, "Carte de visite").unwrap();
    let mut s = Scene::load(&scene_file).unwrap();
    let fill = s.fill_mut("zone-1");
    fill.content = Content::BrandAsset { asset_id: id, role: Role::PrimaryLogo, variant: Variant::Color };
    fill.background = Some(Paint::BrandColor { color_id: p.data.colors[2].id.clone(), index: 2 });
    fill.scale = 0.7;
    s.save(&scene_file).unwrap();
    (dir, tfile, scene_file)
}

fn harness(theme: ThemeChoice, dir: &std::path::Path) -> Harness<'static, App> {
    let mut app = App::new(None);
    app.settings.theme = theme;
    app.settings.templates_root = root().join("Modèles").display().to_string();
    app.open_project(dir);
    app.status = None;
    Harness::builder().with_size(egui::vec2(1440.0, 880.0)).build_ui_state(|ui, app: &mut App| app.draw(ui), app)
}

/// Steps until the preview shows the latest request.
fn settle(h: &mut Harness<'_, App>) {
    for _ in 0..600 {
        h.step();
        let ready = h.state().preview.as_ref().is_some_and(|p| p.photo_size.is_some() && !p.busy()) && !h.state().preview_dirty;
        if ready {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    for _ in 0..4 {
        h.step();
    }
}

#[test]
fn scene_and_template_light_and_dark_then_render() {
    let (dir, tfile, scene_file) = demo();
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for (theme, suffix) in [(ThemeChoice::Light, "light"), (ThemeChoice::Dark, "dark")] {
        let mut h = harness(theme, &dir);
        settle(&mut h);
        assert!(matches!(h.state().mode, Mode::Scene { .. }), "the project's first scene opens");
        assert!(h.state().preview.as_ref().unwrap().error.is_none(), "{:?}", h.state().preview.as_ref().unwrap().error);
        h.get_by_label_contains("Jamais rendue");
        h.get_by_label("Rendre la scène");
        h.snapshot(format!("scene_{suffix}"));
        h.state_mut().open_template(&tfile, Some(scene_file.clone()));
        settle(&mut h);
        h.get_by_label("← Retour à la scène");
        h.get_by_label_contains("Masque : premier-plan.png");
        h.snapshot(format!("template_{suffix}"));
        snapshots.extend_harness(&mut h);
    }

    // A real render from the window.
    let mut h = harness(ThemeChoice::Light, &dir);
    settle(&mut h);
    h.get_by_label("Rendre la scène").click();
    for _ in 0..1200 {
        h.step();
        if h.state().rendering.is_none() && dir.join("Mockups/Carte de visite.png").is_file() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(h.state().rendering.is_none(), "the render never finished");
    let out = dir.join("Mockups/Carte de visite.png");
    assert!(out.is_file());
    assert_eq!(image::image_dimensions(&out).unwrap(), oracle_brand::mockup_demo::SIZE);
    h.state_mut().status = None;
    settle(&mut h);
    h.get_by_label("À jour");
    h.snapshot("scene_rendered_light");
    snapshots.extend_harness(&mut h);

    // The logo changes in the Brand Kit: the scene shows it is out of date.
    std::fs::write(dir.join("Sources/logo-principal.svg"), LOGO.replace("#1D1D1B", "#2E86AB")).unwrap();
    h.state_mut().refresh_states();
    settle(&mut h);
    h.get_by_label_contains("À mettre à jour : logo « Logo principal »");
    h.get_by_label("Mettre à jour les scènes (1)");
}
