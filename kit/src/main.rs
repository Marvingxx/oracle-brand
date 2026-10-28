//! Oracle Brand Kit: the brand's source of truth (logos in roles, palette, type, rules) and its
//! client deliveries. Nothing is generated: every file and value comes from the designer; exports
//! run only when asked, from the masters, never touching them.

#![windows_subsystem = "windows"]

mod pages;
mod guide_ui;
mod studio_ui;
use oracle_style as theme;
mod thumbs;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use oracle_brand::export::{OutputResult, Plan};
use oracle_brand::project::Project;
use oracle_brand::render::Engines;

fn main() -> eframe::Result<()> {
    let icon = image::load_from_memory(include_bytes!("../assets/hicolor/256x256/apps/app.oracle.brand-kit.png")).ok().map(|i| {
        let i = i.to_rgba8();
        egui::IconData { width: i.width(), height: i.height(), rgba: i.into_raw() }
    });
    let mut viewport = egui::ViewportBuilder::default().with_title("Oracle Brand Kit").with_inner_size([1280.0, 820.0]).with_min_inner_size([900.0, 560.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    let guide_page = args.iter().any(|arg| arg == "--guide");
    let studio_page = args.iter().any(|arg| arg == "--studio");
    let start = args.iter().find(|arg| !arg.starts_with("--")).map(PathBuf::from);
    eframe::run_native(
        "Oracle Brand Kit",
        eframe::NativeOptions { viewport, ..Default::default() },
        Box::new(move |_cc| {
            let mut app = App::new(start);
            if guide_page { app.tab = Tab::Guide; }
            if studio_page { app.tab = Tab::Studio; }
            Ok(Box::new(app))
        }),
    )
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Identity,
    Logos,
    Colors,
    Fonts,
    Rules,
    Guide,
    Studio,
    Presets,
    Export,
    Deliveries,
}

impl Tab {
    const ALL: [Tab; 10] = [Tab::Identity, Tab::Logos, Tab::Colors, Tab::Fonts, Tab::Rules, Tab::Guide, Tab::Studio, Tab::Presets, Tab::Export, Tab::Deliveries];
    fn label(self) -> &'static str {
        match self {
            Tab::Identity => "Identité",
            Tab::Logos => "Logos",
            Tab::Colors => "Couleurs",
            Tab::Fonts => "Typographies",
            Tab::Rules => "Règles",
            Tab::Guide => "Brand guide",
            Tab::Studio => "Studio",
            Tab::Presets => "Presets",
            Tab::Export => "Export",
            Tab::Deliveries => "Livraisons",
        }
    }
}

/// Settings kept between runs (theme, recent projects).
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Settings {
    #[serde(default)]
    theme: theme::ThemeChoice,
    #[serde(default)]
    recent: Vec<String>,
}

fn settings_path() -> Option<PathBuf> {
    // Tests never touch the user's own settings.
    if cfg!(test) {
        return None;
    }
    dirs::config_dir().map(|d| d.join("Oracle").join("brand-kit.json"))
}

/// An export running on a worker thread.
pub struct Running {
    pub rx: Receiver<Progress>,
    pub cancel: Arc<AtomicBool>,
    pub done: usize,
    pub total: usize,
    pub current: String,
}

pub enum Progress {
    Step { done: usize, total: usize, path: String },
    Finished { project: Box<Project>, results: Vec<OutputResult>, cancelled: bool, folder: PathBuf },
}

pub struct App {
    settings: Settings,
    applied_theme: Option<theme::ThemeChoice>,
    fonts_installed: bool,
    pub project: Option<Project>,
    pub tab: Tab,
    pub engines: Engines,
    pub thumbs: thumbs::Thumbs,
    pub studio_ui: studio_ui::State,
    pub status: Option<(String, bool, Instant)>,
    dirty_since: Option<Instant>,
    // Export page state.
    pub plan: Option<Plan>,
    pub plan_assets: Vec<String>,
    pub plan_presets: Vec<String>,
    /// Deliver the Mockup Studio renders too.
    pub plan_mockups: bool,
    /// Files the planned export takes over unchanged from the last delivery: (folder, count).
    pub plan_carry: Option<(String, usize)>,
    pub running: Option<Running>,
    pub last_results: Vec<OutputResult>,
    pub last_folder: Option<PathBuf>,
    // Start page.
    new_name: String,
    /// The Oracle Library window: its assets dropped on the window become logos.
    pub library: oracle_library_ui::InAppLibrary,
    library_dock: oracle_library_ui::LibraryDock,
}

impl App {
    fn new(start: Option<PathBuf>) -> App {
        let settings: Settings = settings_path().and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        let mut app = App {
            settings,
            applied_theme: None,
            fonts_installed: false,
            project: None,
            tab: Tab::Logos,
            engines: Engines::discover(),
            thumbs: thumbs::Thumbs::new(),
            studio_ui: studio_ui::State::default(),
            status: None,
            dirty_since: None,
            plan: None,
            plan_assets: Vec::new(),
            plan_presets: Vec::new(),
            plan_mockups: true,
            plan_carry: None,
            running: None,
            last_results: Vec::new(),
            last_folder: None,
            new_name: String::new(),
            library: oracle_library_ui::InAppLibrary::new("oracle-brand-kit"),
            library_dock: oracle_library_ui::LibraryDock::default(),
        };
        if let Some(p) = start {
            app.open(&p);
        }
        app
    }

    pub fn say(&mut self, text: impl Into<String>, error: bool) {
        let text = text.into();
        oracle_brand::log_line("brand-kit", if error { "ERREUR" } else { "info" }, &text);
        self.status = Some((text, error, Instant::now()));
    }

    /// Marks the project changed; it is saved shortly after the last change.
    pub fn touch(&mut self) {
        self.dirty_since = Some(Instant::now());
        self.plan = None;
        self.studio_ui.invalidate();
    }

    fn save_now(&mut self) {
        if let Some(p) = &self.project {
            match p.save() {
                Ok(()) => self.dirty_since = None,
                Err(e) => self.say(format!("Enregistrement impossible : {e}"), true),
            }
        }
    }

    fn save_settings(&self) {
        if let Some(path) = settings_path() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(json) = serde_json::to_vec_pretty(&self.settings) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    fn remember(&mut self, dir: &std::path::Path) {
        // Its logos, colours and type show in the library and the other Oracle apps.
        if !cfg!(test) {
            oracle_suite_link::brands::register(dir);
        }
        let s = dir.display().to_string();
        self.settings.recent.retain(|r| r != &s);
        self.settings.recent.insert(0, s);
        self.settings.recent.truncate(12);
        self.save_settings();
    }

    pub fn open(&mut self, path: &std::path::Path) {
        self.save_now();
        self.studio_ui = studio_ui::State::default();
        match Project::open(path) {
            Ok(p) => {
                let dir = p.dir.clone();
                self.project = Some(p);
                self.plan = None;
                self.last_results.clear();
                self.remember(&dir);
                self.say(format!("Projet ouvert : {}", dir.display()), false);
            }
            Err(e) => self.say(e, true),
        }
    }

    fn create(&mut self) {
        let name = self.new_name.trim().to_string();
        if name.is_empty() {
            self.say("Donnez un nom à la marque.", true);
            return;
        }
        let Some(parent) = rfd::FileDialog::new().set_title("Dossier où créer le projet").pick_folder() else { return };
        let dir = parent.join(oracle_brand::project::safe_name(&name));
        match Project::create(&dir, &name) {
            Ok(p) => {
                self.project = Some(p);
                self.tab = Tab::Logos;
                self.new_name.clear();
                self.remember(&dir);
                self.say(format!("Projet créé : {}", dir.display()), false);
            }
            Err(e) => self.say(e, true),
        }
    }

    fn start_page(&mut self, ui: &mut egui::Ui) {
        let t = theme::tokens(ui.ctx());
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(560.0);
            ui.heading("Oracle Brand Kit");
            ui.label(egui::RichText::new("La source de vérité d'une identité : logos, couleurs, typographies, règles, et les fichiers à livrer.").color(t.muted));
            ui.add_space(24.0);
            egui::Frame::new().fill(t.panel).stroke(egui::Stroke::new(1.0, t.line)).corner_radius(theme::RADIUS).inner_margin(20).show(ui, |ui| {
                theme::label(ui, "Nouveau projet de marque");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.new_name).hint_text("Nom de la marque").desired_width(320.0));
                    if theme::primary(ui, "Créer…").clicked() {
                        self.create();
                    }
                });
                ui.add_space(8.0);
                if ui.button("Ouvrir un projet existant…").clicked()
                    && let Some(dir) = rfd::FileDialog::new().set_title("Dossier du projet (contenant brand.json)").pick_folder()
                {
                    self.open(&dir);
                }
            });
            if !self.settings.recent.is_empty() {
                ui.add_space(20.0);
                theme::label(ui, "Récents");
                let recent = self.settings.recent.clone();
                for r in recent {
                    let exists = std::path::Path::new(&r).join(oracle_brand::project::FILE_NAME).is_file();
                    let name = std::path::Path::new(&r).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| r.clone());
                    let resp = ui.add_enabled(exists, egui::Button::new(format!("{name}   ·   {r}")).min_size(egui::vec2(560.0, 30.0)));
                    if resp.clicked() {
                        self.open(std::path::Path::new(&r));
                    }
                }
            }
        });
    }
}

impl App {
    /// One frame of the whole window.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.library_dock.initialize(false);
        let ctx = ui.ctx().clone();
        if !self.fonts_installed {
            // The fonts take effect from the next frame: draw nothing until then.
            theme::install_fonts(&ctx);
            self.fonts_installed = true;
            ctx.request_repaint();
            return;
        }
        if self.applied_theme != Some(self.settings.theme) {
            theme::apply(&ctx, self.settings.theme, theme::Accent::Neutral);
            self.applied_theme = Some(self.settings.theme);
        }
        self.thumbs.poll(&ctx);
        pages::poll_export(self, &ctx);
        if let Some(since) = self.dirty_since {
            if since.elapsed() > Duration::from_millis(600) && self.running.is_none() {
                self.save_now();
            } else {
                ctx.request_repaint_after(Duration::from_millis(300));
            }
        }
        // Files dropped on the window become logos.
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| p.is_file()).collect());
        if !dropped.is_empty() && self.project.is_some() {
            pages::add_masters(self, dropped);
            self.tab = Tab::Logos;
        }
        let t = theme::tokens(&ctx);

        egui::Panel::top("header").frame(egui::Frame::new().fill(t.bg)).exact_size(52.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(16.0);
                ui.label(egui::RichText::new("Oracle").family(egui::FontFamily::Name("display".into())).size(19.0));
                ui.label(egui::RichText::new("Brand Kit").color(t.muted).size(15.0));
                if let Some(p) = &self.project {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.label(egui::RichText::new(&p.data.brand_name).strong());
                    ui.label(egui::RichText::new(format!("{} · {}", p.data.version, p.data.status.label_fr())).color(t.muted));
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
                    if self.project.is_some() && ui.button("Fermer le projet").clicked() {
                        self.save_now();
                        self.project = None;
                    }
                    self.library_dock.menu(ui);
                });
            });
            let r = ui.max_rect();
            ui.painter().hline(r.x_range(), r.bottom() - 0.5, egui::Stroke::new(1.0, t.line));
        });

        egui::Panel::bottom("status").frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin::symmetric(16, 6))).show(ui, |ui| {
            let r = ui.max_rect();
            ui.painter().hline(r.x_range(), r.top() - 6.0, egui::Stroke::new(1.0, t.line));
            ui.horizontal(|ui| {
                match &self.status {
                    Some((text, err, at)) if at.elapsed() < Duration::from_secs(10) => {
                        ui.label(egui::RichText::new(text).color(if *err { t.bad } else { t.muted }).size(12.0));
                    }
                    _ => {
                        let v = if self.engines.vector_cli.is_some() { "Oracle Vector : trouvé" } else { "Oracle Vector : introuvable (SVG seulement)" };
                        let e = if self.engines.edit_cli.is_some() { "Oracle Edit : trouvé" } else { "Oracle Edit : introuvable (PSD indisponible)" };
                        ui.label(egui::RichText::new(format!("{v} · {e}")).color(t.muted).size(12.0));
                    }
                }
            });
        });

        if self.library.update_docked(&ctx) { self.library_dock.open_library(); }
        let mut dock = std::mem::take(&mut self.library_dock);
        dock.show(ui, 360.0, |ui, _panel| { self.library.panel_ui(ui, true); });
        self.library_dock = dock;

        if self.project.is_none() {
            egui::CentralPanel::default().frame(egui::Frame::new().fill(t.bg)).show(ui, |ui| self.start_page(ui));
            return;
        }

        egui::Panel::left("tabs").frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin { left: 12, right: 12, top: 16, bottom: 12 })).exact_size(200.0).show(ui, |ui| {
            for tab in Tab::ALL {
                let on = self.tab == tab;
                let text = egui::RichText::new(tab.label()).color(if on { t.on_accent } else { t.ink });
                let b = egui::Button::new(text).fill(if on { t.accent } else { egui::Color32::TRANSPARENT }).min_size(egui::vec2(176.0, 34.0));
                if ui.add(b).clicked() {
                    self.tab = tab;
                }
            }
            let r = ui.max_rect();
            ui.painter().vline(r.right() + 11.5, r.y_range(), egui::Stroke::new(1.0, t.line));
        });

        egui::CentralPanel::default().frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin { left: 28, right: 28, top: 20, bottom: 12 })).show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| match self.tab {
                Tab::Identity => pages::identity(self, ui),
                Tab::Logos => pages::logos(self, ui),
                Tab::Colors => pages::colors(self, ui),
                Tab::Fonts => pages::fonts(self, ui),
                Tab::Rules => pages::rules(self, ui),
                Tab::Guide => guide_ui::show(self, ui),
                Tab::Studio => studio_ui::show(self, ui),
                Tab::Presets => pages::presets(self, ui),
                Tab::Export => pages::export(self, ui),
                Tab::Deliveries => pages::deliveries(self, ui),
            });
        });
    }

}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // Files dropped on the library window go into the library; assets dragged out of it arrive
        // as dropped files (logos).
        self.library.claim_drops(raw_input, None);
        self.library.inject_drops(raw_input);
    }

    fn on_exit(&mut self) {
        self.save_now();
        self.save_settings();
    }
}
