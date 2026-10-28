//! Oracle Mockup Studio: the brand's logos laid into photographed supports. Strictly 2D and
//! nothing generated: the photo, its zones and masks come from the designer's templates, the logos
//! from the Brand Kit; the studio only warps, masks and blends, and renders when asked.

#![windows_subsystem = "windows"]

mod canvas;
mod inspector;
mod preview;
mod sidebar;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use oracle_brand::Project;
use oracle_brand::mockup::{self, Scene, SceneState, Template};
use oracle_brand::render::Engines;
use oracle_style as theme;

fn main() -> eframe::Result<()> {
    let icon = image::load_from_memory(include_bytes!("../assets/hicolor/256x256/apps/app.oracle.mockup-studio.png")).ok().map(|i| {
        let i = i.to_rgba8();
        egui::IconData { width: i.width(), height: i.height(), rgba: i.into_raw() }
    });
    let mut viewport = egui::ViewportBuilder::default().with_title("Oracle Mockup Studio").with_inner_size([1440.0, 880.0]).with_min_inner_size([1000.0, 600.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let start = std::env::args().nth(1).map(PathBuf::from);
    eframe::run_native("Oracle Mockup Studio", eframe::NativeOptions { viewport, ..Default::default() }, Box::new(move |_cc| Ok(Box::new(App::new(start)))))
}

/// Settings kept between runs.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: theme::ThemeChoice,
    #[serde(default)]
    pub recent: Vec<String>,
    /// Where templates are made and looked for; empty: `Documents/Oracle/Modèles de mockups`.
    #[serde(default)]
    pub templates_root: String,
}

fn settings_path() -> Option<PathBuf> {
    // Tests never touch the user's own settings.
    if cfg!(test) {
        return None;
    }
    dirs::config_dir().map(|d| d.join("Oracle").join("mockup-studio.json"))
}

/// What the centre shows.
#[allow(clippy::large_enum_variant)]
pub enum Mode {
    /// Nothing open: pick a scene or a template.
    None,
    /// A scene of the project, on its template.
    Scene { file: PathBuf, scene: Scene, template_file: PathBuf, template: Template },
    /// A template, being set up (from the scene `back`, if any).
    Template { file: PathBuf, template: Template, back: Option<PathBuf> },
}

impl Mode {
    pub fn template(&self) -> Option<(&Path, &Template)> {
        match self {
            Mode::Scene { template_file, template, .. } => Some((template_file, template)),
            Mode::Template { file, template, .. } => Some((file, template)),
            Mode::None => None,
        }
    }
}

/// Scenes rendering on a worker thread.
pub struct Rendering {
    pub rx: Receiver<(String, Result<PathBuf, String>)>,
    pub total: usize,
    pub done: usize,
    pub failures: Vec<String>,
}

pub struct App {
    pub settings: Settings,
    applied_theme: Option<theme::ThemeChoice>,
    fonts_installed: bool,
    pub project: Option<Project>,
    pub engines: Engines,
    pub mode: Mode,
    pub zone: usize,
    pub preview: Option<preview::Previewer>,
    /// Bumped whenever what the preview shows changes.
    pub preview_dirty: bool,
    pub view: canvas::View,
    pub templates: Vec<(PathBuf, Template)>,
    pub states: HashMap<PathBuf, SceneState>,
    states_at: Option<Instant>,
    pub checked: Vec<PathBuf>,
    pub rendering: Option<Rendering>,
    pub status: Option<(String, bool, Instant)>,
    dirty_since: Option<Instant>,
    pub library: oracle_library_ui::InAppLibrary,
    library_dock: oracle_library_ui::LibraryDock,
}

impl App {
    pub fn new(start: Option<PathBuf>) -> App {
        let settings: Settings = settings_path().and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        let mut app = App {
            settings,
            applied_theme: None,
            fonts_installed: false,
            project: None,
            engines: Engines::discover(),
            mode: Mode::None,
            zone: 0,
            preview: None,
            preview_dirty: true,
            view: canvas::View::default(),
            templates: Vec::new(),
            states: HashMap::new(),
            states_at: None,
            checked: Vec::new(),
            rendering: None,
            status: None,
            dirty_since: None,
            library: oracle_library_ui::InAppLibrary::new("oracle-mockup-studio"),
            library_dock: oracle_library_ui::LibraryDock::default(),
        };
        app.reload_templates();
        if let Some(p) = start {
            app.open_project(&p);
        }
        app
    }

    pub fn say(&mut self, text: impl Into<String>, error: bool) {
        let text = text.into();
        oracle_brand::log_line("mockup-studio", if error { "ERREUR" } else { "info" }, &text);
        self.status = Some((text, error, Instant::now()));
    }

    pub fn templates_root(&self) -> PathBuf {
        if !self.settings.templates_root.is_empty() {
            return PathBuf::from(&self.settings.templates_root);
        }
        dirs::document_dir().unwrap_or_else(std::env::temp_dir).join("Oracle").join("Modèles de mockups")
    }

    /// Folders searched for a scene's template when its path no longer exists.
    pub fn template_roots(&self) -> Vec<PathBuf> {
        let mut roots = vec![self.templates_root()];
        if let Some(p) = &self.project {
            roots.push(p.dir.join("Modèles"));
        }
        roots
    }

    pub fn save_settings(&self) {
        if let Some(path) = settings_path() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(json) = serde_json::to_vec_pretty(&self.settings) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    pub fn reload_templates(&mut self) {
        let mut out = Vec::new();
        for root in self.template_roots() {
            let Ok(entries) = std::fs::read_dir(&root) else { continue };
            for e in entries.flatten() {
                let f = e.path().join(mockup::TEMPLATE_FILE);
                if let Ok((f, t)) = Template::load(&f) {
                    out.push((f, t));
                }
            }
        }
        out.sort_by_key(|(_, t)| t.name.to_lowercase());
        self.templates = out;
    }

    pub fn open_project(&mut self, path: &Path) {
        self.save_now();
        match Project::open(path) {
            Ok(p) => {
                let dir = p.dir.clone();
                self.project = Some(p);
                self.mode = Mode::None;
                self.checked.clear();
                self.states.clear();
                self.states_at = None;
                // Its logos, colours and type show in the library and the other apps.
                if !cfg!(test) {
                    oracle_suite_link::brands::register(&dir);
                }
                let s = dir.display().to_string();
                self.settings.recent.retain(|r| r != &s);
                self.settings.recent.insert(0, s);
                self.settings.recent.truncate(12);
                self.save_settings();
                self.reload_templates();
                self.refresh_states();
                // The first scene, if any, is opened.
                if let Some((_, file, Ok(_))) = self.project.as_ref().map(mockup::scenes).unwrap_or_default().into_iter().next() {
                    self.open_scene(&file);
                }
                self.say(format!("Projet de marque ouvert : {}", dir.display()), false);
            }
            Err(e) => self.say(e, true),
        }
    }

    pub fn open_scene(&mut self, file: &Path) {
        self.save_now();
        let Some(project) = &self.project else { return };
        let result = Scene::load(file).and_then(|scene| {
            let tfile = mockup::find_template(project, &scene, &self.template_roots())?;
            let (tfile, template) = Template::load(&tfile)?;
            Ok((scene, tfile, template))
        });
        match result {
            Ok((scene, template_file, template)) => {
                self.mode = Mode::Scene { file: file.to_path_buf(), scene, template_file, template };
                self.zone = 0;
                self.view = canvas::View::default();
                self.changed_view();
            }
            Err(e) => self.say(e, true),
        }
    }

    pub fn open_template(&mut self, file: &Path, back: Option<PathBuf>) {
        self.save_now();
        match Template::load(file) {
            Ok((file, template)) => {
                self.mode = Mode::Template { file, template, back };
                self.zone = 0;
                self.view = canvas::View::default();
                self.changed_view();
            }
            Err(e) => self.say(e, true),
        }
    }

    fn changed_view(&mut self) {
        if let Some(p) = &mut self.preview {
            p.clear();
        }
        self.preview_dirty = true;
    }

    /// The scene or template changed: preview again, save shortly.
    pub fn touch(&mut self) {
        self.preview_dirty = true;
        self.dirty_since = Some(Instant::now());
    }

    pub fn save_now(&mut self) {
        if self.dirty_since.take().is_none() {
            return;
        }
        let r = match &self.mode {
            Mode::Scene { file, scene, .. } => scene.save(file),
            Mode::Template { file, template, .. } => template.save(file.parent().unwrap_or(Path::new("."))),
            Mode::None => Ok(()),
        };
        match r {
            Ok(()) => self.states_at = None,
            Err(e) => self.say(format!("Enregistrement impossible : {e}"), true),
        }
    }

    /// Each scene's state (from file hashes, remembered while files don't change).
    pub fn refresh_states(&mut self) {
        let Some(project) = &self.project else { return };
        let roots = self.template_roots();
        self.states = mockup::scenes(project)
            .into_iter()
            .map(|(_, file, s)| {
                let state = match s {
                    Ok(s) => mockup::scene_state(project, &s, &roots),
                    Err(e) => SceneState::Broken(e),
                };
                (file, state)
            })
            .collect();
        self.states_at = Some(Instant::now());
    }

    /// Renders scenes to their files on a worker thread.
    pub fn render(&mut self, files: Vec<PathBuf>) {
        if self.rendering.is_some() || files.is_empty() {
            return;
        }
        self.save_now();
        let Some(project) = self.project.clone() else { return };
        let engines = self.engines.clone();
        let roots = self.template_roots();
        let (tx, rx) = std::sync::mpsc::channel();
        let total = files.len();
        std::thread::spawn(move || {
            for f in files {
                let name = f.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let r = mockup::export_scene(&project, &f, &engines, &roots);
                if tx.send((name, r)).is_err() {
                    break;
                }
            }
        });
        self.rendering = Some(Rendering { rx, total, done: 0, failures: Vec::new() });
    }

    fn poll_rendering(&mut self) {
        let Some(r) = &mut self.rendering else { return };
        let mut last = None;
        while let Ok((name, result)) = r.rx.try_recv() {
            r.done += 1;
            match result {
                Ok(path) => last = Some(path),
                Err(e) => r.failures.push(format!("{name} : {e}")),
            }
        }
        if r.done < r.total {
            return;
        }
        let r = self.rendering.take();
        let failures = r.map(|r| r.failures).unwrap_or_default();
        // The open scene's record changed on disk: take it back.
        if let Mode::Scene { file, scene, .. } = &mut self.mode
            && let Ok(s) = Scene::load(file)
        {
            scene.last_render = s.last_render;
        }
        self.refresh_states();
        if failures.is_empty() {
            if let Some(p) = last {
                self.say(format!("Rendu enregistré : {}", p.display()), false);
            }
        } else {
            self.say(format!("Échec du rendu — {}", failures.join(" ; ")), true);
        }
    }

    /// Creates a template from a photo (in the templates folder) and opens it.
    pub fn new_template(&mut self, photo: &Path) {
        let root = self.templates_root();
        if let Err(e) = std::fs::create_dir_all(&root) {
            self.say(format!("Impossible de créer {} : {e}", root.display()), true);
            return;
        }
        match mockup::create_template(&root, photo, "") {
            Ok(file) => {
                self.reload_templates();
                self.open_template(&file, None);
                self.say("Modèle créé : placez les quatre coins de la zone sur la photo.", false);
            }
            Err(e) => self.say(e, true),
        }
    }

    pub fn new_scene(&mut self, template_file: &Path) {
        let Some(project) = &mut self.project else { return };
        match mockup::create_scene(project, template_file, "") {
            Ok(file) => {
                self.refresh_states();
                self.open_scene(&file);
            }
            Err(e) => self.say(e, true),
        }
    }

    /// Files dropped on the window (from the Explorer or the library): onto a zone of the open
    /// scene, a design; otherwise a photo makes a new template.
    fn dropped(&mut self, files: Vec<PathBuf>, pos: Option<egui::Pos2>) {
        let Some(file) = files.into_iter().next() else { return };
        if let Mode::Scene { .. } = self.mode {
            let zone = pos.and_then(|p| self.view.zone_at(&self.mode, p)).unwrap_or(self.zone);
            if oracle_brand::render::master_kind(&file).is_some() {
                self.zone = zone;
                inspector::set_content_file(self, &file);
                return;
            }
        }
        match oracle_brand::render::master_kind(&file) {
            Some("raster" | "document") => self.new_template(&file),
            _ => self.say(format!("{} : déposez une photo (PNG, JPEG, WebP, TIFF, PSD) pour créer un modèle", file.display()), true),
        }
    }

    fn start_page(&mut self, ui: &mut egui::Ui) {
        let t = theme::tokens(ui.ctx());
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(600.0);
            ui.heading("Oracle Mockup Studio");
            ui.label(egui::RichText::new("Vos logos posés sur vos photos de supports : quatre coins, des masques, et un rendu net. Sans IA, sans 3D.").color(t.muted));
            ui.add_space(24.0);
            egui::Frame::new().fill(t.panel).stroke(egui::Stroke::new(1.0, t.line)).corner_radius(theme::RADIUS).inner_margin(20).show(ui, |ui| {
                theme::label(ui, "Projet de marque");
                ui.label(egui::RichText::new("Les scènes appartiennent à une marque du Brand Kit : ses logos et sa palette y sont liés.").color(t.muted));
                ui.add_space(6.0);
                if theme::primary(ui, "Ouvrir un projet de marque…").clicked()
                    && let Some(dir) = rfd::FileDialog::new().set_title("Dossier du projet (contenant brand.json)").pick_folder()
                {
                    self.open_project(&dir);
                }
            });
            // Recent projects: this app's, then the Brand Kit's.
            let mut recent = self.settings.recent.clone();
            if let Some(kit) = dirs::config_dir().map(|d| d.join("Oracle").join("brand-kit.json"))
                && let Ok(b) = std::fs::read(kit)
                && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&b)
            {
                for r in v.get("recent").and_then(|r| r.as_array()).into_iter().flatten().filter_map(|r| r.as_str()) {
                    if !recent.iter().any(|x| x == r) {
                        recent.push(r.to_string());
                    }
                }
            }
            recent.retain(|r| Path::new(r).join(oracle_brand::project::FILE_NAME).is_file());
            if !recent.is_empty() {
                ui.add_space(20.0);
                theme::label(ui, "Projets récents");
                for r in recent.into_iter().take(10) {
                    let name = Path::new(&r).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| r.clone());
                    if ui.add(egui::Button::new(format!("{name}   ·   {r}")).min_size(egui::vec2(600.0, 30.0))).clicked() {
                        self.open_project(Path::new(&r));
                    }
                }
            }
        });
    }

    /// One frame of the whole window.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.library_dock.initialize(true);
        let ctx = ui.ctx().clone();
        if !self.fonts_installed {
            theme::install_fonts(&ctx);
            self.fonts_installed = true;
            ctx.request_repaint();
            return;
        }
        if self.applied_theme != Some(self.settings.theme) {
            theme::apply(&ctx, self.settings.theme, theme::Accent::Neutral);
            self.applied_theme = Some(self.settings.theme);
        }
        let preview = self.preview.get_or_insert_with(|| preview::Previewer::new(&ctx, self.engines.clone()));
        preview.poll(&ctx);
        if self.preview_dirty {
            self.preview_dirty = false;
            if let Some((tfile, template)) = self.mode.template() {
                let scene = match &self.mode {
                    Mode::Scene { scene, .. } => Some(scene),
                    _ => None,
                };
                preview.request(self.project.as_ref(), scene, tfile, template);
            }
        }
        self.poll_rendering();
        if let Some(since) = self.dirty_since {
            if since.elapsed() > Duration::from_millis(600) {
                self.save_now();
            } else {
                ctx.request_repaint_after(Duration::from_millis(300));
            }
        }
        if self.project.is_some() && self.states_at.is_none_or(|t| t.elapsed() > Duration::from_secs(3)) && self.rendering.is_none() {
            self.refresh_states();
        }
        if self.rendering.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| p.is_file()).collect());
        if !dropped.is_empty() && self.project.is_some() {
            let pos = ctx.input(|i| i.pointer.latest_pos());
            self.dropped(dropped, pos);
        }
        let t = theme::tokens(&ctx);

        egui::Panel::top("header").frame(egui::Frame::new().fill(t.bg)).exact_size(52.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(16.0);
                ui.label(egui::RichText::new("Oracle").family(egui::FontFamily::Name("display".into())).size(19.0));
                ui.label(egui::RichText::new("Mockup Studio").color(t.muted).size(15.0));
                if let Some(p) = &self.project {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.label(egui::RichText::new(&p.data.brand_name).strong());
                    ui.label(egui::RichText::new(format!("{} · {} logo(s) · {} couleur(s)", p.data.version, p.data.assets.len(), p.data.colors.len())).color(t.muted));
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
                    self.library_dock.menu(ui);
                    if self.project.is_some() {
                        if ui.button("Fermer le projet").clicked() {
                            self.save_now();
                            self.project = None;
                            self.mode = Mode::None;
                        }
                    }
                });
            });
            let r = ui.max_rect();
            ui.painter().hline(r.x_range(), r.bottom() - 0.5, egui::Stroke::new(1.0, t.line));
        });

        egui::Panel::bottom("status").frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin::symmetric(16, 6))).show(ui, |ui| {
            let r = ui.max_rect();
            ui.painter().hline(r.x_range(), r.top() - 6.0, egui::Stroke::new(1.0, t.line));
            ui.horizontal(|ui| {
                if let Some(r) = &self.rendering {
                    ui.add(egui::Spinner::new().size(12.0));
                    ui.label(egui::RichText::new(format!("Rendu {}/{}…", r.done + 1, r.total)).color(t.muted).size(12.0));
                    return;
                }
                match &self.status {
                    Some((text, err, at)) if at.elapsed() < Duration::from_secs(12) => {
                        ui.label(egui::RichText::new(text).color(if *err { t.bad } else { t.muted }).size(12.0));
                    }
                    _ => {
                        let v = if self.engines.vector_cli.is_some() { "Oracle Vector : trouvé" } else { "Oracle Vector : introuvable (SVG seulement)" };
                        let e = if self.engines.edit_cli.is_some() { "Oracle Edit : trouvé" } else { "Oracle Edit : introuvable (PSD indisponible)" };
                        ui.label(egui::RichText::new(format!("{v} · {e} · Modèles : {}", self.templates_root().display())).color(t.muted).size(12.0));
                    }
                }
            });
        });

        if self.library.update_docked(&ctx) { self.library_dock.open_library(); }
        let mut dock = std::mem::take(&mut self.library_dock);
        dock.show(ui, 360.0, |ui, panel| { match panel {
                oracle_library_ui::LibraryPanel::Library => self.library.panel_ui(ui, true),
                oracle_library_ui::LibraryPanel::Properties => {
                    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| inspector::inspector(self, ui));
                }
            }; });
        self.library_dock = dock;

        if self.project.is_none() {
            egui::CentralPanel::default().frame(egui::Frame::new().fill(t.bg)).show(ui, |ui| self.start_page(ui));
            return;
        }

        egui::Panel::left("scenes").frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin { left: 14, right: 14, top: 16, bottom: 12 })).exact_size(270.0).show(ui, |ui| {
            sidebar::sidebar(self, ui);
            let r = ui.max_rect();
            ui.painter().vline(r.right() + 13.5, r.y_range(), egui::Stroke::new(1.0, t.line));
        });

        egui::CentralPanel::default().frame(egui::Frame::new().fill(t.sunken)).show(ui, |ui| canvas::canvas(self, ui));

    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // Files dropped on the library window go into the library; assets dragged out of it arrive
        // as dropped files.
        self.library.claim_drops(raw_input, None);
        self.library.inject_drops(raw_input);
    }

    fn on_exit(&mut self) {
        self.save_now();
        self.save_settings();
    }
}
