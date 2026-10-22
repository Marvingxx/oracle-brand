//! The left column: the brand's scenes (with what is out of date, and batch rendering) and the
//! templates.

use std::path::PathBuf;

use egui::{Color32, RichText};
use oracle_brand::mockup::{self, SceneState};

use crate::{App, Mode, theme};

pub fn sidebar(app: &mut App, ui: &mut egui::Ui) {
    let t = theme::tokens(ui.ctx());
    let Some(project) = &app.project else { return };
    let scenes = mockup::scenes(project);
    let open_file = match &app.mode {
        Mode::Scene { file, .. } => Some(file.clone()),
        _ => None,
    };

    theme::label(ui, &format!("Scènes · {}", scenes.len()));
    let mut open = None;
    egui::ScrollArea::vertical().id_salt("scenes").max_height(ui.available_height() * 0.5).auto_shrink([false, true]).show(ui, |ui| {
        if scenes.is_empty() {
            ui.label(RichText::new("Aucune scène. Ouvrez un modèle ci-dessous puis « Nouvelle scène ».").color(t.muted).size(12.5));
        }
        for (_, file, scene) in &scenes {
            let state = app.states.get(file).cloned();
            let (dot, hint) = match &state {
                Some(SceneState::UpToDate) => (t.good, "À jour".to_string()),
                Some(SceneState::NeverRendered) => (t.muted, "Jamais rendue".to_string()),
                Some(s @ SceneState::Outdated(_)) => (t.warn, s.label_fr()),
                Some(s @ SceneState::Broken(_)) => (t.bad, s.label_fr()),
                None => (t.muted, String::new()),
            };
            let name = match scene {
                Ok(s) => s.name.clone(),
                Err(_) => file.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            };
            ui.horizontal(|ui| {
                let mut on = app.checked.contains(file);
                if ui.checkbox(&mut on, "").on_hover_text("Inclure dans le rendu par lot").changed() {
                    if on {
                        app.checked.push(file.clone());
                    } else {
                        app.checked.retain(|f| f != file);
                    }
                }
                let (r, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 4.0, dot);
                let selected = open_file.as_ref() == Some(file);
                let resp = ui.add(egui::Button::selectable(selected, RichText::new(&name).color(if selected { Color32::WHITE } else { t.ink })).min_size(egui::vec2(ui.available_width(), 28.0)));
                let resp = if hint.is_empty() { resp } else { resp.on_hover_text(hint) };
                if resp.clicked() {
                    open = Some(file.clone());
                }
            });
        }
    });
    if let Some(f) = open {
        app.open_scene(&f);
    }

    let outdated: Vec<PathBuf> = app.states.iter().filter(|(_, s)| matches!(s, SceneState::Outdated(_) | SceneState::NeverRendered)).map(|(f, _)| f.clone()).collect();
    let checked: Vec<PathBuf> = app.checked.clone();
    ui.add_space(4.0);
    ui.add_enabled_ui(app.rendering.is_none(), |ui| {
        if ui.add_enabled(!checked.is_empty(), egui::Button::new(format!("Rendre la sélection ({})", checked.len()))).clicked() {
            app.render(checked);
        }
        if ui.add_enabled(!outdated.is_empty(), egui::Button::new(format!("Mettre à jour les scènes ({})", outdated.len()))).on_hover_text("Rend les scènes jamais rendues ou dont un logo, un masque ou un réglage a changé").clicked() {
            let mut files = outdated;
            files.sort();
            app.render(files);
        }
    });

    ui.add_space(14.0);
    let r = ui.cursor();
    ui.painter().hline(r.x_range(), r.top(), egui::Stroke::new(1.0, t.line));
    ui.add_space(12.0);
    theme::label(ui, &format!("Modèles · {}", app.templates.len()));
    let open_template = match &app.mode {
        Mode::Template { file, .. } | Mode::Scene { template_file: file, .. } => Some(file.clone()),
        Mode::None => None,
    };
    let mut pick = None;
    egui::ScrollArea::vertical().id_salt("templates").max_height(ui.available_height() - 110.0).auto_shrink([false, true]).show(ui, |ui| {
        for (file, tpl) in &app.templates {
            let selected = open_template.as_ref() == Some(file) && matches!(app.mode, Mode::Template { .. });
            let label = if tpl.category.is_empty() { tpl.name.clone() } else { format!("{}  ·  {}", tpl.name, tpl.category) };
            let resp = ui.add(egui::Button::selectable(selected, RichText::new(label).color(if selected { Color32::WHITE } else { t.ink })).min_size(egui::vec2(ui.available_width(), 28.0)));
            if resp.on_hover_text(format!("{} zone(s) · {}", tpl.zones.len(), file.display())).clicked() {
                pick = Some(file.clone());
            }
        }
        if app.templates.is_empty() {
            ui.label(RichText::new("Aucun modèle. Déposez une photo de support sur la zone centrale.").color(t.muted).size(12.5));
        }
    });
    if let Some(f) = pick {
        app.open_template(&f, None);
    }
    ui.add_space(6.0);
    if ui.add(egui::Button::new("Nouveau modèle depuis une photo…").min_size(egui::vec2(ui.available_width(), 30.0))).clicked()
        && let Some(photo) = rfd::FileDialog::new().set_title("Photo du support").add_filter("Photos", &["png", "jpg", "jpeg", "webp", "tif", "tiff", "psd"]).pick_file()
    {
        app.new_template(&photo);
    }
    ui.horizontal(|ui| {
        if ui.small_button("Dossier…").on_hover_text(app.templates_root().display().to_string()).clicked()
            && let Some(dir) = rfd::FileDialog::new().set_title("Dossier des modèles de mockups").pick_folder()
        {
            app.settings.templates_root = dir.display().to_string();
            app.save_settings();
            app.reload_templates();
        }
        if ui.small_button("Actualiser").clicked() {
            app.reload_templates();
            app.refresh_states();
        }
    });
}
