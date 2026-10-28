//! The Brand Kit's pages. Each edits the open project; changes are saved shortly after (see
//! `App::touch`).

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Color32, RichText};
use oracle_brand::export;
use oracle_brand::model::{Asset, BrandColor, BrandFont, Cmyk, ExportPreset, ForbiddenUse, Format, Role, Status, Variant, parse_hex};
use oracle_brand::project::SourceState;

use crate::theme::{self, RADIUS};
use crate::thumbs::Thumb;
use crate::{App, Progress, Running, Tab};

fn heading(ui: &mut egui::Ui, title: &str, sub: &str) {
    let t = theme::tokens(ui.ctx());
    ui.heading(title);
    if !sub.is_empty() {
        ui.label(RichText::new(sub).color(t.muted));
    }
    ui.add_space(14.0);
}

fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let t = theme::tokens(ui.ctx());
    egui::Frame::new()
        .fill(t.panel)
        .stroke(egui::Stroke::new(1.0, t.line))
        .corner_radius(RADIUS)
        .inner_margin(14)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add(ui)
        })
        .inner
}

fn field(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str, width: f32) -> bool {
    ui.horizontal(|ui| {
        ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new(label).color(theme::tokens(ui.ctx()).muted)));
        ui.add(egui::TextEdit::singleline(value).hint_text(hint).desired_width(width)).changed()
    })
    .inner
}

fn swatch(ui: &mut egui::Ui, hex: &str, size: egui::Vec2) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let t = theme::tokens(ui.ctx());
    match parse_hex(hex) {
        Some([r, g, b]) => {
            ui.painter().rect_filled(rect, RADIUS, Color32::from_rgb(r, g, b));
        }
        None => {
            ui.painter().rect_filled(rect, RADIUS, t.sunken);
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "?", egui::FontId::proportional(14.0), t.bad);
        }
    }
    ui.painter().rect_stroke(rect, RADIUS, egui::Stroke::new(1.0, t.line), egui::StrokeKind::Inside);
    resp
}

// ---------------------------------------------------------------- Identité

pub fn identity(app: &mut App, ui: &mut egui::Ui) {
    let Some(p) = app.project.as_mut() else { return };
    let mut changed = false;
    heading(ui, "Identité", "Les informations générales du projet de marque.");
    card(ui, |ui| {
        changed |= field(ui, "Marque", &mut p.data.brand_name, "Nom de la marque", 360.0);
        changed |= field(ui, "Client", &mut p.data.client, "Entreprise ou personne", 360.0);
        changed |= field(ui, "Version", &mut p.data.version, "v1", 120.0);
        ui.horizontal(|ui| {
            ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Statut").color(theme::tokens(ui.ctx()).muted)));
            egui::ComboBox::from_id_salt("status").selected_text(p.data.status.label_fr()).show_ui(ui, |ui| {
                for s in Status::ALL {
                    changed |= ui.selectable_value(&mut p.data.status, s, s.label_fr()).changed();
                }
            });
        });
        ui.horizontal_top(|ui| {
            ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Notes").color(theme::tokens(ui.ctx()).muted)));
            changed |= ui.add(egui::TextEdit::multiline(&mut p.data.notes).desired_rows(5).desired_width(520.0)).changed();
        });
    });
    ui.add_space(12.0);
    let t = theme::tokens(ui.ctx());
    ui.label(RichText::new(format!("Dossier : {}", p.dir.display())).color(t.muted).size(12.0));
    ui.label(RichText::new(format!("Créé le {} · identifiant {}", date_fr(&p.data.created_at), p.data.project_id)).color(t.muted).size(12.0));
    if ui.button("Ouvrir le dossier du projet").clicked() {
        open_path(&p.dir);
    }
    if changed {
        app.touch();
    }
}

// ---------------------------------------------------------------- Logos

/// Adds master files as logos (role "Autre" except the very first, "Logo principal"): the designer
/// sets the role and declares the variant; nothing is derived.
pub fn add_masters(app: &mut App, files: Vec<PathBuf>) {
    let Some(p) = app.project.as_mut() else { return };
    let mut added = 0;
    let mut refused = Vec::new();
    for f in files {
        if oracle_brand::render::master_kind(&f).is_none() {
            refused.push(f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            continue;
        }
        let source = p.store_path(&f);
        if p.data.assets.iter().any(|a| a.source == source) {
            continue;
        }
        let role = if p.data.assets.is_empty() { Role::PrimaryLogo } else { Role::Other };
        let label = f.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let id = p.data.new_id("asset");
        p.data.assets.push(Asset { id, role, label, variant: Variant::Color, source, source_hash: String::new() });
        added += 1;
    }
    if !refused.is_empty() {
        app.say(format!("Format non pris en charge : {}", refused.join(", ")), true);
    } else if added > 0 {
        app.say(format!("{added} fichier(s) ajouté(s). Indiquez leur rôle et leur variante."), false);
    }
    if added > 0 {
        app.touch();
    }
}

pub fn logos(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Logos", "Les fichiers maîtres, chacun dans un rôle. Glissez des fichiers sur la fenêtre pour les ajouter.");
    ui.horizontal(|ui| {
        if theme::primary(ui, "Ajouter des fichiers maîtres…").clicked()
            && let Some(files) = rfd::FileDialog::new()
                .add_filter("Logos", &["svg", "pdf", "ai", "eps", "oracle-vector", "png", "jpg", "jpeg", "webp", "tif", "tiff", "psd", "oracle-edit", "pcraft"])
                .pick_files()
        {
            add_masters(app, files);
        }
        if ui.button("Vérifier les sources").on_hover_text("Compare chaque fichier avec son état au dernier export").clicked() {
            let n = app.project.as_ref().map(export::stale).map(|s| s.len()).unwrap_or(0);
            app.say(if n == 0 { "Toutes les sources sont à jour.".to_string() } else { format!("{n} source(s) à regarder (modifiée, introuvable ou jamais exportée).") }, false);
        }
    });
    ui.add_space(12.0);
    let Some(p) = app.project.as_mut() else { return };
    if p.data.assets.is_empty() {
        let t = theme::tokens(ui.ctx());
        card(ui, |ui| {
            ui.label(RichText::new("Aucun logo pour l'instant.").strong());
            ui.label(RichText::new("Ajoutez les fichiers que vous avez dessinés (SVG, PDF, AI, EPS, .oracle-vector, PNG, PSD…). Ils restent à leur place : Oracle ne les modifie jamais.").color(t.muted));
        });
        return;
    }
    let t = theme::tokens(ui.ctx());
    let mut changed = false;
    let mut remove = None;
    let mut relink = None;
    let mut vectorize = None;
    let width = ui.available_width();
    let cols = ((width + 12.0) / 300.0).floor().max(1.0) as usize;
    let card_w = (width - 12.0 * (cols as f32 - 1.0)) / cols as f32;
    let dir = p.dir.clone();
    let states: Vec<SourceState> = p.data.assets.iter().map(|a| p.source_state(a)).collect();
    let paths: Vec<PathBuf> = p.data.assets.iter().map(|a| p.resolve(&a.source)).collect();
    // The Mockup Studio scenes using each logo, and whether their render is out of date.
    let scenes: Vec<_> = oracle_brand::mockup::scenes(p).into_iter().filter_map(|(_, _, s)| s.ok()).map(|s| (oracle_brand::mockup::scene_state(p, &s, &[]), s)).collect();
    let mockups: Vec<Vec<(String, bool)>> = p
        .data
        .assets
        .iter()
        .map(|a| {
            scenes
                .iter()
                .filter(|(_, s)| s.fills.iter().any(|f| matches!(&f.content, oracle_brand::mockup::Content::BrandAsset { asset_id, .. } if *asset_id == a.id)))
                .map(|(st, s)| (s.name.clone(), !matches!(st, oracle_brand::mockup::SceneState::UpToDate)))
                .collect()
        })
        .collect();
    for (row, chunk) in p.data.assets.chunks_mut(cols).enumerate() {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            for (j, a) in chunk.iter_mut().enumerate() {
                let i = row * cols + j;
                ui.allocate_ui_with_layout(egui::vec2(card_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_width(card_w);
                    card(ui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(card_w - 28.0, 150.0), egui::Sense::hover());
                        // A checkerboard shows transparency.
                        checker(ui, rect);
                        match app.thumbs.get(&paths[i]) {
                            Thumb::Ready(tex, [w, h]) => {
                                let s = ((rect.width() - 24.0) / *w as f32).min((rect.height() - 24.0) / *h as f32).min(4.0);
                                let r = egui::Rect::from_center_size(rect.center(), egui::vec2(*w as f32 * s, *h as f32 * s));
                                ui.painter().image(tex.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                            }
                            Thumb::Loading => {
                                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "…", egui::FontId::proportional(18.0), t.muted);
                            }
                            Thumb::Failed(e) => {
                                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "Aperçu impossible", egui::FontId::proportional(12.0), t.bad);
                                ui.interact(rect, ui.id().with(("err", i)), egui::Sense::hover()).on_hover_text(e.clone());
                            }
                        }
                        ui.add_space(6.0);
                        changed |= ui.add(egui::TextEdit::singleline(&mut a.label).hint_text("Nom").desired_width(f32::INFINITY)).changed();
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt(("role", &a.id)).selected_text(a.role.label_fr()).width(card_w * 0.5 - 24.0).show_ui(ui, |ui| {
                                for r in Role::ALL {
                                    changed |= ui.selectable_value(&mut a.role, r, r.label_fr()).changed();
                                }
                            });
                            egui::ComboBox::from_id_salt(("variant", &a.id)).selected_text(a.variant.label_fr()).width(card_w * 0.5 - 24.0).show_ui(ui, |ui| {
                                for v in Variant::ALL {
                                    changed |= ui.selectable_value(&mut a.variant, v, v.label_fr()).changed();
                                }
                            });
                        });
                        let st = states[i];
                        let color = match st {
                            SourceState::Unchanged => t.good,
                            SourceState::New => t.muted,
                            SourceState::Modified => t.warn,
                            SourceState::Missing => t.bad,
                        };
                        ui.label(RichText::new(format!("● {}", st.label_fr())).color(color).size(12.0));
                        ui.label(RichText::new(&a.source).color(t.muted).size(11.0)).on_hover_text(paths[i].display().to_string());
                        if !mockups[i].is_empty() {
                            let outdated = mockups[i].iter().filter(|(_, o)| *o).count();
                            let names = mockups[i].iter().map(|(n, o)| if *o { format!("{n} (à mettre à jour)") } else { n.clone() }).collect::<Vec<_>>().join(", ");
                            let text = RichText::new(format!("Mockups : {names}")).size(11.5).color(if outdated > 0 { t.warn } else { t.muted });
                            ui.label(text).on_hover_text("Scènes de Mockup Studio qui utilisent ce logo. Mettez-les à jour dans Mockup Studio.");
                        }
                        ui.horizontal(|ui| {
                            if ui.small_button("Montrer").clicked() {
                                reveal(&paths[i]);
                            }
                            if ui.small_button("Remplacer le fichier…").clicked() {
                                relink = Some(i);
                            }
                            if oracle_brand::render::master_kind(&paths[i]) == Some("raster")
                                && ui
                                    .small_button("Vectoriser")
                                    .on_hover_text("Crée un maître SVG à côté de l'image (VTracer), à vérifier ; l'image n'est pas modifiée")
                                    .clicked()
                            {
                                vectorize = Some(i);
                            }
                            if ui.small_button("Retirer").on_hover_text("Retire le logo du projet. Le fichier n'est pas supprimé.").clicked() {
                                remove = Some(i);
                            }
                        });
                    });
                });
            }
        });
        ui.add_space(12.0);
    }
    if let Some(i) = relink
        && let Some(f) = rfd::FileDialog::new().set_directory(&dir).pick_file()
    {
        if oracle_brand::render::master_kind(&f).is_some() {
            p.data.assets[i].source = p.store_path(&f);
            changed = true;
        } else {
            app.say("Format non pris en charge.", true);
        }
    }
    if let Some(i) = vectorize {
        let traced = app.project.as_mut().map(|p| {
            let id = p.data.assets[i].id.clone();
            oracle_brand::vectorize::vectorize(p, &id, oracle_brand::vectorize::Detail::Logo)
        });
        match traced.unwrap_or_else(|| Err("aucun projet ouvert".into())) {
            Ok(_) => {
                changed = true;
                app.say("Maître SVG créé à côté de l'image : vérifiez-le avant de l'exporter.", false);
            }
            Err(e) => app.say(format!("Vectorisation impossible : {e}"), true),
        }
    }
    if let Some(i) = remove {
        let p = app.project.as_mut().map(|p| p.data.assets.remove(i));
        if let Some(a) = p {
            app.say(format!("« {} » retiré du projet (le fichier reste sur le disque).", a.label), false);
        }
        changed = true;
    }
    if changed {
        app.touch();
    }
}

fn checker(ui: &egui::Ui, rect: egui::Rect) {
    let t = theme::tokens(ui.ctx());
    let (a, b) = if t.dark { (Color32::from_gray(40), Color32::from_gray(52)) } else { (Color32::from_gray(250), Color32::from_gray(232)) };
    ui.painter().rect_filled(rect, RADIUS, a);
    let s = 10.0;
    let mut y = rect.top();
    let mut row = 0;
    while y < rect.bottom() {
        let mut x = rect.left() + if row % 2 == 0 { 0.0 } else { s };
        while x < rect.right() {
            let r = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(s, s)).intersect(rect);
            ui.painter().rect_filled(r, 0.0, b);
            x += 2.0 * s;
        }
        y += s;
        row += 1;
    }
}

// ---------------------------------------------------------------- Couleurs

pub fn colors(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Couleurs", "La palette, avec les valeurs que vous fournissez. Aucune conversion CMJN n'est faite sans profil ICC.");
    let Some(p) = app.project.as_mut() else { return };
    let t = theme::tokens(ui.ctx());
    let mut changed = false;
    if theme::primary(ui, "Ajouter une couleur").clicked() {
        let id = p.data.new_id("color");
        p.data.colors.push(BrandColor { id, name: format!("Couleur {}", p.data.colors.len() + 1), hex: "#000000".into(), ..Default::default() });
        changed = true;
    }
    ui.add_space(10.0);
    let mut remove = None;
    let mut swap = None;
    let n = p.data.colors.len();
    for (i, c) in p.data.colors.iter_mut().enumerate() {
        card(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    swatch(ui, &c.hex, egui::vec2(96.0, 72.0));
                    // The colour picker edits the hex value.
                    if let Some(mut rgb) = parse_hex(&c.hex)
                        && egui::color_picker::color_edit_button_srgb(ui, &mut rgb).on_hover_text("Choisir la couleur").changed()
                    {
                        c.hex = format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                        changed = true;
                    }
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        changed |= ui.add(egui::TextEdit::singleline(&mut c.name).hint_text("Nom").desired_width(220.0)).changed();
                        let hex_ok = parse_hex(&c.hex).is_some();
                        let r = ui.add(egui::TextEdit::singleline(&mut c.hex).hint_text("#RRGGBB").desired_width(90.0).text_color(if hex_ok { t.ink } else { t.bad }));
                        changed |= r.changed();
                        if let Some(rgb) = parse_hex(&c.hex) {
                            let (h, s, l) = oracle_brand::model::hsl(rgb);
                            ui.label(RichText::new(format!("RVB {} {} {}   ·   TSL {:.0}° {:.0} % {:.0} %", rgb[0], rgb[1], rgb[2], h, s, l)).color(t.muted).size(12.0));
                        } else {
                            ui.label(RichText::new("Hexadécimal attendu : #RRGGBB").color(t.bad).size(12.0));
                        }
                    });
                    ui.horizontal(|ui| {
                        let mut has = c.cmyk.is_some();
                        if ui.checkbox(&mut has, "CMJN").changed() {
                            c.cmyk = if has { Some(Cmyk::default()) } else { None };
                            changed = true;
                        }
                        if let Some(k) = c.cmyk.as_mut() {
                            for (lbl, v) in [("C", &mut k.c), ("M", &mut k.m), ("J", &mut k.y), ("N", &mut k.k)] {
                                ui.label(lbl);
                                changed |= ui.add(egui::DragValue::new(v).range(0.0..=100.0).speed(0.5).suffix(" %")).changed();
                            }
                            changed |= ui.add(egui::TextEdit::singleline(&mut k.profile).hint_text("profil ICC d'origine (vide = saisi)").desired_width(200.0)).changed();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Pantone").color(t.muted));
                        changed |= ui.add(egui::TextEdit::singleline(&mut c.pantone).hint_text("ex. 186 C").desired_width(110.0)).changed();
                        ui.label(RichText::new("RAL").color(t.muted));
                        changed |= ui.add(egui::TextEdit::singleline(&mut c.ral).hint_text("ex. 3020").desired_width(80.0)).changed();
                        changed |= ui.add(egui::TextEdit::singleline(&mut c.notes).hint_text("Notes d'usage").desired_width(260.0)).changed();
                    });
                    ui.horizontal(|ui| {
                        if ui.add_enabled(i > 0, egui::Button::new("↑").small()).clicked() {
                            swap = Some((i, i - 1));
                        }
                        if ui.add_enabled(i + 1 < n, egui::Button::new("↓").small()).clicked() {
                            swap = Some((i, i + 1));
                        }
                        if ui.small_button("Supprimer").clicked() {
                            remove = Some(i);
                        }
                    });
                });
            });
        });
        ui.add_space(8.0);
    }
    if let Some((a, b)) = swap {
        p.data.colors.swap(a, b);
        changed = true;
    }
    if let Some(i) = remove {
        p.data.colors.remove(i);
        changed = true;
    }
    if changed {
        app.touch();
    }
}

// ---------------------------------------------------------------- Typographies

pub fn fonts(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Typographies", "Les familles de la marque et leur usage. Les fichiers de police ne sont jamais livrés sans droit de redistribution.");
    let Some(p) = app.project.as_mut() else { return };
    let t = theme::tokens(ui.ctx());
    let mut changed = false;
    if theme::primary(ui, "Ajouter une typographie").clicked() {
        let id = p.data.new_id("font");
        p.data.fonts.push(BrandFont { id, family: String::new(), weights: vec!["Regular".into()], ..Default::default() });
        changed = true;
    }
    ui.add_space(10.0);
    let mut remove = None;
    let dir = p.dir.clone();
    for (i, f) in p.data.fonts.iter_mut().enumerate() {
        card(ui, |ui| {
            changed |= field(ui, "Famille", &mut f.family, "ex. Lexend", 260.0);
            changed |= field(ui, "Usage", &mut f.usage, "Titres, texte courant…", 360.0);
            let mut weights = f.weights.join(", ");
            if field(ui, "Graisses", &mut weights, "Regular, SemiBold", 360.0) {
                f.weights = weights.split(',').map(|w| w.trim().to_string()).filter(|w| !w.is_empty()).collect();
                changed = true;
            }
            changed |= field(ui, "Licence (lien)", &mut f.license_url, "https://…", 360.0);
            ui.horizontal(|ui| {
                ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Redistribution").color(t.muted)));
                changed |= ui.checkbox(&mut f.redistributable, "La licence permet de remettre les fichiers au client").changed();
            });
            ui.horizontal(|ui| {
                ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Fichiers locaux").color(t.muted)));
                ui.label(RichText::new(if f.local_files.is_empty() { "aucun".to_string() } else { f.local_files.join(" · ") }).color(t.muted).size(12.0));
                if ui.small_button("Associer…").clicked()
                    && let Some(files) = rfd::FileDialog::new().add_filter("Polices", &["ttf", "otf", "woff", "woff2", "ttc"]).set_directory(&dir).pick_files()
                {
                    f.local_files = files.iter().map(|x| x.display().to_string()).collect();
                    changed = true;
                }
            });
            if !f.family.is_empty() {
                let sample = RichText::new("Aa Bb Cc — Portez ce vieux whisky au juge blond qui fume").size(18.0);
                ui.label(sample).on_hover_text("Aperçu dans la police de l'interface : la police de la marque n'est pas chargée par Oracle.");
            }
            if ui.small_button("Supprimer").clicked() {
                remove = Some(i);
            }
        });
        ui.add_space(8.0);
    }
    if let Some(i) = remove {
        p.data.fonts.remove(i);
        changed = true;
    }
    if changed {
        app.touch();
    }
}

// ---------------------------------------------------------------- Règles

pub fn rules(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Règles d'usage", "Vos règles, telles que vous les écrivez. Elles alimentent le README des livraisons et le Brand Guide.");
    let Some(p) = app.project.as_mut() else { return };
    let mut changed = false;
    let r = &mut p.data.rules;
    card(ui, |ui| {
        changed |= field(ui, "Zone de protection", &mut r.clear_space, "ex. la hauteur du symbole sur chaque côté", 480.0);
        changed |= field(ui, "Taille minimale", &mut r.min_size, "ex. 24 px à l'écran, 15 mm en impression", 480.0);
        changed |= field(ui, "Fonds autorisés", &mut r.allowed_backgrounds, "ex. blanc, noir, rouge principal", 480.0);
    });
    ui.add_space(12.0);
    theme::label(ui, "Usages interdits");
    let mut remove = None;
    let dir = p.dir.clone();
    for (i, f) in r.forbidden.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            changed |= ui.add(egui::TextEdit::singleline(&mut f.text).hint_text("ex. Ne pas déformer le logo").desired_width(420.0)).changed();
            let ex = if f.example.is_empty() { "Exemple…".to_string() } else { format!("Exemple : {}", f.example.rsplit(['/', '\\']).next().unwrap_or("")) };
            if ui.button(ex).on_hover_text("Un visuel que vous avez préparé (facultatif)").clicked()
                && let Some(file) = rfd::FileDialog::new().set_directory(&dir).pick_file()
            {
                f.example = file.display().to_string();
                changed = true;
            }
            if ui.small_button("Supprimer").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        r.forbidden.remove(i);
        changed = true;
    }
    if ui.button("Ajouter un usage interdit").clicked() {
        r.forbidden.push(ForbiddenUse::default());
        changed = true;
    }
    if changed {
        app.touch();
    }
}

// ---------------------------------------------------------------- Presets

pub fn presets(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Presets d'export", "Comment chaque fichier livré est fabriqué. Noms et dossiers : {marque} {role} {variante} {traitement} {format} {preset} {version} {w} {h}.");
    let Some(p) = app.project.as_mut() else { return };
    let t = theme::tokens(ui.ctx());
    let mut changed = false;
    ui.horizontal(|ui| {
        let has_matrix = p.data.export_presets.iter().any(|x| x.id.starts_with("matrix-"));
        if ui
            .add_enabled(!has_matrix, egui::Button::new("Ajouter la matrice de livraison"))
            .on_hover_text("Chaque logo en couleur, inversé, noir, blanc et niveaux de gris, en SVG, PDF, EPS, AI, PNG et JPEG, rangé par logo, traitement et format")
            .clicked()
        {
            p.data.export_presets.extend(oracle_brand::presets::delivery_matrix());
            changed = true;
        }
        if theme::primary(ui, "Nouveau preset").clicked() {
            let id = p.data.new_id("preset");
            p.data.export_presets.push(ExportPreset {
                id,
                name: "Nouveau preset".into(),
                format: Format::Png,
                width: Some(1000),
                height: None,
                scale: 1.0,
                margin: 0.0,
                background: None,
                dpi: None,
                quality: None,
                folder: "Autres".into(),
                naming: "{marque}_{role}_{variante}_{w}".into(),
                roles: Vec::new(),
                treatments: Vec::new(),
            });
            changed = true;
        }
        if ui.button("Rétablir les presets d'origine").on_hover_text("Ajoute les presets d'Oracle manquants ; vos presets restent").clicked() {
            for d in oracle_brand::presets::defaults() {
                if !p.data.export_presets.iter().any(|x| x.id == d.id) {
                    p.data.export_presets.push(d);
                    changed = true;
                }
            }
        }
    });
    ui.add_space(10.0);
    let mut remove = None;
    let mut dup = None;
    for (i, pr) in p.data.export_presets.iter_mut().enumerate() {
        egui::CollapsingHeader::new(RichText::new(format!("{}   ·   {}", pr.name, pr.format.ext().to_uppercase())).strong()).id_salt(&pr.id).show(ui, |ui| {
            card(ui, |ui| {
                changed |= field(ui, "Nom", &mut pr.name, "", 300.0);
                ui.horizontal(|ui| {
                    ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Format").color(t.muted)));
                    for f in Format::ALL {
                        changed |= ui.selectable_value(&mut pr.format, f, f.ext().to_uppercase()).changed();
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Traitements").color(t.muted)))
                        .on_hover_text("Versions calculées depuis le maître couleur ; une version que vous avez dessinée (même rôle, cette variante) est livrée à la place. Aucune case : le maître tel quel.");
                    for tr in oracle_brand::recolor::Treatment::ALL {
                        let mut on = pr.treatments.contains(&tr);
                        if ui.checkbox(&mut on, tr.label_fr()).changed() {
                            if on {
                                pr.treatments.push(tr);
                            } else {
                                pr.treatments.retain(|x| *x != tr);
                            }
                            changed = true;
                        }
                    }
                });
                if pr.format.is_raster() {
                    ui.horizontal(|ui| {
                        ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Taille (px)").color(t.muted)));
                        changed |= opt_u32(ui, "largeur", &mut pr.width);
                        ui.label("×");
                        changed |= opt_u32(ui, "hauteur", &mut pr.height);
                        if pr.width.is_none() && pr.height.is_none() {
                            ui.label("échelle");
                            changed |= ui.add(egui::DragValue::new(&mut pr.scale).range(0.05..=50.0).speed(0.05).suffix(" ×")).changed();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Marge").color(t.muted)));
                        let mut pct = pr.margin * 100.0;
                        if ui.add(egui::Slider::new(&mut pct, 0.0..=40.0).suffix(" %")).changed() {
                            pr.margin = pct / 100.0;
                            changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Fond").color(t.muted)));
                        let mut transparent = pr.background.is_none();
                        if pr.format != Format::Jpeg && ui.checkbox(&mut transparent, "Transparent").changed() {
                            pr.background = if transparent { None } else { Some("#FFFFFF".into()) };
                            changed = true;
                        }
                        if pr.format == Format::Jpeg && pr.background.is_none() {
                            pr.background = Some("#FFFFFF".into());
                            changed = true;
                        }
                        if let Some(bg) = pr.background.as_mut() {
                            swatch(ui, bg, egui::vec2(28.0, 28.0));
                            changed |= ui.add(egui::TextEdit::singleline(bg).desired_width(90.0)).changed();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Densité").color(t.muted)));
                        changed |= opt_u32(ui, "dpi", &mut pr.dpi);
                        if pr.format == Format::Jpeg {
                            let mut q = pr.quality.unwrap_or(92);
                            if ui.add(egui::Slider::new(&mut q, 50..=100).text("qualité")).changed() {
                                pr.quality = Some(q);
                                changed = true;
                            }
                        }
                    });
                }
                changed |= field(ui, "Dossier", &mut pr.folder, "Logos/PNG", 300.0);
                changed |= field(ui, "Nom des fichiers", &mut pr.naming, "{marque}_{role}_{variante}", 360.0);
                ui.horizontal_wrapped(|ui| {
                    ui.add_sized([150.0, 28.0], egui::Label::new(RichText::new("Rôles").color(t.muted)));
                    let mut all = pr.roles.is_empty();
                    if ui.checkbox(&mut all, "Tous").changed() {
                        pr.roles = if all { Vec::new() } else { vec![Role::PrimaryLogo] };
                        changed = true;
                    }
                    if !all {
                        for r in Role::ALL {
                            let mut on = pr.roles.contains(&r);
                            if ui.checkbox(&mut on, r.label_fr()).changed() {
                                if on { pr.roles.push(r) } else { pr.roles.retain(|x| *x != r) }
                                changed = true;
                            }
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.small_button("Dupliquer").clicked() {
                        dup = Some(i);
                    }
                    if ui.small_button("Supprimer").clicked() {
                        remove = Some(i);
                    }
                });
            });
        });
    }
    if let Some(i) = dup {
        let mut copy = p.data.export_presets[i].clone();
        copy.id = p.data.new_id("preset");
        copy.name = format!("{} (copie)", copy.name);
        p.data.export_presets.insert(i + 1, copy);
        changed = true;
    }
    if let Some(i) = remove {
        p.data.export_presets.remove(i);
        changed = true;
    }
    if changed {
        app.touch();
    }
}

fn opt_u32(ui: &mut egui::Ui, hint: &str, v: &mut Option<u32>) -> bool {
    let mut s = v.map(|x| x.to_string()).unwrap_or_default();
    let r = ui.add(egui::TextEdit::singleline(&mut s).hint_text(hint).desired_width(70.0));
    if r.changed() {
        *v = s.trim().parse::<u32>().ok().filter(|x| *x > 0 && *x <= oracle_brand::render::MAX_SIDE);
        return true;
    }
    false
}

// ---------------------------------------------------------------- Export

/// Where Mockup Studio keeps templates (its setting, else its default), and the project's own.
fn mockup_template_roots(p: &oracle_brand::Project) -> Vec<PathBuf> {
    let configured = dirs::config_dir()
        .and_then(|d| std::fs::read(d.join("Oracle").join("mockup-studio.json")).ok())
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| v.get("templates_root").and_then(|r| r.as_str()).filter(|r| !r.is_empty()).map(PathBuf::from));
    let default = dirs::document_dir().map(|d| d.join("Oracle").join("Modèles de mockups"));
    configured.or(default).into_iter().chain(std::iter::once(p.dir.join("Modèles"))).collect()
}

pub fn export(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Export", "Choisissez les logos et les presets, vérifiez la liste, puis lancez. Les fichiers maîtres ne sont jamais modifiés.");
    let t = theme::tokens(ui.ctx());
    if let Some(run) = &app.running {
        card(ui, |ui| {
            ui.label(RichText::new(format!("Export en cours : {} / {}", run.done, run.total)).strong());
            ui.add(egui::ProgressBar::new(if run.total == 0 { 0.0 } else { run.done as f32 / run.total as f32 }).fill(t.accent));
            ui.label(RichText::new(&run.current).color(t.muted).size(12.0));
            if ui.button("Annuler").on_hover_text("S'arrête après le fichier en cours ; les fichiers faits restent").clicked() {
                run.cancel.store(true, Ordering::Relaxed);
            }
        });
        return;
    }
    let Some(p) = app.project.as_ref() else { return };
    let mut replan = app.plan.is_none();
    ui.columns(2, |cols| {
        theme::label(&mut cols[0], "Logos");
        for a in &p.data.assets {
            let mut on = app.plan_assets.is_empty() || app.plan_assets.contains(&a.id);
            let st = p.source_state(a);
            let text = if a.label.is_empty() || a.label == a.role.label_fr() {
                format!("{} · {}", a.role.label_fr(), a.variant.label_fr())
            } else {
                format!("{} — {} · {}", a.label, a.role.label_fr(), a.variant.label_fr())
            };
            let resp = cols[0].checkbox(&mut on, text);
            if st == SourceState::Modified || st == SourceState::Missing {
                cols[0].label(RichText::new(format!("    {}", st.label_fr())).color(if st == SourceState::Missing { t.bad } else { t.warn }).size(12.0));
            }
            if resp.changed() {
                toggle(&mut app.plan_assets, &a.id, on, p.data.assets.iter().map(|x| x.id.clone()));
                replan = true;
            }
        }
        cols[0].add_space(4.0);
        if cols[0].horizontal(|ui| ui.button("Seulement les sources modifiées").clicked()).inner {
            app.plan_assets = export::stale(p).into_iter().filter(|(_, s)| *s != SourceState::Missing).map(|(id, _)| id).collect();
            if app.plan_assets.is_empty() {
                app.status = Some(("Aucune source modifiée depuis le dernier export.".into(), false, std::time::Instant::now()));
            }
            replan = true;
        }
        theme::label(&mut cols[1], "Presets");
        for pr in &p.data.export_presets {
            let mut on = app.plan_presets.is_empty() || app.plan_presets.contains(&pr.id);
            let ext = pr.format.ext().to_uppercase();
            let text = if pr.name.to_uppercase().contains(&ext) { pr.name.clone() } else { format!("{} ({ext})", pr.name) };
            if cols[1].checkbox(&mut on, text).changed() {
                toggle(&mut app.plan_presets, &pr.id, on, p.data.export_presets.iter().map(|x| x.id.clone()));
                replan = true;
            }
        }
    });
    let scenes = oracle_brand::mockup::scenes(p).len();
    if scenes > 0 {
        ui.add_space(6.0);
        if ui.checkbox(&mut app.plan_mockups, format!("Mockups de Mockup Studio ({scenes} scène(s), rendus à jour seulement)")).changed() {
            replan = true;
        }
    }
    if replan {
        let mut plan = export::plan(p, &app.plan_assets, &app.plan_presets);
        if app.plan_mockups {
            export::plan_mockups(p, &mut plan, &mockup_template_roots(p));
        }
        app.plan_carry = export::carried_over(p, &plan).map(|(folder, files)| (folder, files.len()));
        app.plan = Some(plan);
    }
    let Some(plan) = app.plan.clone() else { return };
    ui.add_space(14.0);
    let blocked = plan.outputs.iter().filter(|o| o.blocked.is_some()).count();
    let delivered_last = p.data.deliveries.last().is_some_and(|d| d.delivered);
    let doable = plan.outputs.len() - blocked;
    let dest = format!("Livrables/{}/{}", oracle_brand::project::safe_name(&p.data.brand_name), plan.folder);
    let reuses_draft = !delivered_last && p.data.deliveries.iter().any(|d| d.folder == plan.folder);
    ui.horizontal(|ui| {
        let label = format!("Exporter {doable} fichier{}", if doable > 1 { "s" } else { "" });
        if ui.add_enabled(doable > 0, egui::Button::new(RichText::new(label).color(t.on_accent)).fill(t.accent).min_size(egui::vec2(170.0, 34.0))).clicked() {
            start(app, ui.ctx(), plan.clone(), None);
        }
        let failed: Vec<String> = app.last_results.iter().filter(|r| r.error.is_some() && plan.outputs.iter().any(|o| o.path == r.path && o.blocked.is_none())).map(|r| r.path.clone()).collect();
        if !failed.is_empty() && ui.button(format!("Relancer les {} échec(s)", failed.len())).clicked() {
            start(app, ui.ctx(), plan.clone(), Some(failed));
        }
        if let Some(f) = app.last_folder.clone()
            && ui.button("Ouvrir le dossier exporté").clicked()
        {
            open_path(&f);
        }
        ui.label(RichText::new(format!("vers {dest}")).color(t.muted));
    });
    if delivered_last {
        ui.label(RichText::new("La livraison précédente a été envoyée au client : elle ne sera pas modifiée. Cet export va dans un nouveau dossier.").color(t.muted).size(12.0));
        if let Some((folder, n)) = &app.plan_carry {
            ui.label(RichText::new(format!("+ {n} fichier(s) inchangé(s) repris de {folder} à l'identique : le nouveau dossier est complet.")).color(t.muted).size(12.0));
        }
    } else if reuses_draft {
        ui.label(RichText::new("Ce dossier n'a pas encore été livré : les fichiers du même nom seront remplacés, les autres conservés.").color(t.muted).size(12.0));
    }
    if blocked > 0 {
        ui.label(RichText::new(format!("{blocked} fichier(s) ne peuvent pas être faits (voir la liste).")).color(t.warn));
    }
    if !app.last_results.is_empty() {
        ui.add_space(10.0);
        let ok = app.last_results.iter().filter(|r| r.error.is_none()).count();
        theme::label(ui, &format!("Dernier export : {ok} réussi(s), {} échec(s)", app.last_results.len() - ok));
        for r in app.last_results.iter().filter(|r| r.error.is_some()) {
            ui.label(RichText::new(format!("{} — {}", r.path, r.error.as_deref().unwrap_or(""))).color(t.bad).size(12.5));
        }
    }
    ui.add_space(8.0);
    card(ui, |ui| {
        theme::label(ui, &format!("{} fichier(s) prévus", plan.outputs.len()));
        egui::ScrollArea::vertical().max_height(320.0).id_salt("plan").show(ui, |ui| {
            for o in &plan.outputs {
                match &o.blocked {
                    None => ui.label(RichText::new(&o.path).size(12.5)),
                    Some(b) => ui.label(RichText::new(format!("{} — {b}", o.path)).color(t.bad).size(12.5)),
                };
            }
        });
    });
}

/// Selection lists: empty means "all"; `[""]` means none.
fn toggle(list: &mut Vec<String>, id: &str, on: bool, all: impl Iterator<Item = String>) {
    if list.is_empty() {
        *list = all.collect();
    }
    list.retain(|x| !x.is_empty() && x != id);
    if on {
        list.push(id.to_string());
    }
    if list.is_empty() {
        list.push(String::new());
    }
}

fn start(app: &mut App, ctx: &egui::Context, plan: export::Plan, only: Option<Vec<String>>) {
    let Some(project) = app.project.clone() else { return };
    // Save first: the worker writes the project's delivery record afterwards.
    if let Err(e) = project.save() {
        app.say(format!("Enregistrement impossible : {e}"), true);
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let c2 = cancel.clone();
    let engines = app.engines.clone();
    let ctx2 = ctx.clone();
    let total = only.as_ref().map(|o| o.len()).unwrap_or(plan.outputs.len());
    std::thread::spawn(move || {
        let mut project = project;
        let report = export::run(&mut project, &engines, &plan, only.as_deref(), &c2, |done, total, path| {
            let _ = tx.send(Progress::Step { done, total, path: path.to_string() });
            ctx2.request_repaint();
        });
        let _ = tx.send(Progress::Finished { project: Box::new(project), results: report.results, cancelled: report.cancelled, folder: report.folder });
        ctx2.request_repaint();
    });
    app.running = Some(Running { rx, cancel, done: 0, total, current: String::new() });
}

pub fn poll_export(app: &mut App, _ctx: &egui::Context) {
    let Some(run) = app.running.as_mut() else { return };
    let mut finished = None;
    while let Ok(msg) = run.rx.try_recv() {
        match msg {
            Progress::Step { done, total, path } => {
                run.done = done;
                run.total = total;
                run.current = path;
            }
            Progress::Finished { project, results, cancelled, folder } => finished = Some((project, results, cancelled, folder)),
        }
    }
    if let Some((project, results, cancelled, folder)) = finished {
        app.running = None;
        let ok = results.iter().filter(|r| r.error.is_none()).count();
        let failed = results.len() - ok;
        app.project = Some(*project);
        app.last_results = results;
        app.last_folder = Some(folder);
        app.plan = None;
        app.touch();
        let msg = if cancelled { format!("Export annulé : {ok} fichier(s) faits.") } else { format!("Export terminé : {ok} fichier(s), {failed} échec(s).") };
        app.say(msg, failed > 0);
    }
}

// ---------------------------------------------------------------- Livraisons

pub fn deliveries(app: &mut App, ui: &mut egui::Ui) {
    heading(ui, "Livraisons", "Chaque export crée ou complète un dossier. Une fois marqué « livré », un dossier n'est plus jamais réécrit.");
    let Some(p) = app.project.as_mut() else { return };
    let t = theme::tokens(ui.ctx());
    let mut changed = false;
    if p.data.deliveries.is_empty() {
        ui.label(RichText::new("Aucune livraison pour l'instant : lancez un export.").color(t.muted));
        if ui.button("Aller à l'export").clicked() {
            app.tab = Tab::Export;
        }
        return;
    }
    let root = p.deliveries_root();
    for d in p.data.deliveries.iter_mut().rev() {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&d.folder).strong().size(16.0));
                let (txt, col) = if d.delivered { ("Livrée", t.good) } else { ("Brouillon", t.muted) };
                ui.label(RichText::new(txt).color(col));
                ui.label(RichText::new(format!("{} fichier(s) · {}", d.outputs, date_fr(&d.created_at))).color(t.muted).size(12.0));
            });
            ui.horizontal(|ui| {
                let folder = root.join(&d.folder);
                if ui.add_enabled(folder.is_dir(), egui::Button::new("Ouvrir le dossier")).clicked() {
                    open_path(&folder);
                }
                if !d.delivered {
                    if ui.button("Marquer comme livrée").on_hover_text("Le dossier ne sera plus modifié ; le prochain export ira dans un nouveau dossier").clicked() {
                        d.delivered = true;
                        changed = true;
                    }
                } else {
                    ui.label(RichText::new("Verrouillée : un nouvel export crée un nouveau dossier.").color(t.muted).size(12.0));
                }
            });
        });
        ui.add_space(8.0);
    }
    if changed {
        app.touch();
    }
}

/// `2026-10-22T12:03:09Z` → `22/10/2026 12:03 UTC`.
fn date_fr(rfc: &str) -> String {
    let b = rfc.as_bytes();
    if rfc.len() >= 16 && b[4] == b'-' && b[10] == b'T' {
        format!("{}/{}/{} {} UTC", &rfc[8..10], &rfc[5..7], &rfc[0..4], &rfc[11..16])
    } else {
        rfc.to_string()
    }
}

// ---------------------------------------------------------------- System

pub fn open_path(p: &std::path::Path) {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer").arg(p).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(p).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(p).spawn();
}

fn reveal(p: &std::path::Path) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer").arg(format!("/select,{}", p.display())).spawn();
    }
    #[cfg(not(windows))]
    if let Some(dir) = p.parent() {
        open_path(dir);
    }
}
