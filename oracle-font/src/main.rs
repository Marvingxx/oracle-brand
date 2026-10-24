//! Oracle Font: every font at hand (the system's, the user's, the library's and the brands' from
//! Brand Kit), tried out before use: a sample line in each, then for the chosen family a waterfall of
//! sizes, a paragraph, its glyphs, a side-by-side comparison, a heading-and-body pairing, the text
//! on a brand's colours, and its variation axes. A font can be activated for the user (no
//! administrator rights), added to the library, or given to a brand of Brand Kit.

#![windows_subsystem = "windows"]

mod activation;
mod catalog;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use catalog::{Face, Family, Source};
use egui::{Color32, FontFamily, FontId, RichText, vec2};
use oracle_style as theme;

fn main() -> eframe::Result<()> {
    let icon = image::load_from_memory(include_bytes!("../assets/hicolor/256x256/apps/app.oracle.oracle-font.png")).ok().map(|i| {
        let i = i.to_rgba8();
        egui::IconData { width: i.width(), height: i.height(), rgba: i.into_raw() }
    });
    let mut viewport = egui::ViewportBuilder::default().with_title("Oracle Font").with_inner_size([1440.0, 900.0]).with_min_inner_size([980.0, 600.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    eframe::run_native("Oracle Font", eframe::NativeOptions { viewport, ..Default::default() }, Box::new(|_cc| Ok(Box::new(App::new()))))
}

/// The sample text a new window starts with.
const PANGRAM: &str = "Portez ce vieux whisky au juge blond qui fume";
const PARAGRAPH: &str = "La typographie est l'art de donner forme au texte. Une bonne police se fait oublier : elle porte la voix de la marque sans jamais la couvrir. Chiffres 0123456789, ponctuation « ! ? ; : », accents àâéèêëîïôöùûüç et ligatures fi fl ff.";
/// Characters the glyph view tries, in order.
const GLYPHS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789àâäéèêëîïôöùûüçœæÀÉÈÇŒÆ.,;:!?¿¡«»“”‘’'\"()[]{}<>/\\|@#$%&*+-=_~^€£¥¢§©®™°±×÷µ¶•…–—ÅØßÐÞΩπΣЖЯ中あ한";

/// The faces loaded into egui, and in what order (the oldest go first past the limit: egui keeps
/// fonts for good, so the count is capped).
const MAX_LOADED: usize = 400;

/// Settings kept between runs.
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Settings {
    #[serde(default)]
    theme: theme::ThemeChoice,
    #[serde(default)]
    favorites: Vec<String>,
}

fn settings_path() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    dirs::config_dir().map(|d| d.join("Oracle").join("font.json"))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Specimen,
    Paragraph,
    Glyphs,
    Compare,
    Pairing,
    Brand,
    Info,
}

impl Tab {
    const ALL: [Tab; 7] = [Tab::Specimen, Tab::Paragraph, Tab::Glyphs, Tab::Compare, Tab::Pairing, Tab::Brand, Tab::Info];
    fn label(self) -> &'static str {
        match self {
            Tab::Specimen => "Cascade",
            Tab::Paragraph => "Paragraphe",
            Tab::Glyphs => "Glyphes",
            Tab::Compare => "Comparer",
            Tab::Pairing => "Paires",
            Tab::Brand => "Sur une marque",
            Tab::Info => "Infos",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum Filter {
    All,
    Favorites,
    Variable,
    Monospaced,
    From(Source),
}

struct App {
    settings: Settings,
    applied_theme: Option<theme::ThemeChoice>,
    fonts_installed: bool,
    scanning: Option<Receiver<Vec<Family>>>,
    families: Vec<Family>,
    filter: Filter,
    search: String,
    sample: String,
    size: f32,
    selected: Option<String>,
    face: usize,
    tab: Tab,
    /// Axis values set for the selected face, by tag.
    axes: HashMap<[u8; 4], f32>,
    /// Families shown next to the selected one in Compare.
    compare: Vec<String>,
    /// The body font of a pairing (the selected family is the heading's).
    body: Option<String>,
    brand: usize,
    /// Faces given to egui, with the pass they were given in: a font is usable from the next pass.
    loaded: HashMap<String, u64>,
    status: Option<String>,
}

impl App {
    fn new() -> App {
        let settings = settings_path().and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        let mut app = App {
            settings,
            applied_theme: None,
            fonts_installed: false,
            scanning: None,
            families: Vec::new(),
            filter: Filter::All,
            search: String::new(),
            sample: PANGRAM.into(),
            size: 30.0,
            selected: None,
            face: 0,
            tab: Tab::Specimen,
            axes: HashMap::new(),
            compare: Vec::new(),
            body: None,
            brand: 0,
            loaded: HashMap::new(),
            status: None,
        };
        app.rescan();
        app
    }

    fn rescan(&mut self) {
        let (tx, rx) = channel();
        let library = oracle_library::UserData::default_path().map(|p| oracle_library::UserData::load(&p)).and_then(|u| u.library_root());
        let _ = std::thread::Builder::new().name("font-scan".into()).spawn(move || {
            let _ = tx.send(catalog::scan(library.as_deref()));
        });
        self.scanning = Some(rx);
    }

    fn save_settings(&self) {
        let Some(path) = settings_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_vec_pretty(&self.settings) {
            // Best effort: a setting not kept is only asked again.
            let _ = std::fs::write(path, json);
        }
    }

    fn family(&self, name: &str) -> Option<&Family> {
        self.families.iter().find(|f| f.name == name)
    }

    /// The egui font family drawing `face` (with these axis values), loaded on first use. `None`
    /// past the load limit or for a file that can't be read.
    fn font(&mut self, ctx: &egui::Context, face: &Face, axes: &[([u8; 4], f32)]) -> Option<FontFamily> {
        let mut key = face.key();
        for (t, v) in axes {
            key.push_str(&format!("|{}={v:.0}", String::from_utf8_lossy(t)));
        }
        let pass = ctx.cumulative_pass_nr();
        if let Some(given) = self.loaded.get(&key) {
            if *given >= pass {
                // Given this pass: egui uses it from the next one.
                ctx.request_repaint();
                return None;
            }
        } else {
            if self.loaded.len() >= MAX_LOADED {
                return None;
            }
            let bytes = std::fs::read(&face.path).ok()?;
            let mut data = egui::FontData::from_owned(bytes);
            data.index = face.index;
            if !axes.is_empty() {
                data = data.tweak(egui::epaint::text::FontTweak { coords: egui::epaint::text::VariationCoords::new(axes.iter().copied()), ..Default::default() });
            }
            ctx.add_font(egui::epaint::text::FontInsert::new(
                &key,
                data,
                vec![egui::epaint::text::InsertFontFamily { family: FontFamily::Name(key.clone().into()), priority: egui::epaint::text::FontPriority::Highest }],
            ));
            self.loaded.insert(key, pass);
            ctx.request_repaint();
            return None;
        }
        Some(FontFamily::Name(key.into()))
    }

    fn visible(&self) -> Vec<usize> {
        let q = self.search.trim().to_lowercase();
        self.families
            .iter()
            .enumerate()
            .filter(|(_, f)| q.is_empty() || f.name.to_lowercase().contains(&q))
            .filter(|(_, f)| match &self.filter {
                Filter::All => true,
                Filter::Favorites => self.settings.favorites.contains(&f.name),
                Filter::Variable => f.is_variable(),
                Filter::Monospaced => f.is_monospaced(),
                Filter::From(s) => f.sources().contains(s),
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        let t = theme::tokens(ui.ctx());
        let mut sources: Vec<Source> = self.families.iter().flat_map(|f| f.sources()).collect();
        sources.sort();
        sources.dedup();
        let mut items: Vec<(Filter, String)> = vec![
            (Filter::All, "Toutes".into()),
            (Filter::Favorites, "★ Favorites".into()),
            (Filter::Variable, "Variables".into()),
            (Filter::Monospaced, "Chasse fixe".into()),
        ];
        items.extend(sources.into_iter().map(|s| {
            let l = s.label();
            (Filter::From(s), l)
        }));
        theme::label(ui, "Polices");
        for (f, label) in items {
            let on = self.filter == f;
            let text = RichText::new(label).color(if on { Color32::WHITE } else { t.ink });
            if ui.add(egui::Button::new(text).fill(if on { theme::RED } else { Color32::TRANSPARENT }).min_size(vec2(196.0, 30.0))).clicked() {
                self.filter = f;
            }
        }
        ui.add_space(12.0);
        if ui.button("Actualiser").on_hover_text("Relire les polices installées, la Bibliothèque et les marques").clicked() {
            self.rescan();
        }
    }

    fn list(&mut self, ui: &mut egui::Ui) {
        let t = theme::tokens(ui.ctx());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Rechercher une police").desired_width(240.0));
            ui.add(egui::TextEdit::singleline(&mut self.sample).hint_text("Texte d'essai").desired_width((ui.available_width() - 200.0).max(160.0)));
            ui.add(egui::Slider::new(&mut self.size, 12.0..=96.0).suffix(" pt").show_value(true));
        });
        ui.add_space(6.0);
        let rows = self.visible();
        ui.label(RichText::new(format!("{} familles", rows.len())).color(t.muted).size(12.0));
        let row_h = (self.size * 1.5 + 34.0).max(60.0);
        let ctx = ui.ctx().clone();
        egui::ScrollArea::vertical().id_salt("families").auto_shrink(false).show_rows(ui, row_h, rows.len(), |ui, range| {
            for i in range {
                let Some(&fi) = rows.get(i) else { continue };
                let Some(fam) = self.families.get(fi).cloned() else { continue };
                let selected = self.selected.as_deref() == Some(fam.name.as_str());
                let frame = egui::Frame::new().fill(if selected { t.sunken } else { t.bg }).stroke(egui::Stroke::new(1.0, t.line)).corner_radius(theme::RADIUS).inner_margin(10);
                let resp = frame
                    .show(ui, |ui| {
                        ui.set_min_height(row_h - 22.0);
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let fav = self.settings.favorites.contains(&fam.name);
                            if ui.small_button(if fav { "★" } else { "☆" }).on_hover_text("Favorite").clicked() {
                                if fav {
                                    self.settings.favorites.retain(|f| f != &fam.name);
                                } else {
                                    self.settings.favorites.push(fam.name.clone());
                                }
                                self.save_settings();
                            }
                            ui.label(RichText::new(&fam.name).strong());
                            let n = fam.faces.len();
                            let mut info = format!("{n} style{}", if n > 1 { "s" } else { "" });
                            if fam.is_variable() {
                                info.push_str(" · variable");
                            }
                            ui.label(RichText::new(info).color(t.muted).size(12.0));
                        });
                        let sample = if self.sample.trim().is_empty() { PANGRAM.to_string() } else { self.sample.clone() };
                        match fam.regular().and_then(|f| self.font(&ctx, &f.clone(), &[])) {
                            Some(family) => {
                                ui.add(egui::Label::new(RichText::new(sample).font(FontId::new(self.size, family))).truncate());
                            }
                            None => {
                                ui.label(RichText::new("…").color(t.muted).size(self.size));
                            }
                        }
                    })
                    .response
                    .interact(egui::Sense::click());
                if resp.clicked() {
                    self.select(&fam.name);
                }
                ui.add_space(6.0);
            }
        });
    }

    fn select(&mut self, name: &str) {
        self.selected = Some(name.to_string());
        self.face = self.family(name).and_then(|f| f.regular().and_then(|r| f.faces.iter().position(|x| x == r))).unwrap_or(0);
        self.axes.clear();
    }

    fn details(&mut self, ui: &mut egui::Ui) {
        let t = theme::tokens(ui.ctx());
        let Some(name) = self.selected.clone() else {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| ui.label(RichText::new("Choisissez une police pour l'essayer.").color(t.muted)));
            return;
        };
        let Some(fam) = self.family(&name).cloned() else { return };
        let Some(face) = fam.faces.get(self.face).cloned().or_else(|| fam.faces.first().cloned()) else { return };
        ui.horizontal(|ui| {
            ui.heading(&fam.name);
            egui::ComboBox::from_id_salt("face").selected_text(&face.style).show_ui(ui, |ui| {
                for (i, f) in fam.faces.iter().enumerate() {
                    if ui.selectable_label(i == self.face, &f.style).clicked() {
                        self.face = i;
                        self.axes.clear();
                    }
                }
            });
        });
        ui.horizontal_wrapped(|ui| {
            self.actions(ui, &face);
        });
        if !face.axes.is_empty() {
            ui.horizontal_wrapped(|ui| {
                for a in &face.axes {
                    let v = self.axes.entry(a.tag).or_insert(a.default);
                    ui.label(a.name());
                    ui.add(egui::Slider::new(v, a.min..=a.max).step_by(1.0));
                }
            });
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for tab in Tab::ALL {
                if ui.selectable_label(self.tab == tab, tab.label()).clicked() {
                    self.tab = tab;
                }
            }
        });
        ui.separator();
        let ctx = ui.ctx().clone();
        let axes: Vec<([u8; 4], f32)> = face.axes.iter().map(|a| (a.tag, self.axes.get(&a.tag).copied().unwrap_or(a.default))).filter(|(t, v)| face.axes.iter().any(|a| &a.tag == t && (a.default - v).abs() > 0.5)).collect();
        let Some(family) = self.font(&ctx, &face, &axes) else {
            ui.label("Chargement de la police…");
            return;
        };
        let sample = if self.sample.trim().is_empty() { PANGRAM.to_string() } else { self.sample.clone() };
        egui::ScrollArea::vertical().id_salt("details").auto_shrink(false).show(ui, |ui| match self.tab {
            Tab::Specimen => {
                for size in [12.0, 14.0, 18.0, 24.0, 32.0, 48.0, 64.0, 96.0] {
                    ui.horizontal(|ui| {
                        ui.add_sized(vec2(48.0, 20.0), egui::Label::new(RichText::new(format!("{size:.0}")).color(t.muted).size(11.0)));
                        ui.add(egui::Label::new(RichText::new(&sample).font(FontId::new(size, family.clone()))).truncate());
                    });
                }
            }
            Tab::Paragraph => {
                for (size, leading) in [(13.0, "corps 13"), (16.0, "corps 16"), (20.0, "corps 20")] {
                    ui.label(RichText::new(leading).color(t.muted).size(11.0));
                    ui.add(egui::Label::new(RichText::new(PARAGRAPH).font(FontId::new(size, family.clone()))).wrap());
                    ui.add_space(10.0);
                }
            }
            Tab::Glyphs => {
                let data = std::fs::read(&face.path).unwrap_or_default();
                let parsed = ttf_parser::Face::parse(&data, face.index).ok();
                ui.horizontal_wrapped(|ui| {
                    for c in GLYPHS.chars() {
                        let has = parsed.as_ref().is_some_and(|p| p.glyph_index(c).is_some());
                        let (r, _) = ui.allocate_exact_size(vec2(54.0, 62.0), egui::Sense::hover());
                        ui.painter().rect_stroke(r, theme::RADIUS, egui::Stroke::new(1.0, t.line), egui::StrokeKind::Inside);
                        if has {
                            ui.painter().text(r.center() - vec2(0.0, 6.0), egui::Align2::CENTER_CENTER, c, FontId::new(30.0, family.clone()), t.ink);
                        } else {
                            ui.painter().text(r.center() - vec2(0.0, 6.0), egui::Align2::CENTER_CENTER, "—", FontId::proportional(14.0), t.muted);
                        }
                        ui.painter().text(r.center_bottom() - vec2(0.0, 9.0), egui::Align2::CENTER_CENTER, format!("U+{:04X}", c as u32), FontId::proportional(8.5), t.muted);
                    }
                });
            }
            Tab::Compare => {
                ui.horizontal(|ui| {
                    ui.label("Ajouter :");
                    let names: Vec<String> = self.families.iter().map(|f| f.name.clone()).collect();
                    egui::ComboBox::from_id_salt("compare-add").selected_text("une police…").height(320.0).show_ui(ui, |ui| {
                        for n in names {
                            if ui.selectable_label(false, &n).clicked() && !self.compare.contains(&n) && self.compare.len() < 5 {
                                self.compare.push(n);
                            }
                        }
                    });
                    if !self.compare.is_empty() && ui.button("Vider").clicked() {
                        self.compare.clear();
                    }
                });
                ui.add_space(8.0);
                ui.label(RichText::new(&fam.name).color(t.muted).size(12.0));
                ui.add(egui::Label::new(RichText::new(&sample).font(FontId::new(self.size, family.clone()))).wrap());
                for other in self.compare.clone() {
                    let Some(f) = self.family(&other).and_then(|f| f.regular()).cloned() else { continue };
                    ui.add_space(8.0);
                    ui.label(RichText::new(&other).color(t.muted).size(12.0));
                    if let Some(of) = self.font(&ctx, &f, &[]) {
                        ui.add(egui::Label::new(RichText::new(&sample).font(FontId::new(self.size, of))).wrap());
                    }
                }
            }
            Tab::Pairing => {
                let names: Vec<String> = self.families.iter().map(|f| f.name.clone()).collect();
                ui.horizontal(|ui| {
                    ui.label("Texte courant :");
                    let shown = self.body.clone().unwrap_or_else(|| "même police".into());
                    egui::ComboBox::from_id_salt("pair-body").selected_text(shown).height(320.0).show_ui(ui, |ui| {
                        if ui.selectable_label(self.body.is_none(), "même police").clicked() {
                            self.body = None;
                        }
                        for n in names {
                            if ui.selectable_label(self.body.as_deref() == Some(n.as_str()), &n).clicked() {
                                self.body = Some(n);
                            }
                        }
                    });
                });
                ui.add_space(12.0);
                let body = self.body.clone().and_then(|b| self.family(&b).and_then(|f| f.regular()).cloned()).and_then(|f| self.font(&ctx, &f, &[])).unwrap_or(family.clone());
                ui.add(egui::Label::new(RichText::new(&sample).font(FontId::new(40.0, family.clone()))).wrap());
                ui.add_space(6.0);
                ui.add(egui::Label::new(RichText::new(PARAGRAPH).font(FontId::new(15.0, body))).wrap());
            }
            Tab::Brand => self.on_brand(ui, &family, &sample),
            Tab::Info => {
                egui::Grid::new("info").num_columns(2).spacing(vec2(14.0, 6.0)).show(ui, |ui| {
                    let rows: Vec<(&str, String)> = vec![
                        ("Famille", face.family.clone()),
                        ("Style", face.style.clone()),
                        ("Graisse", face.weight.to_string()),
                        ("Glyphes", face.glyphs.to_string()),
                        ("Écritures", face.scripts.join(", ")),
                        ("Axes", face.axes.iter().map(|a| format!("{} {:.0}–{:.0}", a.name(), a.min, a.max)).collect::<Vec<_>>().join(", ")),
                        ("Créateur", face.designer.clone()),
                        ("Version", face.version.clone()),
                        ("Licence", face.license.clone()),
                        ("Provenance", face.sources.iter().map(Source::label).collect::<Vec<_>>().join(", ")),
                        ("Fichier", face.path.display().to_string()),
                    ];
                    for (k, v) in rows {
                        ui.label(RichText::new(k).color(t.muted));
                        ui.add(egui::Label::new(if v.is_empty() { "—".to_string() } else { v }).wrap());
                        ui.end_row();
                    }
                });
            }
        });
    }

    /// The text on each colour of a brand, and each colour on the others' background.
    fn on_brand(&mut self, ui: &mut egui::Ui, family: &FontFamily, sample: &str) {
        let brands = oracle_suite_link::brands::brands();
        if brands.is_empty() {
            ui.label("Aucune marque : créez-en une dans Oracle Brand Kit.");
            return;
        }
        self.brand = self.brand.min(brands.len() - 1);
        ui.horizontal(|ui| {
            ui.label("Marque :");
            egui::ComboBox::from_id_salt("brand").selected_text(brands[self.brand].name.clone()).show_ui(ui, |ui| {
                for (i, b) in brands.iter().enumerate() {
                    if ui.selectable_label(i == self.brand, &b.name).clicked() {
                        self.brand = i;
                    }
                }
            });
        });
        let colors: Vec<Color32> = brands[self.brand].colors.iter().map(|c| Color32::from_rgb(c.rgb[0], c.rgb[1], c.rgb[2])).collect();
        let names: Vec<String> = brands[self.brand].colors.iter().map(|c| c.name.clone()).collect();
        let backgrounds = [vec![Color32::WHITE, Color32::from_gray(20)], colors.clone()].concat();
        for bg in backgrounds {
            for (fg, n) in colors.iter().zip(&names) {
                if *fg == bg {
                    continue;
                }
                egui::Frame::new().fill(bg).corner_radius(theme::RADIUS).inner_margin(12).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.add(egui::Label::new(RichText::new(sample).font(FontId::new(26.0, family.clone())).color(*fg)).truncate());
                    ui.label(RichText::new(format!("{n} · contraste {:.1}:1", contrast(*fg, bg))).size(11.0).color(if luminance(bg) > 0.4 { Color32::from_gray(60) } else { Color32::from_gray(200) }));
                });
                ui.add_space(4.0);
            }
        }
    }

    fn actions(&mut self, ui: &mut egui::Ui, face: &Face) {
        if activation::is_user_font(face) {
            if ui.button("Désactiver").on_hover_text("Retirer cette police des polices de l'utilisateur").clicked() {
                self.status = Some(match activation::deactivate(face) {
                    Ok(()) => format!("{} désactivée (les logiciels ouverts la gardent jusqu'à leur fermeture)", face.family),
                    Err(e) => e,
                });
                self.rescan();
            }
        } else if !face.sources.contains(&Source::System)
            && theme::primary(ui, "Activer").on_hover_text("Installer cette police pour l'utilisateur, sans droits administrateur").clicked()
        {
            self.status = Some(match activation::activate(face) {
                Ok(_) => format!("{} activée : les logiciels la voient à leur prochain démarrage", face.family),
                Err(e) => e,
            });
            self.rescan();
        }
        if !face.sources.contains(&Source::Library) && ui.button("Ajouter à la Bibliothèque").clicked() {
            let root = oracle_library::UserData::default_path().map(|p| oracle_library::UserData::load(&p)).and_then(|u| u.library_root());
            self.status = Some(match root {
                Some(root) => match oracle_library::ingest::add(&root, std::slice::from_ref(&face.path)) {
                    Ok(_) => format!("{} ajoutée à « {} »", face.family, oracle_library::ingest::OWN_FOLDER),
                    Err(e) => e,
                },
                None => "Choisissez d'abord le dossier de la Bibliothèque (Oracle Suite ou Oracle Assets)".into(),
            });
        }
        let brands = oracle_suite_link::brands::brands();
        if !brands.is_empty() {
            ui.menu_button("Utiliser dans une marque", |ui| {
                for b in &brands {
                    if ui.button(&b.name).clicked() {
                        self.status = Some(match add_to_brand(&b.dir, face) {
                            Ok(()) => format!("{} ajoutée aux typographies de {}", face.family, b.name),
                            Err(e) => e,
                        });
                        ui.close();
                    }
                }
            });
        }
        if ui.button("Afficher le fichier").clicked() {
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("explorer").raw_arg(format!("/select,\"{}\"", face.path.display())).spawn();
            }
        }
    }
}

/// Adds `face`'s family to the brand project in `dir` (its file kept as a local file), unless the
/// brand already has it.
fn add_to_brand(dir: &std::path::Path, face: &Face) -> Result<(), String> {
    let mut p = oracle_brand::Project::open(dir)?;
    let file = p.store_path(&face.path);
    if let Some(f) = p.data.fonts.iter_mut().find(|f| f.family.eq_ignore_ascii_case(&face.family)) {
        if !f.local_files.contains(&file) {
            f.local_files.push(file);
        }
        let w = face.weight.to_string();
        if !f.weights.contains(&w) {
            f.weights.push(w);
        }
    } else {
        let id = p.data.new_id("font");
        p.data.fonts.push(oracle_brand::model::BrandFont { id, family: face.family.clone(), weights: vec![face.weight.to_string()], local_files: vec![file], ..Default::default() });
    }
    p.save()
}

fn luminance(c: Color32) -> f32 {
    let f = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
}

/// WCAG contrast ratio between two colours.
fn contrast(a: Color32, b: Color32) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.fonts_installed {
            theme::install_fonts(&ctx);
            self.fonts_installed = true;
            ctx.request_repaint();
            return;
        }
        if self.applied_theme != Some(self.settings.theme) {
            theme::apply(&ctx, self.settings.theme);
            self.applied_theme = Some(self.settings.theme);
        }
        if let Some(rx) = &self.scanning {
            match rx.try_recv() {
                Ok(families) => {
                    self.families = families;
                    self.scanning = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(100)),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.scanning = None,
            }
        }
        let t = theme::tokens(&ctx);
        egui::Panel::top("header").frame(egui::Frame::new().fill(t.bg)).exact_size(52.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(16.0);
                ui.label(RichText::new("Oracle").family(FontFamily::Name("display".into())).size(19.0));
                ui.label(RichText::new("Font").color(t.muted).size(15.0));
                if self.scanning.is_some() {
                    ui.add_space(12.0);
                    ui.spinner();
                    ui.label(RichText::new("Lecture des polices…").color(t.muted).size(12.0));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(12.0);
                    let mut choice = self.settings.theme;
                    egui::ComboBox::from_id_salt("theme").selected_text(choice.label()).width(110.0).show_ui(ui, |ui| {
                        for c in [theme::ThemeChoice::System, theme::ThemeChoice::Light, theme::ThemeChoice::Dark] {
                            ui.selectable_value(&mut choice, c, c.label());
                        }
                    });
                    if choice != self.settings.theme {
                        self.settings.theme = choice;
                        self.save_settings();
                    }
                });
            });
        });
        egui::Panel::bottom("status").frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin::symmetric(16, 6))).show(ui, |ui| {
            let text = self.status.clone().unwrap_or_else(|| format!("{} familles · {} polices chargées pour l'aperçu", self.families.len(), self.loaded.len()));
            ui.label(RichText::new(text).color(t.muted).size(12.0));
        });
        egui::Panel::left("filters").frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin { left: 12, right: 12, top: 16, bottom: 12 })).exact_size(220.0).show(ui, |ui| self.sidebar(ui));
        egui::Panel::right("details").frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin { left: 16, right: 16, top: 16, bottom: 12 })).default_size(560.0).resizable(true).show(ui, |ui| self.details(ui));
        egui::CentralPanel::default().frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin { left: 16, right: 16, top: 16, bottom: 12 })).show(ui, |ui| self.list(ui));
    }

    fn on_exit(&mut self) {
        self.save_settings();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_follows_wcag() {
        assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
        assert!((contrast(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 0.01);
        // Oracle red on white: readable for large text.
        assert!(contrast(Color32::from_rgb(0xd3, 0x1c, 0x3b), Color32::WHITE) > 4.5);
    }

    #[test]
    fn a_font_goes_into_a_brand_once() {
        let dir = std::env::temp_dir().join(format!("oracle-font-brand-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        oracle_brand::Project::create(&dir, "Essai").unwrap();
        let face = catalog::read_file(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../style/assets/Lexend-Variable.ttf"), Source::Library).remove(0);
        add_to_brand(&dir, &face).unwrap();
        add_to_brand(&dir, &face).unwrap();
        let p = oracle_brand::Project::open(&dir).unwrap();
        assert_eq!(p.data.fonts.len(), 1);
        assert_eq!(p.data.fonts[0].family, "Lexend");
        assert_eq!(p.data.fonts[0].local_files.len(), 1);
    }
}
