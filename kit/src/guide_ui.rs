//! Guide authoring uses the same project records as the existing identity and delivery pages.
use crate::App;
use oracle_brand::guide::{self, CHAPTERS, ContrastPair, ReviewState};

fn field(ui: &mut egui::Ui, title: &str, value: &mut String, multiline: bool) -> bool {
    ui.label(title);
    let edit = if multiline {
        egui::TextEdit::multiline(value).desired_rows(3)
    } else {
        egui::TextEdit::singleline(value)
    };
    ui.add(edit.desired_width(f32::INFINITY)).changed()
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let Some(project) = app.project.as_ref() else {
        return;
    };
    let report = guide::check(project);
    ui.heading("Brand guide");
    ui.label("Les logos, couleurs, polices et règles viennent des autres pages de ce projet. Une modification rend les validations précédentes obsolètes.");
    ui.add_space(10.0);
    ui.horizontal_wrapped(|ui| {
        ui.strong(format!(
            "{} / {} chapitres validés",
            report.approved_chapters, report.applicable_chapters
        ));
        ui.label(format!("{} point(s) à résoudre", report.blockers.len()));
    });
    let mut export = None;
    ui.horizontal_wrapped(|ui| {
        if ui.button("Exporter un brouillon").clicked() {
            export = Some(true);
        }
        if ui
            .add_enabled(
                report.complete(),
                egui::Button::new("Livrer le guide validé"),
            )
            .on_disabled_hover_text("Complète les décisions, droits et validations ci-dessous.")
            .clicked()
        {
            export = Some(false);
        }
        let root = project.dir.join("Livrables/Brand-guides");
        if ui
            .add_enabled(root.is_dir(), egui::Button::new("Voir les guides exportés"))
            .clicked()
            && !cfg!(test)
        {
            crate::pages::open_path(&root);
        }
    });
    ui.small("Livraison : guide HTML hors ligne, fiche de référence, ressources autorisées, contrôle et manifeste. Aucun fichier de police n'est copié. Chaque export crée un nouveau dossier.");
    egui::CollapsingHeader::new(format!(
        "Contrôle avant livraison · {} blocage(s)",
        report.blockers.len()
    ))
    .show(ui, |ui| {
        for issue in &report.blockers {
            ui.label(format!("• {} : {}", issue.code, issue.message));
        }
        for issue in &report.warnings {
            ui.small(&issue.message);
        }
    });
    ui.separator();
    let Some(project) = app.project.as_mut() else {
        return;
    };
    let mut changed = false;
    egui::CollapsingHeader::new("Périmètre et modules").default_open(true).show(ui, |ui| {
        changed |= field(ui, "Destinataires et supports concernés", &mut project.data.guide.audience, false);
        changed |= field(ui, "Responsable et contact", &mut project.data.guide.owner, false);
        changed |= field(ui, "Personne qui valide", &mut project.data.guide.reviewer, false);
        changed |= ui.checkbox(&mut project.data.guide.print_required, "L'impression fait partie de cette livraison (CMJN et profil requis)").changed();
        ui.small("Les modules décochés sont hors périmètre. Cocher un module ajoute ses décisions au contrôle.");
        ui.horizontal_wrapped(|ui| {
            for spec in CHAPTERS.iter().filter(|s| s.optional) {
                let mut enabled = project.data.guide.modules.iter().any(|id| id == spec.id);
                if ui.checkbox(&mut enabled, spec.title).changed() {
                    changed = true;
                    if enabled { project.data.guide.modules.push(spec.id.into()); }
                    else { project.data.guide.modules.retain(|id| id != spec.id); }
                }
            }
        });
    });
    egui::CollapsingHeader::new("Droits des ressources à remettre").show(ui, |ui| {
        ui.label("Documente l'autorisation, puis confirme-la pour la version actuelle du fichier. Si le maître change, cette confirmation devra être renouvelée.");
        let reviewer = project.data.guide.reviewer.clone();
        for asset in project.data.assets.clone() {
            let valid = guide::authorized(project, &asset.id);
            let source = project.resolve(&asset.source);
            ui.push_id(&asset.id, |ui| {
                ui.strong(format!("{} · {}", asset.label, if valid { "autorisé" } else { "à confirmer" }));
                let before = project.data.guide.rights.get(&asset.id).cloned().unwrap_or_default();
                let mut rights = before.clone();
                changed |= field(ui, "Source / licence / accord de remise", &mut rights.evidence, false);
                if ui.add_enabled(!reviewer.trim().is_empty() && !rights.evidence.trim().is_empty(), egui::Button::new("Confirmer l'autorisation de remise")).clicked() {
                    match oracle_brand::project::hash_file(&source) {
                        Ok(hash) => {
                            rights.allow_delivery = true; rights.source_hash = hash;
                            rights.reviewer = reviewer.clone(); rights.reviewed_at = oracle_brand::now_rfc3339(); changed = true;
                        }
                        Err(e) => { ui.label(e); }
                    }
                }
                if rights.allow_delivery && ui.small_button("Révoquer l'autorisation").clicked() { rights.allow_delivery = false; changed = true; }
                if rights != before { project.data.guide.rights.insert(asset.id.clone(), rights); }
                ui.separator();
            });
        }
    });
    egui::CollapsingHeader::new("Associations de couleurs et contraste").show(ui, |ui| {
        ui.label("Décris des usages réels. Les seuils sont calculés en sRGB opaque : 4,5 pour le texte courant, 3 pour le grand texte ou un élément graphique significatif.");
        let colors = &project.data.colors;
        let pairs = &mut project.data.guide.contrast_pairs;
        let mut remove = None;
        for (i, pair) in pairs.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                changed |= field(ui, "Contexte d'utilisation", &mut pair.context, false);
                for (key, title, id) in [("fg", "Premier plan", &mut pair.foreground_id), ("bg", "Fond", &mut pair.background_id)] {
                    egui::ComboBox::from_id_salt(key).selected_text(format!("{title} : {}", colors.iter().find(|c| c.id == *id).map(|c| c.name.as_str()).unwrap_or("Choisir"))).show_ui(ui, |ui| {
                        for color in colors { changed |= ui.selectable_value(id, color.id.clone(), &color.name).changed(); }
                    });
                }
                changed |= ui.checkbox(&mut pair.large_text_or_graphic, "Grand texte (≥ 18 pt, ou ≥ 14 pt gras) / élément graphique").changed();
                let a = colors.iter().find(|c| c.id == pair.foreground_id);
                let b = colors.iter().find(|c| c.id == pair.background_id);
                if let Some(ratio) = a.zip(b).and_then(|(a,b)| guide::contrast(&a.hex, &b.hex)) {
                    ui.label(format!("{ratio:.3}:1 · {}", if ratio >= if pair.large_text_or_graphic { 3.0 } else { 4.5 } { "seuil atteint" } else { "contraste insuffisant" }));
                }
                if ui.small_button("Supprimer cette association").clicked() { remove = Some(i); }
                ui.separator();
            });
        }
        if let Some(i) = remove { pairs.remove(i); changed = true; }
        if ui.button("Ajouter une association de couleurs").clicked() { pairs.push(ContrastPair::default()); changed = true; }
    });
    ui.separator();
    ui.strong("Décisions du guide");
    ui.small("Renseigne tous les chapitres, puis valide-les après la dernière modification. Oracle vérifie leur présence et leur cohérence ; la qualité des décisions reste à relire.");
    let mut approval = None;
    let fingerprint = guide::fingerprint(&project.data);
    for spec in CHAPTERS
        .iter()
        .filter(|s| !s.optional || project.data.guide.modules.iter().any(|id| id == s.id))
        .collect::<Vec<_>>()
    {
        let chapter = project.data.guide.chapters.get(spec.id);
        let status = chapter
            .map(|c| {
                if matches!(c.state, ReviewState::Validated | ReviewState::NotApplicable)
                    && c.approved_fingerprint != fingerprint
                {
                    "À revalider"
                } else {
                    c.state.label()
                }
            })
            .unwrap_or("Manquant");
        let before = chapter.cloned().unwrap_or_default();
        let mut edited = before.clone();
        egui::CollapsingHeader::new(format!("{} · {} · {status}", spec.id, spec.title))
            .id_salt(spec.id)
            .show(ui, |ui| {
                let chapter = &mut edited;
                for title in spec.fields {
                    let mut text = chapter.fields.get(*title).cloned().unwrap_or_default();
                    if field(ui, title, &mut text, true) {
                        chapter.fields.insert((*title).into(), text);
                        chapter.state = ReviewState::Draft;
                        changed = true;
                    }
                }
                ui.label("Ressources de la bibliothèque de marque liées à ce chapitre");
                for asset in &project.data.assets {
                    let mut linked = chapter.asset_ids.contains(&asset.id);
                    if ui.checkbox(&mut linked, &asset.label).changed() {
                        changed = true;
                        chapter.state = ReviewState::Draft;
                        if linked {
                            chapter.asset_ids.push(asset.id.clone());
                        } else {
                            chapter.asset_ids.retain(|id| id != &asset.id);
                        }
                    }
                }
                if ui.button("Valider ce chapitre").clicked() {
                    approval = Some((spec.id, false));
                }
                if spec.optional {
                    changed |= field(
                        ui,
                        "Justification si non applicable",
                        &mut chapter.reason,
                        false,
                    );
                    if ui
                        .button("Déclarer non applicable avec justification")
                        .clicked()
                    {
                        approval = Some((spec.id, true));
                    }
                }
                if !chapter.reviewer.is_empty() {
                    ui.small(format!(
                        "Dernière validation : {} · {}",
                        chapter.reviewer, chapter.reviewed_at
                    ));
                }
            });
        if edited != before {
            project.data.guide.chapters.insert(spec.id.into(), edited);
        }
    }
    if changed {
        app.touch();
    }
    if let Some((id, na)) = approval {
        if let Some(p) = &mut app.project {
            match guide::approve(&mut p.data, id, na) {
                Ok(()) => {
                    app.touch();
                    app.say("Validation enregistrée. Toute modification du contenu imposera une nouvelle validation.", false);
                }
                Err(e) => app.say(e, true),
            }
        }
    }
    if let Some(draft) = export {
        if let Some(p) = app.project.as_ref() {
            let result = p
                .save()
                .and_then(|()| guide::export(p, &app.engines, draft));
            match result {
                Ok(folder) => {
                    let index_result = if let Some(p) = app.project.as_mut() {
                        let path = p.store_path(&folder.join("guide.html"));
                        p.data.guide_documents.push(path);
                        p.save()
                    } else {
                        Ok(())
                    };
                    match index_result {
                        Ok(()) => app.say(format!("Guide créé : {}", folder.display()), false),
                        Err(e) => app.say(
                            format!(
                                "Guide créé : {}. Son lien n'a pas pu être enregistré : {e}",
                                folder.display()
                            ),
                            true,
                        ),
                    }
                    if !cfg!(test) {
                        crate::pages::open_path(&folder.join("guide.html"));
                    }
                }
                Err(e) => app.say(e, true),
            }
        }
    }
}
