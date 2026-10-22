//! Previews of master files, drawn on a worker thread so the window never waits for an engine.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::SystemTime;

use oracle_brand::render::{self, Engines};

/// The longest side of a preview, in pixels.
const SIDE: f32 = 320.0;

type Key = (PathBuf, Option<SystemTime>);

pub enum Thumb {
    Loading,
    Ready(egui::TextureHandle, [u32; 2]),
    Failed(String),
}

type Loaded = (Key, Result<image::RgbaImage, String>);

pub struct Thumbs {
    map: HashMap<Key, Thumb>,
    tx: Sender<(PathBuf, Sender<Loaded>)>,
    results_tx: Sender<(Key, Result<image::RgbaImage, String>)>,
    results: Receiver<(Key, Result<image::RgbaImage, String>)>,
}

impl Thumbs {
    pub fn new() -> Thumbs {
        let (tx, rx) = channel::<(PathBuf, Sender<(Key, Result<image::RgbaImage, String>)>)>();
        std::thread::spawn(move || {
            let engines = Engines::discover();
            while let Ok((path, back)) = rx.recv() {
                let key = (path.clone(), mtime(&path));
                let _ = back.send((key, preview(&engines, &path)));
            }
        });
        let (results_tx, results) = channel();
        Thumbs { map: HashMap::new(), tx, results_tx, results }
    }

    pub fn poll(&mut self, ctx: &egui::Context) {
        while let Ok((key, result)) = self.results.try_recv() {
            let thumb = match result {
                Ok(img) => {
                    let size = [img.width() as usize, img.height() as usize];
                    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
                    let tex = ctx.load_texture(format!("thumb:{}", key.0.display()), color, egui::TextureOptions::LINEAR);
                    Thumb::Ready(tex, [img.width(), img.height()])
                }
                Err(e) => Thumb::Failed(e),
            };
            self.map.insert(key, thumb);
            ctx.request_repaint();
        }
    }

    /// The preview of `path`, asking for it when it isn't known; a changed file is drawn again.
    pub fn get(&mut self, path: &Path) -> &Thumb {
        let key = (path.to_path_buf(), mtime(path));
        if !self.map.contains_key(&key) {
            let _ = self.tx.send((path.to_path_buf(), self.results_tx.clone()));
            self.map.insert(key.clone(), Thumb::Loading);
        }
        &self.map[&key]
    }
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

fn preview(engines: &Engines, path: &Path) -> Result<image::RgbaImage, String> {
    // SVG: resvg directly (fast); other masters through the Oracle engines.
    let is_svg = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"));
    let full = if is_svg { render::render_svg(path, 1.0)? } else { render::rasterize(engines, path, 1.0)? };
    let (w, h) = full.dimensions();
    let s = SIDE / w.max(h).max(1) as f32;
    if s >= 1.0 && !is_svg {
        return Ok(full);
    }
    if is_svg {
        return render::render_svg(path, s.min(8.0));
    }
    let (nw, nh) = (((w as f32 * s).round() as u32).max(1), ((h as f32 * s).round() as u32).max(1));
    Ok(image::imageops::resize(&full, nw, nh, image::imageops::FilterType::Triangle))
}
