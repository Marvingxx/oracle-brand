//! Oracle Assets: the Oracle Library as an app of its own, to find and use the assets outside the
//! other Oracle apps. The same library as everywhere in the suite (`oracle_library_ui`): sections,
//! the brands of Oracle Brand Kit, favourites, tags, search in French and English. An asset dragged
//! out of the window goes to any app, as a file dragged from the Explorer.

#![windows_subsystem = "windows"]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use oracle_library_ui::{LibraryBrowser, Request};
use oracle_style as theme;

fn main() -> eframe::Result<()> {
    let icon = image::load_from_memory(include_bytes!("../assets/hicolor/256x256/apps/app.oracle.oracle-assets.png")).ok().map(|i| {
        let i = i.to_rgba8();
        egui::IconData { width: i.width(), height: i.height(), rgba: i.into_raw() }
    });
    let mut viewport = egui::ViewportBuilder::default().with_title("Oracle Assets").with_inner_size([1360.0, 860.0]).with_min_inner_size([820.0, 520.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let search = search_arg(std::env::args().skip(1));
    eframe::run_native("Oracle Assets", eframe::NativeOptions { viewport, ..Default::default() }, Box::new(move |_cc| Ok(Box::new(App::new(search)))))
}

/// `--search "métal"` (or a lone word) opens the library on that search.
fn search_arg(mut args: impl Iterator<Item = String>) -> Option<String> {
    let first = args.next()?;
    if first == "--search" { args.next() } else { Some(first) }
}

/// Settings kept between runs.
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Settings {
    #[serde(default)]
    theme: theme::ThemeChoice,
}

fn settings_path() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    dirs::config_dir().map(|d| d.join("Oracle").join("assets.json"))
}

struct App {
    settings: Settings,
    applied_theme: Option<theme::ThemeChoice>,
    fonts_installed: bool,
    library: LibraryBrowser,
}

impl App {
    fn new(search: Option<String>) -> App {
        let settings = settings_path().and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        let mut library = LibraryBrowser::new();
        if let Some(s) = search {
            library.set_search(&s);
        }
        App { settings, applied_theme: None, fonts_installed: false, library }
    }

    fn save_settings(&self) {
        let Some(path) = settings_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_vec_pretty(&self.settings) {
            // Best effort: a setting not kept is only asked again next time.
            let _ = std::fs::write(path, json);
        }
    }

    /// Carries out what the library asks for.
    fn handle(&mut self, frame: &eframe::Frame) {
        for request in self.library.take_requests() {
            match request {
                Request::PickFolder => {
                    if let Some(folder) = rfd::FileDialog::new().set_title("Dossier de la bibliothèque").pick_folder() {
                        self.library.set_root(folder);
                    }
                }
                Request::Drag { file, preview } => {
                    if start_drag(frame, &file, preview.as_deref()) {
                        self.library.used(&file);
                    }
                }
                Request::OpenWith { app, file } => {
                    let result = match oracle_suite_link::App::from_id(app).and_then(oracle_suite_link::App::exe) {
                        Some(exe) => std::process::Command::new(exe).arg(&file).spawn().map(|_| ()).map_err(|e| e.to_string()),
                        // Not installed: the system's default app still opens it.
                        None => open_with_system(&file),
                    };
                    self.report(&file, result);
                }
                Request::Open(file) => {
                    let result = open_with_system(&file);
                    self.report(&file, result);
                }
                Request::Reveal(file) => {
                    if let Err(e) = reveal(&file) {
                        log::warn!("couldn't show {}: {e}", file.display());
                    }
                }
            }
        }
    }

    fn report(&mut self, file: &Path, result: Result<(), String>) {
        match result {
            Ok(()) => self.library.used(file),
            Err(e) => self.library.say(e),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.fonts_installed {
            // The fonts take effect from the next frame.
            theme::install_fonts(&ctx);
            self.fonts_installed = true;
            ctx.request_repaint();
            return;
        }
        if self.applied_theme != Some(self.settings.theme) {
            theme::apply(&ctx, self.settings.theme, theme::Accent::Assets);
            self.applied_theme = Some(self.settings.theme);
        }
        let t = theme::tokens(&ctx);
        egui::Panel::top("header").frame(egui::Frame::new().fill(t.bg)).exact_size(52.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(16.0);
                ui.label(egui::RichText::new("Oracle").family(egui::FontFamily::Name("display".into())).size(19.0));
                ui.label(egui::RichText::new("Assets").color(t.muted).size(15.0));
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
            let r = ui.max_rect();
            ui.painter().hline(r.x_range(), r.bottom() - 0.5, egui::Stroke::new(1.0, t.line));
        });
        self.library.accent = theme::tokens(ui.ctx()).accent;
        self.library.french = true;
        egui::CentralPanel::default().frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin { left: 20, right: 20, top: 18, bottom: 12 })).show(ui, |ui| self.library.ui(ui));
        self.handle(frame);
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // Files dropped on the window go into the library.
        let files: Vec<PathBuf> = raw_input.dropped_files.drain(..).map(|f| f.path().to_path_buf()).filter(|p| p.is_absolute()).collect();
        if !files.is_empty() {
            let _ = self.library.add_files(&files); // The browser says what happened.
        }
    }

    fn on_exit(&mut self) {
        self.save_settings();
    }
}

/// Starts a native drag of `file` from the window; true when it was dropped somewhere.
#[cfg(any(target_os = "windows", target_os = "macos"))]
fn start_drag(frame: &eframe::Frame, file: &Path, preview: Option<&Path>) -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    let Ok(file) = std::path::absolute(file) else { return false };
    const MARK: &[u8] = include_bytes!("../assets/hicolor/64x64/apps/app.oracle.oracle-assets.png");
    let image = match preview.filter(|p| p.is_file() && p.extension().is_some_and(|e| matches!(e.to_string_lossy().to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"))) {
        Some(p) => drag::Image::File(p.to_path_buf()),
        None => drag::Image::Raw(MARK.to_vec()),
    };
    let dropped = Arc::new(AtomicBool::new(false));
    let flag = dropped.clone();
    let result = drag::start_drag(
        frame,
        drag::DragItem::Files(vec![file]),
        image,
        move |result, _| flag.store(matches!(result, drag::DragResult::Dropped), Ordering::SeqCst),
        drag::Options::default(),
    );
    if let Err(e) = result {
        log::warn!("couldn't start the drag: {e}");
        return false;
    }
    dropped.load(Ordering::SeqCst)
}

/// No native drag out of a winit window on Linux yet.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn start_drag(_frame: &eframe::Frame, _file: &Path, _preview: Option<&Path>) -> bool {
    false
}

fn open_with_system(file: &Path) -> Result<(), String> {
    let program = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program).arg(file).spawn().map(|_| ()).map_err(|e| e.to_string())
}

fn reveal(file: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Passed raw: Explorer wants `/select,"<path>"` as one argument, quoted inside.
        let arg = format!("/select,\"{}\"", file.display());
        return std::process::Command::new("explorer").raw_arg(arg).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    #[cfg(target_os = "macos")]
    {
        return std::process::Command::new("open").arg("-R").arg(file).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    #[allow(unreachable_code)]
    open_with_system(file.parent().unwrap_or(file))
}

#[cfg(test)]
mod tests {
    use super::search_arg;

    #[test]
    fn a_search_from_the_command_line() {
        assert_eq!(search_arg(["--search".to_string(), "métal".to_string()].into_iter()).as_deref(), Some("métal"));
        assert_eq!(search_arg(["papier".to_string()].into_iter()).as_deref(), Some("papier"));
        assert_eq!(search_arg(std::iter::empty()), None);
    }
}
