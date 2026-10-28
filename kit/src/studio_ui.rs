//! Independent tools sharing the brand project, export engine and delivery permissions.
use crate::{App, Tab};
use oracle_brand::{
    guide,
    studio::{self, Bento, Concept, Decision, Tool},
};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum Page {
    #[default]
    Grid,
    Presentations,
    Bento,
    Exports,
    Hub,
}
impl Page {
    fn tool(self) -> Option<Tool> {
        match self {
            Self::Grid => Some(Tool::Grid),
            Self::Presentations => Some(Tool::Presentations),
            Self::Bento => Some(Tool::Bento),
            _ => None,
        }
    }
}
#[derive(Default)]
pub struct State {
    page: Page,
    concept: usize,
    preview: Option<egui::TextureHandle>,
    pub last_folder: Option<std::path::PathBuf>,
}
impl State {
    pub fn invalidate(&mut self) {
        self.preview = None;
    }
}

fn asset_picker(
    ui: &mut egui::Ui,
    id: &str,
    selected: &mut String,
    assets: &[(String, String)],
) -> bool {
    let before = selected.clone();
    let label = assets
        .iter()
        .find(|(id, _)| id == selected)
        .map(|(_, name)| name.as_str())
        .unwrap_or("Choisir un logo du projet");
    egui::ComboBox::from_id_salt(id)
        .selected_text(label)
        .show_ui(ui, |ui| {
            for (id, label) in assets {
                ui.selectable_value(selected, id.clone(), label);
            }
        });
    *selected != before
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    if app.project.is_none() {
        return;
    }
    ui.heading("Studio de marque");
    ui.label("Une identité partagée, des compositions enregistrées dans ce projet, des livraisons distinctes.");
    let old_page = app.studio_ui.page;
    ui.horizontal_wrapped(|ui| {
        for (page, label) in [
            (Page::Grid, "Grilles"),
            (Page::Presentations, "Présentations"),
            (Page::Bento, "Bento"),
            (Page::Exports, "Exports de logos"),
            (Page::Hub, "Espace client"),
        ] {
            ui.selectable_value(&mut app.studio_ui.page, page, label);
        }
    });
    if app.studio_ui.page != old_page {
        app.studio_ui.preview = None;
    }
    ui.separator();
    let page = app.studio_ui.page;
    if page == Page::Exports {
        ui.heading("Exports de logos");
        ui.label("Retrouve les variantes, formats et dimensions du moteur d'export existant. Les presets et les livraisons restent ceux du Brand Kit.");
        ui.horizontal(|ui| {
            if ui.button("Configurer les presets").clicked() {
                app.tab = Tab::Presets;
            }
            if ui.button("Préparer les fichiers").clicked() {
                app.tab = Tab::Export;
            }
            if ui.button("Historique des livraisons").clicked() {
                app.tab = Tab::Deliveries;
            }
        });
        ui.small("Les maîtres restent inchangés. Les formats disponibles dépendent des convertisseurs détectés ; aucune conversion CMJN implicite.");
        return;
    }
    if page == Page::Hub {
        ui.heading("Espace client hors ligne");
        ui.label("Une page d'accueil avec le guide, la fiche de référence et les ressources autorisées à télécharger. Chaque export crée une nouvelle révision.");
        ui.small("Ce dossier peut être partagé manuellement. Aucun hébergement, compte client, lien actualisé automatiquement ou synchronisation en ligne n'est fourni.");
        let report = guide::check(app.project.as_ref().unwrap());
        let mut action = None;
        ui.horizontal_wrapped(|ui| {
            if ui.button("Créer un espace client brouillon").clicked() {
                action = Some(true);
            }
            if ui
                .add_enabled(
                    report.complete(),
                    egui::Button::new("Livrer l'espace client validé"),
                )
                .on_disabled_hover_text(
                    "Résoudre les contrôles du Brand guide avant la livraison finale.",
                )
                .clicked()
            {
                action = Some(false);
            }
            if ui.button("Compléter le Brand guide").clicked() {
                app.tab = Tab::Guide;
            }
        });
        ui.label(format!(
            "{} contrôle(s) à résoudre avant livraison finale",
            report.blockers.len()
        ));
        if let Some(draft) = action {
            let result = (|| {
                let p = app.project.as_ref().unwrap();
                p.save()?;
                guide::export_hub(p, &app.engines, draft)
            })();
            finish(app, result);
        }
        last_export(app, ui);
        return;
    }
    let mut changed = false;
    let project = app.project.as_mut().unwrap();
    let assets: Vec<_> = project
        .data
        .assets
        .iter()
        .map(|a| (a.id.clone(), format!("{} · {}", a.label, a.id)))
        .collect();
    let s = &mut project.data.studio;
    match page {
        Page::Grid => {
            ui.label("Grille de présentation et zone de protection autour du cadre du fichier, marges incluses.");
            changed |= asset_picker(ui, "grid-logo", &mut s.asset_id, &assets);
            changed |= ui
                .add(egui::Slider::new(&mut s.grid_unit, 10..=80).text("Taille d'une unité"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut s.protection_units, 0..=2).text("Protection en unités"))
                .changed();
            ui.small("Les axes et la grille ne déduisent ni les courbes ni les proportions de construction du logo.");
        }
        Page::Bento => {
            changed |= asset_picker(ui, "bento-logo", &mut s.asset_id, &assets);
            ui.horizontal(|ui| {
                changed |= ui
                    .selectable_value(&mut s.bento, Bento::Editorial, "Éditorial")
                    .changed();
                changed |= ui
                    .selectable_value(&mut s.bento, Bento::Mosaic, "Mosaïque")
                    .changed();
            });
            ui.label("Phrase de présentation");
            changed |= ui
                .add(egui::TextEdit::singleline(&mut s.headline).desired_width(f32::INFINITY))
                .changed();
            ui.small("Reprend jusqu'à 8 couleurs et 3 références typographiques du projet. Les noms de polices sont affichés en Arial ; ce n'est pas un spécimen composé avec tes polices.");
        }
        Page::Presentations => {
            ui.label("Associe chaque piste à un logo du projet, explique ton intention et conserve ta décision interne.");
            ui.small("L'export inclut toutes les pistes, y compris celles écartées, et leurs commentaires. Relis-les avant de transmettre le dossier.");
            if ui
                .add_enabled(
                    s.concepts.len() < 50,
                    egui::Button::new("Ajouter une piste"),
                )
                .clicked()
            {
                s.concepts.push(Concept::new(s.asset_id.clone()));
                app.studio_ui.concept = s.concepts.len() - 1;
                changed = true;
            }
            if !s.concepts.is_empty() {
                app.studio_ui.concept = app.studio_ui.concept.min(s.concepts.len() - 1);
                let mut selected = app.studio_ui.concept;
                ui.horizontal_wrapped(|ui| {
                    for (i, c) in s.concepts.iter().enumerate() {
                        ui.selectable_value(&mut selected, i, format!("{} · {}", i + 1, c.title));
                    }
                });
                if selected != app.studio_ui.concept {
                    app.studio_ui.concept = selected;
                    app.studio_ui.preview = None;
                }
                let i = app.studio_ui.concept;
                let c = &mut s.concepts[i];
                ui.label("Nom de la piste");
                changed |= ui.text_edit_singleline(&mut c.title).changed();
                changed |= asset_picker(ui, "concept-logo", &mut c.asset_id, &assets);
                ui.label("Intention et argumentaire");
                changed |= ui
                    .add(
                        egui::TextEdit::multiline(&mut c.rationale)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY),
                    )
                    .changed();
                ui.horizontal_wrapped(|ui| {
                    for decision in [Decision::Proposed, Decision::Retained, Decision::Rejected] {
                        changed |= ui
                            .selectable_value(&mut c.decision, decision, decision.label())
                            .changed();
                    }
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(i > 0, egui::Button::new("Monter la piste"))
                        .clicked()
                    {
                        s.concepts.swap(i, i - 1);
                        app.studio_ui.concept -= 1;
                        changed = true;
                    }
                    if ui
                        .add_enabled(
                            i + 1 < s.concepts.len(),
                            egui::Button::new("Descendre la piste"),
                        )
                        .clicked()
                    {
                        s.concepts.swap(i, i + 1);
                        app.studio_ui.concept += 1;
                        changed = true;
                    }
                    if ui.button("Supprimer la piste").clicked() {
                        s.concepts.remove(i);
                        app.studio_ui.concept = i.saturating_sub(1);
                        changed = true;
                    }
                });
            }
        }
        _ => {}
    }
    if changed {
        app.studio_ui.preview = None;
        app.touch();
    }
    ui.add_space(8.0);
    let mut preview = false;
    let mut export = false;
    ui.horizontal_wrapped(|ui| {
        preview = ui.button("Actualiser l'aperçu").clicked();
        export = ui.button("Exporter les planches").clicked();
    });
    ui.small("SVG de composition + PNG 1200 × 800 + page HTML et recette. Les logos sont incorporés en aperçus PNG, sans fichier de police. L'export exige des droits de remise à jour dans Brand guide.");
    let tool = page.tool().unwrap();
    if preview {
        let result = studio::document(
            app.project.as_ref().unwrap(),
            &app.engines,
            tool,
            app.studio_ui.concept,
        )
        .and_then(|svg| studio::raster(&svg));
        match result {
            Ok(pixels) => {
                let color = egui::ColorImage::from_rgba_unmultiplied(
                    [pixels.width() as usize, pixels.height() as usize],
                    pixels.as_raw(),
                );
                app.studio_ui.preview = Some(ui.ctx().load_texture(
                    "studio-preview",
                    color,
                    egui::TextureOptions::LINEAR,
                ));
            }
            Err(e) => app.say(e, true),
        }
    }
    if export {
        let result = (|| {
            let p = app.project.as_ref().unwrap();
            p.save()?;
            studio::export(p, &app.engines, tool)
        })();
        finish(app, result);
    }
    last_export(app, ui);
    if let Some(texture) = &app.studio_ui.preview {
        ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(
            ui.available_width().min(900.0),
            ui.available_width().min(900.0) * 2.0 / 3.0,
        )));
    }
}

fn finish(app: &mut App, result: Result<std::path::PathBuf, String>) {
    match result {
        Ok(folder) => {
            app.say(format!("Export créé : {}", folder.display()), false);
            app.studio_ui.last_folder = Some(folder);
        }
        Err(e) => app.say(e, true),
    }
}
fn last_export(app: &mut App, ui: &mut egui::Ui) {
    if let Some(folder) = &app.studio_ui.last_folder {
        if ui.button("Ouvrir le dernier export Studio").clicked() && !cfg!(test) {
            crate::pages::open_path(&folder.join("index.html"));
        }
        ui.small(folder.display().to_string());
    }
}
