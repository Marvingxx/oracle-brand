//! The spec's Mockup acceptance test, end to end on disk: an oblique business card with four
//! corners, a foreground (occlusion) mask and a linked logo; the logo changes and the scene
//! renders again at the same place; save, reopen, move the project, reuse the scene for another
//! brand; and broken inputs give clear errors without losing the previous render.

use std::path::{Path, PathBuf};

use oracle_brand::composite::zone_homography;
use oracle_brand::mockup::{self, Blend, Content, Paint, Scene, SceneState, Template};
use oracle_brand::mockup_demo;
use oracle_brand::model::{Asset, BrandColor, Format, Role, Variant};
use oracle_brand::project::hash_file;
use oracle_brand::render::Engines;
use oracle_brand::Project;

const RED: [u8; 3] = [200, 32, 41];
const BLUE: [u8; 3] = [46, 134, 171];
const GREEN: [u8; 3] = [29, 122, 58];

fn logo(left: [u8; 3], right: [u8; 3]) -> String {
    let hex = |c: [u8; 3]| format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="160" viewBox="0 0 400 160"><rect width="200" height="160" fill="{}"/><rect x="200" width="200" height="160" fill="{}"/></svg>"#,
        hex(left),
        hex(right)
    )
}

/// A brand project with the card template inside it (`Modèles/Carte oblique/`).
fn setup(root: &Path) -> (Project, PathBuf) {
    let dir = root.join("Marque Été");
    let mut p = Project::create(&dir, "Marque Été").unwrap();
    std::fs::create_dir_all(dir.join("Sources")).unwrap();
    std::fs::write(dir.join("Sources/logo principal.svg"), logo(RED, BLUE)).unwrap();
    let id = p.data.new_id("asset");
    p.data.assets.push(Asset { id, role: Role::PrimaryLogo, label: "Logo principal".into(), variant: Variant::Color, source: "Sources/logo principal.svg".into(), source_hash: String::new() });
    let cid = p.data.new_id("color");
    p.data.colors.push(BrandColor { id: cid, name: "Papier".into(), hex: "#F4F1EA".into(), ..Default::default() });
    p.save().unwrap();

    // The "photo" and the pen's mask, as a designer would have prepared them.
    let (photo, pen) = mockup_demo::business_card();
    let incoming = root.join("à importer");
    std::fs::create_dir_all(&incoming).unwrap();
    photo.save(incoming.join("carte oblique.png")).unwrap();
    pen.save(incoming.join("stylo.png")).unwrap();
    let templates = dir.join("Modèles");
    std::fs::create_dir_all(&templates).unwrap();
    let tfile = mockup::create_template(&templates, &incoming.join("carte oblique.png"), "Carte oblique").unwrap();
    let tdir = tfile.parent().unwrap().to_path_buf();
    let (_, mut t) = Template::load(&tfile).unwrap();
    t.zones[0].name = "Recto".into();
    t.zones[0].corners = mockup_demo::CARD;
    t.zones[0].ratio = [85.0, 55.0];
    t.occlusion = Some(mockup::adopt_file(&tdir, &incoming.join("stylo.png"), "premier-plan").unwrap());
    t.license = "Image de synthèse générée pour les tests".into();
    t.save(&tdir).unwrap();
    (p, tfile)
}

fn px(img: &image::RgbaImage, at: [f64; 2]) -> [u8; 3] {
    let p = img.get_pixel(at[0].floor() as u32, at[1].floor() as u32);
    [p[0], p[1], p[2]]
}

fn close(a: [u8; 3], b: [u8; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (i32::from(*x) - i32::from(y)).abs() <= 2)
}

#[test]
fn oblique_card_occlusion_linked_logo_then_logo_change() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut p, tfile) = setup(tmp.path());
    let photo_hash = hash_file(&tfile.parent().unwrap().join("photo.png")).unwrap();
    let engines = Engines::discover();

    // A scene: the main logo, contained at 80 %, over the palette's paper colour.
    let scene_file = mockup::create_scene(&mut p, &tfile, "Carte de visite").unwrap();
    assert_eq!(p.data.mockup_scenes, ["Mockups/Scènes/Carte de visite.json"]);
    let mut scene = Scene::load(&scene_file).unwrap();
    let asset = p.data.assets[0].clone();
    let fill = scene.fill_mut("zone-1");
    fill.content = Content::BrandAsset { asset_id: asset.id.clone(), role: asset.role, variant: asset.variant };
    fill.background = Some(Paint::BrandColor { color_id: p.data.colors[0].id.clone(), index: 0 });
    fill.scale = 0.8;
    scene.save(&scene_file).unwrap();
    assert_eq!(mockup::scene_state(&p, &scene, &[]), SceneState::NeverRendered);
    assert_eq!(mockup::scenes_using_asset(&p, &asset.id), ["Carte de visite"]);

    let out = mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap();
    assert_eq!(out, p.dir.join("Mockups/Carte de visite.png"));
    let img = image::open(&out).unwrap().to_rgba8();
    assert_eq!(img.dimensions(), mockup_demo::SIZE);
    let (photo, _) = mockup_demo::business_card();

    // Logo box in the zone: 85:55 zone, logo 2.5:1 at 80 % → u 0.1–0.9, v ≈ 0.253–0.747.
    let h = zone_homography(&mockup_demo::CARD, 1.0).unwrap();
    let at = |u: f64, v: f64| {
        let (x, y) = h.apply(u, v);
        [x, y]
    };
    assert!(close(px(&img, at(0.3, 0.5)), RED), "{:?}", px(&img, at(0.3, 0.5)));
    assert!(close(px(&img, at(0.7, 0.4)), BLUE));
    assert!(close(px(&img, at(0.05, 0.1)), [0xF4, 0xF1, 0xEA]), "paper colour from the palette");
    // Outside the card: the photo, untouched.
    for q in [[100.0, 100.0], [1500.0, 900.0], [800.0, 900.0]] {
        assert_eq!(px(&img, q), px(&photo, q));
    }
    // The pen stays in front of the logo: where it lies over the logo, the photo's pen shows.
    let pen = at(mockup_demo::PEN_UV.0, mockup_demo::PEN_UV.1);
    assert!(mockup_demo::under_pen(pen[0], pen[1], 0.0));
    assert_eq!(px(&img, pen), px(&photo, pen));
    let beside = at(0.75, 0.62);
    assert!(!mockup_demo::under_pen(beside[0], beside[1], 3.0));
    assert!(close(px(&img, beside), BLUE), "the logo next to the pen");
    assert_eq!(mockup::scene_state(&p, &Scene::load(&scene_file).unwrap(), &[]), SceneState::UpToDate);

    // The logo changes in the Brand Kit: the scene says so, and a new render puts the new logo
    // at the same place, with nothing to set again.
    std::fs::write(p.dir.join("Sources/logo principal.svg"), logo(RED, GREEN)).unwrap();
    let scene = Scene::load(&scene_file).unwrap();
    match mockup::scene_state(&p, &scene, &[]) {
        SceneState::Outdated(why) => assert_eq!(why, ["logo « Logo principal »"]),
        other => panic!("{other:?}"),
    }
    mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap();
    let img2 = image::open(&out).unwrap().to_rgba8();
    assert!(close(px(&img2, at(0.3, 0.5)), RED));
    assert!(close(px(&img2, at(0.7, 0.4)), GREEN));
    assert_eq!(px(&img2, pen), px(&photo, pen));
    assert_eq!(mockup::scene_state(&p, &Scene::load(&scene_file).unwrap(), &[]), SceneState::UpToDate);

    // The photo was never written.
    assert_eq!(hash_file(&tfile.parent().unwrap().join("photo.png")).unwrap(), photo_hash);

    // Save / reopen: same scene, same pixels (deterministic).
    let before = hash_file(&out).unwrap();
    let reopened = Scene::load(&scene_file).unwrap();
    assert_eq!(reopened.fills, scene.fills);
    mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap();
    assert_eq!(hash_file(&out).unwrap(), before);

    // Other formats and sizes.
    let mut s = Scene::load(&scene_file).unwrap();
    s.output.format = Format::Jpeg;
    s.output.width = Some(800);
    s.save(&scene_file).unwrap();
    assert!(matches!(mockup::scene_state(&p, &s, &[]), SceneState::Outdated(_)), "a setting changed");
    let jpg = mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap();
    assert_eq!(jpg.extension().unwrap(), "jpg");
    let j = image::open(&jpg).unwrap();
    assert_eq!((j.width(), j.height()), (800, 500));
    assert!(!j.color().has_alpha());
    s.output.format = Format::Webp;
    s.output.width = None;
    s.save(&scene_file).unwrap();
    let webp = mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap();
    assert_eq!(image::open(&webp).unwrap().width(), 1600);
    assert!(out.is_file(), "the PNG render is still there");
}

#[test]
fn a_moved_project_still_renders_and_a_scene_serves_another_brand() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut p, tfile) = setup(tmp.path());
    let engines = Engines::discover();
    let scene_file = mockup::create_scene(&mut p, &tfile, "Carte").unwrap();
    let mut scene = Scene::load(&scene_file).unwrap();
    let a = p.data.assets[0].clone();
    scene.fill_mut("zone-1").content = Content::BrandAsset { asset_id: a.id, role: a.role, variant: a.variant };
    scene.save(&scene_file).unwrap();
    mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap();

    // The whole project copied elsewhere (accents and spaces in the path): links still hold.
    let moved = tmp.path().join("Copie ailleurs").join("Marque déplacée");
    copy_dir(&p.dir, &moved);
    let q = Project::open(&moved).unwrap();
    let (_, sf, s) = mockup::scenes(&q).remove(0);
    let s = s.unwrap();
    assert!(sf.starts_with(&moved));
    assert_eq!(mockup::scene_state(&q, &s, &[]), SceneState::UpToDate);
    mockup::export_scene(&q, &sf, &engines, &[]).unwrap();

    // Another brand reuses the scene: its own primary logo goes in, found by role.
    let other_dir = tmp.path().join("Autre marque");
    let mut other = Project::create(&other_dir, "Autre marque").unwrap();
    std::fs::write(other_dir.join("logo.svg"), logo(GREEN, GREEN)).unwrap();
    let id = other.data.new_id("asset");
    other.data.assets.push(Asset { id: id.clone(), role: Role::PrimaryLogo, label: "Logo".into(), variant: Variant::Color, source: "logo.svg".into(), source_hash: String::new() });
    other.save().unwrap();
    let copied = mockup::copy_scene_to(&scene_file, &p, &mut other).unwrap();
    let cs = Scene::load(&copied).unwrap();
    assert!(matches!(&cs.fill("zone-1").unwrap().content, Content::BrandAsset { asset_id, .. } if *asset_id == id));
    let out = mockup::export_scene(&other, &copied, &engines, &[]).unwrap();
    let img = image::open(out).unwrap().to_rgba8();
    let c = mockup_demo::card_point(0.3, 0.5);
    assert!(close(px(&img, c), GREEN));
}

#[test]
fn broken_inputs_give_clear_errors_and_keep_the_previous_render() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut p, tfile) = setup(tmp.path());
    let engines = Engines::discover();
    let scene_file = mockup::create_scene(&mut p, &tfile, "Carte").unwrap();
    let mut scene = Scene::load(&scene_file).unwrap();
    let a = p.data.assets[0].clone();
    scene.fill_mut("zone-1").content = Content::BrandAsset { asset_id: a.id, role: a.role, variant: a.variant };
    scene.save(&scene_file).unwrap();
    let out = mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap();
    let good = hash_file(&out).unwrap();
    let tdir = tfile.parent().unwrap().to_path_buf();

    // Invalid SVG logo.
    std::fs::write(p.dir.join("Sources/logo principal.svg"), "<svg pas fermé").unwrap();
    let e = mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap_err();
    assert!(e.contains("Logo principal"), "{e}");
    assert_eq!(hash_file(&out).unwrap(), good);
    std::fs::write(p.dir.join("Sources/logo principal.svg"), logo(RED, BLUE)).unwrap();

    // Crossed corners.
    let (_, mut t) = Template::load(&tfile).unwrap();
    let good_corners = t.zones[0].corners;
    t.zones[0].corners.swap(1, 2);
    t.save(&tdir).unwrap();
    let e = mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap_err();
    assert!(e.contains("convexe"), "{e}");
    t.zones[0].corners = good_corners;

    // A mask without the photo's proportions.
    image::GrayImage::new(100, 100).save(tdir.join("masque carré.png")).unwrap();
    t.zones[0].mask = Some("masque carré.png".into());
    t.zones[0].blend = Blend::Multiply;
    t.save(&tdir).unwrap();
    let e = mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap_err();
    assert!(e.contains("proportions"), "{e}");
    t.zones[0].mask = None;
    t.save(&tdir).unwrap();

    // A corrupt photo.
    let photo = tdir.join("photo.png");
    let photo_bytes = std::fs::read(&photo).unwrap();
    std::fs::write(&photo, b"pas une image").unwrap();
    let state = mockup::scene_state(&p, &Scene::load(&scene_file).unwrap(), &[]);
    assert!(matches!(state, SceneState::Outdated(ref why) if why.contains(&"photo du modèle".to_string())), "{state:?}");
    let e = mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap_err();
    assert!(e.contains("photo du modèle"), "{e}");
    std::fs::write(&photo, photo_bytes).unwrap();

    // The template moved away: found again by id in the templates folder given.
    let elsewhere = tmp.path().join("Mes modèles");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::rename(&tdir, elsewhere.join("Carte oblique")).unwrap();
    let scene = Scene::load(&scene_file).unwrap();
    assert!(matches!(mockup::scene_state(&p, &scene, &[]), SceneState::Broken(ref e) if e.contains("introuvable")));
    assert!(mockup::export_scene(&p, &scene_file, &engines, &[]).unwrap_err().contains("introuvable"));
    mockup::export_scene(&p, &scene_file, &engines, &[elsewhere]).unwrap();

    // The previous good render was kept through every failure.
    assert!(out.is_file());
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let t = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &t);
        } else {
            std::fs::copy(e.path(), t).unwrap();
        }
    }
}
