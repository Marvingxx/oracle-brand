//! The right column: the open scene's settings (what goes in each zone, placement, render) or the
//! open template's (zones, masks, foreground, overlays, rights).

use std::path::Path;

use egui::{Color32, RichText};
use oracle_brand::mockup::{self, Blend, Content, Fit, Overlay, Paint, Zone};
use oracle_brand::model::{Format, parse_hex};

use crate::{App, Mode, theme};

/// A Brand Kit logo being dragged onto a zone.
#[derive(Clone, Debug)]
pub struct AssetPayload(pub String);

pub fn inspector(app: &mut App, ui: &mut egui::Ui) {
    match app.mode {
        Mode::Scene { .. } => scene(app, ui),
        Mode::Template { .. } => template(app, ui),
        Mode::None => {}
    }
}

/// Puts a Brand Kit master in the selected zone.
pub fn set_content_asset(app: &mut App, asset_id: &str) {
    let Some(project) = &app.project else { return };
    let Some(a) = project.data.assets.iter().find(|a| a.id == asset_id).cloned() else { return };
    let zone = app.zone;
    if let Mode::Scene { scene, template, .. } = &mut app.mode
        && let Some(z) = template.zones.get(zone)
    {
        scene.fill_mut(&z.id).content = Content::BrandAsset { asset_id: a.id, role: a.role, variant: a.variant };
        app.touch();
    }
}

/// Puts a file in the selected zone: a Brand Kit master when it is one (linked), else the file.
pub fn set_content_file(app: &mut App, file: &Path) {
    let Some(project) = &app.project else { return };
    let canon = |p: &Path| std::fs::canonicalize(p).ok();
    if let Some(a) = project.data.assets.iter().find(|a| canon(&project.resolve(&a.source)).is_some_and(|c| Some(c) == canon(file))) {
        let id = a.id.clone();
        set_content_asset(app, &id);
        app.say("Logo du Brand Kit lié à la zone.", false);
        return;
    }
    let stored = project.store_path(file);
    let zone = app.zone;
    if let Mode::Scene { scene, template, .. } = &mut app.mode
        && let Some(z) = template.zones.get(zone)
    {
        scene.fill_mut(&z.id).content = Content::File { path: stored };
        app.touch();
    }
}

fn swatch(ui: &mut egui::Ui, rgb: [u8; 3]) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
    ui.painter().rect(r, 3.0, Color32::from_rgb(rgb[0], rgb[1], rgb[2]), egui::Stroke::new(1.0, theme::tokens(ui.ctx()).line), egui::StrokeKind::Inside);
}

fn scene(app: &mut App, ui: &mut egui::Ui) {
    let t = theme::tokens(ui.ctx());
    let Some(project) = app.project.clone() else { return };
    let zone_index = app.zone;
    let mut changed = false;
    let mut edit_template = None;
    let mut copy_to: Option<std::path::PathBuf> = None;
    let mut render = None;
    let mut to_library = None;
    let state = match &app.mode {
        Mode::Scene { file, .. } => app.states.get(file).cloned(),
        _ => None,
    };
    let Mode::Scene { file, scene, template_file, template } = &mut app.mode else { return };

    // The action first, always in view.
    let render_path = scene.render_path(&project);
    // Wrapped: the three buttons are wider than the inspector.
    ui.horizontal_wrapped(|ui| {
        if ui.add_enabled(app.rendering.is_none(), egui::Button::new(RichText::new("Rendre la scène").color(t.on_accent)).fill(t.accent).min_size(egui::vec2(0.0, 34.0))).clicked() {
            render = Some(file.clone());
        }
        if render_path.is_file() && ui.button("Ouvrir le rendu").clicked() {
            let _ = std::process::Command::new("explorer").arg(&render_path).spawn();
        }
        if render_path.is_file() && ui.button("Ajouter à la Bibliothèque").on_hover_text("Une copie du rendu dans « Mes créations » de la Bibliothèque Oracle").clicked() {
            to_library = Some(render_path.clone());
        }
    });
    let color = match &state {
        Some(mockup::SceneState::UpToDate) => t.good,
        Some(mockup::SceneState::Outdated(_)) => t.warn,
        Some(mockup::SceneState::Broken(_)) => t.bad,
        _ => t.muted,
    };
    ui.label(RichText::new(state.as_ref().map(|s| s.label_fr()).unwrap_or_default()).color(color).size(12.5));
    ui.label(RichText::new(format!("→ {}", project.store_path(&render_path))).color(t.muted).size(11.5));
    ui.add_space(10.0);

    theme::label(ui, "Scène");
    changed |= ui.add(egui::TextEdit::singleline(&mut scene.name).desired_width(f32::INFINITY)).changed();
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("Modèle : {}", template.name)).color(t.muted));
        if ui.small_button("Modifier le modèle").clicked() {
            edit_template = Some((template_file.clone(), file.clone()));
        }
    });
    if ui.small_button("Copier vers une autre marque…").on_hover_text("La même mise en situation pour un autre projet de marque : ses logos sont retrouvés par rôle, ses couleurs par position").clicked() {
        copy_to = Some(file.clone());
    }

    ui.add_space(10.0);
    theme::label(ui, "Zone");
    if template.zones.is_empty() {
        ui.label(RichText::new("Ce modèle n'a pas de zone.").color(t.bad));
        return;
    }
    let zi = zone_index.min(template.zones.len() - 1);
    ui.horizontal_wrapped(|ui| {
        for (i, z) in template.zones.iter().enumerate() {
            if ui.selectable_label(i == zi, &z.name).clicked() {
                app.zone = i;
            }
        }
    });
    let zone = template.zones[zi].clone();
    let fill = scene.fill_mut(&zone.id);

    ui.add_space(8.0);
    theme::label(ui, "Contenu");
    ui.label(RichText::new("Glissez un logo sur la zone, ou cliquez-le.").color(t.muted).size(12.0));
    let mut chosen = None;
    ui.horizontal_wrapped(|ui| {
        for a in &project.data.assets {
            let on = matches!(&fill.content, Content::BrandAsset { asset_id, .. } if *asset_id == a.id);
            let label = if a.label.is_empty() { a.role.label_fr().to_string() } else { a.label.clone() };
            let text = RichText::new(format!("{label} · {}", a.variant.label_fr())).color(if on { t.on_accent } else { t.ink }).size(12.5);
            let id = egui::Id::new(("brand-asset", &a.id));
            let resp = ui.dnd_drag_source(id, AssetPayload(a.id.clone()), |ui| {
                egui::Frame::new().fill(if on { t.accent } else { t.panel }).stroke(egui::Stroke::new(1.0, t.line)).corner_radius(theme::RADIUS).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| ui.label(text));
            });
            if resp.response.clicked() {
                chosen = Some(a.id.clone());
            }
        }
    });
    if project.data.assets.is_empty() {
        ui.label(RichText::new("Le Brand Kit de cette marque n'a pas encore de logo.").color(t.muted).size(12.0));
    }
    if let Some(id) = chosen
        && let Some(a) = project.data.assets.iter().find(|a| a.id == id)
    {
        fill.content = Content::BrandAsset { asset_id: a.id.clone(), role: a.role, variant: a.variant };
        changed = true;
    }
    let mut pick_file = false;
    ui.horizontal(|ui| {
        match &fill.content {
            Content::File { path } => {
                ui.label(RichText::new(format!("Image : {}", Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())).size(12.5));
            }
            Content::None => {
                ui.label(RichText::new("Zone vide").color(t.muted).size(12.5));
            }
            Content::BrandAsset { .. } => {}
        }
        if ui.small_button("Autre image…").clicked() {
            pick_file = true;
        }
        if !matches!(fill.content, Content::None) && ui.small_button("Vider").clicked() {
            fill.content = Content::None;
            changed = true;
        }
    });

    ui.add_space(8.0);
    theme::label(ui, "Fond de la zone");
    let current = match &fill.background {
        None => "Aucun (la photo)".to_string(),
        Some(Paint::Hex { hex }) => hex.clone(),
        Some(Paint::BrandColor { color_id, .. }) => project.data.colors.iter().find(|c| &c.id == color_id).map(|c| c.name.clone()).unwrap_or_else(|| "Couleur retirée de la palette".into()),
    };
    egui::ComboBox::from_id_salt("background").selected_text(current).width(ui.available_width() - 8.0).show_ui(ui, |ui| {
        if ui.selectable_label(fill.background.is_none(), "Aucun (la photo)").clicked() {
            fill.background = None;
            changed = true;
        }
        for (index, c) in project.data.colors.iter().enumerate() {
            ui.horizontal(|ui| {
                if let Some(rgb) = parse_hex(&c.hex) {
                    swatch(ui, rgb);
                }
                let on = matches!(&fill.background, Some(Paint::BrandColor { color_id, .. }) if *color_id == c.id);
                if ui.selectable_label(on, format!("{} · {}", c.name, c.hex)).clicked() {
                    fill.background = Some(Paint::BrandColor { color_id: c.id.clone(), index });
                    changed = true;
                }
            });
        }
        if ui.selectable_label(matches!(fill.background, Some(Paint::Hex { .. })), "Autre couleur…").clicked() && !matches!(fill.background, Some(Paint::Hex { .. })) {
            fill.background = Some(Paint::Hex { hex: "#FFFFFF".into() });
            changed = true;
        }
    });
    if let Some(Paint::Hex { hex }) = &mut fill.background {
        ui.horizontal(|ui| {
            if let Some(rgb) = parse_hex(hex) {
                swatch(ui, rgb);
            }
            let r = ui.add(egui::TextEdit::singleline(hex).desired_width(90.0));
            if r.changed() && parse_hex(hex).is_some() {
                changed = true;
            }
            if parse_hex(hex).is_none() {
                ui.label(RichText::new("#RRGGBB attendu").color(t.bad).size(12.0));
            }
        });
    }

    ui.add_space(8.0);
    theme::label(ui, "Placement");
    ui.horizontal(|ui| {
        for f in Fit::ALL {
            if ui.selectable_label(fill.fit == f, f.label_fr()).clicked() {
                fill.fit = f;
                changed = true;
            }
        }
    });
    let mut pct = fill.scale * 100.0;
    if ui.add(egui::Slider::new(&mut pct, 5.0..=300.0).text("Taille").suffix(" %").fixed_decimals(0)).changed() {
        fill.scale = pct / 100.0;
        changed = true;
    }
    changed |= ui.add(egui::Slider::new(&mut fill.rotation, -180.0..=180.0).text("Rotation").suffix("°").step_by(0.5)).changed();
    let mut ox = fill.offset[0] * 100.0;
    let mut oy = fill.offset[1] * 100.0;
    let a = ui.add(egui::Slider::new(&mut ox, -100.0..=100.0).text("Décalage X").suffix(" %").fixed_decimals(1));
    let b = ui.add(egui::Slider::new(&mut oy, -100.0..=100.0).text("Décalage Y").suffix(" %").fixed_decimals(1));
    if a.changed() || b.changed() {
        fill.offset = [ox / 100.0, oy / 100.0];
        changed = true;
    }
    if ui.small_button("Recentrer").clicked() {
        fill.offset = [0.0, 0.0];
        fill.rotation = 0.0;
        changed = true;
    }

    ui.add_space(10.0);
    theme::label(ui, "Format du rendu");
    ui.horizontal(|ui| {
        for f in [Format::Png, Format::Jpeg, Format::Webp] {
            if ui.selectable_label(scene.output.format == f, f.ext().to_uppercase()).clicked() {
                scene.output.format = f;
                changed = true;
            }
        }
    });
    ui.horizontal(|ui| {
        let mut own = scene.output.width.is_none();
        if ui.checkbox(&mut own, "Taille de la photo").changed() {
            scene.output.width = if own { None } else { Some(2000) };
            changed = true;
        }
        if let Some(w) = &mut scene.output.width {
            changed |= ui.add(egui::DragValue::new(w).range(64..=oracle_brand::render::MAX_SIDE).suffix(" px de large")).changed();
        }
    });
    if scene.output.format == Format::Jpeg {
        let mut q = scene.output.quality.unwrap_or(92);
        if ui.add(egui::Slider::new(&mut q, 50..=100).text("Qualité JPEG")).changed() {
            scene.output.quality = Some(q);
            changed = true;
        }
    }

    if pick_file
        && let Some(f) = rfd::FileDialog::new().set_title("Image ou fichier vectoriel").add_filter("Images", &["svg", "png", "jpg", "jpeg", "webp", "tif", "tiff", "pdf", "ai", "eps", "psd", "oracle-vector"]).pick_file()
    {
        set_content_file(app, &f);
    }
    if changed {
        app.touch();
    }
    if let Some(scene_file) = copy_to {
        app.save_now();
        copy_scene_to_brand(app, &scene_file);
    }
    if let Some((tf, back)) = edit_template {
        app.open_template(&tf, Some(back));
    }
    if let Some(f) = render {
        app.render(vec![f]);
    }
    if let Some(f) = to_library {
        app.library.add_files(&[f]);
    }
}

fn file_name(stored: &str) -> String {
    Path::new(stored).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| stored.to_string())
}

fn pick_image(title: &str) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().set_title(title).add_filter("Images", &["png", "jpg", "jpeg", "webp", "tif", "tiff"]).pick_file()
}

fn template(app: &mut App, ui: &mut egui::Ui) {
    let t = theme::tokens(ui.ctx());
    let photo_size = app.preview.as_ref().and_then(|p| p.photo_size);
    let mut changed = false;
    let mut back_to = None;
    let mut new_scene = None;
    let mut error = None;
    let has_project = app.project.is_some();
    let Mode::Template { file, template, back } = &mut app.mode else { return };
    let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();

    if let Some(b) = back.clone()
        && ui.button("← Retour à la scène").clicked()
    {
        back_to = Some(b);
    }
    theme::label(ui, "Modèle");
    changed |= ui.add(egui::TextEdit::singleline(&mut template.name).hint_text("Nom").desired_width(f32::INFINITY)).changed();
    changed |= ui.add(egui::TextEdit::singleline(&mut template.category).hint_text("Catégorie (Papeterie, Enseigne…)").desired_width(f32::INFINITY)).changed();
    ui.label(RichText::new(match photo_size {
        Some([w, h]) => format!("Photo : {} · {w} × {h} px", template.photo),
        None => format!("Photo : {}", template.photo),
    }).color(t.muted).size(12.0));
    if has_project && ui.button("Nouvelle scène avec ce modèle").clicked() {
        new_scene = Some(file.clone());
    }

    ui.add_space(10.0);
    theme::label(ui, "Zones (de bas en haut)");
    let mut zi = app.zone.min(template.zones.len().saturating_sub(1));
    let mut action = None;
    for (i, z) in template.zones.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui.selectable_label(i == zi, &z.name).clicked() {
                zi = i;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Retirer").clicked() {
                    action = Some(("del", i));
                }
                if i + 1 < template.zones.len() && ui.small_button("↑").on_hover_text("Monter (devant)").clicked() {
                    action = Some(("up", i));
                }
                if i > 0 && ui.small_button("↓").on_hover_text("Descendre (derrière)").clicked() {
                    action = Some(("down", i));
                }
            });
        });
    }
    match action {
        Some(("del", i)) => {
            template.zones.remove(i);
            zi = zi.min(template.zones.len().saturating_sub(1));
            changed = true;
        }
        Some(("up", i)) => {
            template.zones.swap(i, i + 1);
            zi = i + 1;
            changed = true;
        }
        Some(("down", i)) => {
            template.zones.swap(i, i - 1);
            zi = i - 1;
            changed = true;
        }
        _ => {}
    }
    if ui.button("Ajouter une zone").clicked() {
        let id = template.new_zone_id();
        let [w, h] = photo_size.map(|[w, h]| [f64::from(w), f64::from(h)]).unwrap_or([1000.0, 1000.0]);
        let mut z = Zone::new(&id, &format!("Zone {}", template.zones.len() + 1));
        z.corners = [[w * 0.35, h * 0.35], [w * 0.65, h * 0.35], [w * 0.65, h * 0.65], [w * 0.35, h * 0.65]];
        z.ratio = [(w * 0.3).round(), (h * 0.3).round()];
        template.zones.push(z);
        zi = template.zones.len() - 1;
        changed = true;
    }
    app.zone = zi;

    if let Some(z) = template.zones.get_mut(zi) {
        ui.add_space(8.0);
        theme::label(ui, "Zone sélectionnée");
        changed |= ui.add(egui::TextEdit::singleline(&mut z.name).desired_width(f32::INFINITY)).changed();
        ui.horizontal(|ui| {
            ui.label("Proportions réelles");
            changed |= ui.add(egui::DragValue::new(&mut z.ratio[0]).range(0.1..=100_000.0).speed(0.5)).changed();
            ui.label("×");
            changed |= ui.add(egui::DragValue::new(&mut z.ratio[1]).range(0.1..=100_000.0).speed(0.5)).changed();
        });
        ui.label(RichText::new("Ex. 85 × 55 pour une carte de visite, 210 × 297 pour un A4.").color(t.muted).size(11.5));
        ui.add_space(4.0);
        ui.label(RichText::new("Coins (px de la photo) — glissez-les sur l'image, flèches pour ajuster").color(t.muted).size(12.0));
        for (k, name) in ["Haut gauche", "Haut droit", "Bas droit", "Bas gauche"].iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(*name).size(12.5));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    changed |= ui.add(egui::DragValue::new(&mut z.corners[k][1]).speed(1.0).prefix("y ")).changed();
                    changed |= ui.add(egui::DragValue::new(&mut z.corners[k][0]).speed(1.0).prefix("x ")).changed();
                });
            });
        }
        if oracle_brand::composite::zone_homography(&z.corners, 1.0).is_err() {
            ui.label(RichText::new("Les coins se croisent ou sont dans le mauvais ordre (haut-gauche, haut-droit, bas-droit, bas-gauche).").color(t.bad).size(12.0));
        }
        if ui.small_button("Remettre en rectangle").on_hover_text("Rectangle englobant les quatre coins").clicked() {
            let xs = z.corners.map(|c| c[0]);
            let ys = z.corners.map(|c| c[1]);
            let (x0, x1) = (xs.iter().copied().fold(f64::MAX, f64::min), xs.iter().copied().fold(f64::MIN, f64::max));
            let (y0, y1) = (ys.iter().copied().fold(f64::MAX, f64::min), ys.iter().copied().fold(f64::MIN, f64::max));
            z.corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
            changed = true;
        }
        ui.add_space(4.0);
        egui::ComboBox::from_label("Fusion").selected_text(z.blend.label_fr()).show_ui(ui, |ui| {
            for b in Blend::ALL {
                changed |= ui.selectable_value(&mut z.blend, b, b.label_fr()).changed();
            }
        });
        let mut op = z.opacity * 100.0;
        if ui.add(egui::Slider::new(&mut op, 0.0..=100.0).text("Opacité").suffix(" %").fixed_decimals(0)).changed() {
            z.opacity = op / 100.0;
            changed = true;
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new(match &z.mask {
                Some(m) => format!("Masque : {}", file_name(m)),
                None => "Pas de masque (toute la zone)".into(),
            }).size(12.5));
        });
        ui.horizontal(|ui| {
            if ui.small_button("Choisir un masque…").on_hover_text("Image noir et blanc (ou transparente) cadrée comme la photo : blanc = le design apparaît").clicked()
                && let Some(f) = pick_image("Masque de la zone")
            {
                match mockup::adopt_file(&dir, &f, &format!("masque-{}", z.id)) {
                    Ok(s) => {
                        z.mask = Some(s);
                        changed = true;
                    }
                    Err(e) => error = Some(e),
                }
            }
            if z.mask.is_some() && ui.small_button("Retirer").clicked() {
                z.mask = None;
                changed = true;
            }
        });
    }

    ui.add_space(10.0);
    theme::label(ui, "Premier plan");
    ui.label(RichText::new(match &template.occlusion {
        Some(m) => format!("Masque : {}", file_name(m)),
        None => "Aucun : rien ne passe devant les designs.".into(),
    }).size(12.5));
    ui.horizontal(|ui| {
        if ui.small_button("Choisir…").on_hover_text("Blanc = ce qui reste devant le design (une main, un objet), cadré comme la photo").clicked()
            && let Some(f) = pick_image("Masque de premier plan")
        {
            match mockup::adopt_file(&dir, &f, "premier-plan") {
                Ok(s) => {
                    template.occlusion = Some(s);
                    changed = true;
                }
                Err(e) => error = Some(e),
            }
        }
        if template.occlusion.is_some() && ui.small_button("Retirer").clicked() {
            template.occlusion = None;
            changed = true;
        }
    });

    ui.add_space(10.0);
    theme::label(ui, "Surcouches (ombres, reflets, grain)");
    let mut remove = None;
    for (i, o) in template.overlays.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(RichText::new(file_name(&o.file)).size(12.5));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Retirer").clicked() {
                    remove = Some(i);
                }
            });
        });
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(("overlay-blend", i)).selected_text(o.blend.label_fr()).width(150.0).show_ui(ui, |ui| {
                for b in Blend::ALL {
                    changed |= ui.selectable_value(&mut o.blend, b, b.label_fr()).changed();
                }
            });
            let mut op = o.opacity * 100.0;
            if ui.add(egui::DragValue::new(&mut op).range(0.0..=100.0).suffix(" %")).changed() {
                o.opacity = op / 100.0;
                changed = true;
            }
        });
    }
    if let Some(i) = remove {
        template.overlays.remove(i);
        changed = true;
    }
    if ui.small_button("Ajouter une surcouche…").on_hover_text("Image cadrée comme la photo, posée au-dessus de tout").clicked()
        && let Some(f) = pick_image("Surcouche")
    {
        match mockup::adopt_file(&dir, &f, &format!("surcouche-{}", template.overlays.len() + 1)) {
            Ok(s) => {
                template.overlays.push(Overlay { name: String::new(), file: s, blend: Blend::Multiply, opacity: 1.0 });
                changed = true;
            }
            Err(e) => error = Some(e),
        }
    }

    ui.add_space(10.0);
    theme::label(ui, "Droits d'usage");
    changed |= ui.add(egui::TextEdit::singleline(&mut template.source).hint_text("Source de la photo").desired_width(f32::INFINITY)).changed();
    changed |= ui.add(egui::TextEdit::multiline(&mut template.license).hint_text("Licence, conditions d'usage").desired_rows(2).desired_width(f32::INFINITY)).changed();

    if let Some(e) = error {
        app.say(e, true);
    }
    if changed {
        app.touch();
    }
    if let Some(f) = new_scene {
        app.save_now();
        app.new_scene(&f);
    } else if let Some(b) = back_to {
        app.save_now();
        app.reload_templates();
        app.open_scene(&b);
    }
}

/// Copies a scene into another brand project the designer picks.
fn copy_scene_to_brand(app: &mut App, scene_file: &Path) {
    let Some(from) = app.project.clone() else { return };
    let Some(dir) = rfd::FileDialog::new().set_title("Projet de marque de destination (dossier contenant brand.json)").pick_folder() else { return };
    let result = oracle_brand::Project::open(&dir).and_then(|mut to| {
        if to.data.project_id == from.data.project_id {
            return Err("c'est le même projet de marque".into());
        }
        mockup::copy_scene_to(scene_file, &from, &mut to).map(|path| (to.data.brand_name.clone(), path))
    });
    match result {
        Ok((brand, path)) => app.say(format!("Scène copiée dans « {brand} » : {}. Ouvrez ce projet pour la rendre.", path.display()), false),
        Err(e) => app.say(format!("Copie impossible : {e}"), true),
    }
}
