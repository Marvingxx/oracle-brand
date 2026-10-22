//! The live preview: the scene drawn at screen size on a worker thread. Only the latest request is
//! drawn; the photo, masks and designs are kept between requests (until their files change), so
//! dragging a corner or the logo only redoes the warp.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::SystemTime;

use image::RgbaImage;
use oracle_brand::Project;
use oracle_brand::composite::{Quality, TemplateImages, ZoneDesign, compose};
use oracle_brand::mockup::{self, Scene, Template};
use oracle_brand::project::resolve_with;
use oracle_brand::render::{Engines, master_kind, rasterize};

/// The preview's width at most (the photo's own when smaller).
const PREVIEW_WIDTH: u32 = 1600;
/// Designs are drawn once at this size for the preview, then warped.
const DESIGN_SIDE: f32 = 1600.0;

pub struct Request {
    pub generation: u64,
    pub project: Option<Project>,
    pub scene: Option<Scene>,
    pub template_file: PathBuf,
    pub template: Template,
}

pub struct Done {
    pub generation: u64,
    /// The picture and the photo's full size.
    pub result: Result<(RgbaImage, [u32; 2]), String>,
}

pub struct Previewer {
    tx: Sender<Request>,
    rx: Receiver<Done>,
    pub sent: u64,
    pub shown: u64,
    pub texture: Option<egui::TextureHandle>,
    /// The photo's size in pixels, once known.
    pub photo_size: Option<[u32; 2]>,
    pub error: Option<String>,
}

type FileKey = (PathBuf, u64, Option<SystemTime>);

fn key(path: &Path) -> FileKey {
    let meta = std::fs::metadata(path).ok();
    (path.to_path_buf(), meta.as_ref().map(|m| m.len()).unwrap_or(0), meta.and_then(|m| m.modified().ok()))
}

#[derive(Default)]
struct Cache {
    template: Option<(Vec<FileKey>, TemplateImages, [u32; 2])>,
    designs: HashMap<FileKey, Result<RgbaImage, String>>,
}

impl Previewer {
    pub fn new(ctx: &egui::Context, engines: Engines) -> Previewer {
        let (tx, worker_rx) = channel::<Request>();
        let (worker_tx, rx) = channel::<Done>();
        let ctx = ctx.clone();
        std::thread::Builder::new()
            .name("mockup-preview".into())
            .spawn(move || {
                let mut cache = Cache::default();
                while let Ok(mut req) = worker_rx.recv() {
                    while let Ok(newer) = worker_rx.try_recv() {
                        req = newer;
                    }
                    let result = draw(&mut cache, &engines, &req);
                    if worker_tx.send(Done { generation: req.generation, result }).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            })
            .ok();
        Previewer { tx, rx, sent: 0, shown: 0, texture: None, photo_size: None, error: None }
    }

    pub fn request(&mut self, project: Option<&Project>, scene: Option<&Scene>, template_file: &Path, template: &Template) {
        self.sent += 1;
        let _ = self.tx.send(Request { generation: self.sent, project: project.cloned(), scene: scene.cloned(), template_file: template_file.to_path_buf(), template: template.clone() });
    }

    /// Whether the picture shown is older than the last request.
    pub fn busy(&self) -> bool {
        self.shown < self.sent
    }

    pub fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(done) = self.rx.try_recv() {
            self.shown = self.shown.max(done.generation);
            match done.result {
                Ok((img, size)) => {
                    let color = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
                    match &mut self.texture {
                        Some(t) => t.set(color, egui::TextureOptions::LINEAR),
                        None => self.texture = Some(ctx.load_texture("mockup-preview", color, egui::TextureOptions::LINEAR)),
                    }
                    self.photo_size = Some(size);
                    self.error = None;
                }
                Err(e) => self.error = Some(e),
            }
        }
    }

    pub fn clear(&mut self) {
        self.texture = None;
        self.photo_size = None;
        self.error = None;
    }
}

fn draw(cache: &mut Cache, engines: &Engines, req: &Request) -> Result<(RgbaImage, [u32; 2]), String> {
    let t = &req.template;
    let dir = req.template_file.parent().unwrap_or(Path::new("."));
    let mut keys = vec![key(&resolve_with(dir, &t.photo))];
    keys.extend(t.zones.iter().filter_map(|z| z.mask.as_ref()).map(|m| key(&resolve_with(dir, m))));
    keys.extend(t.occlusion.iter().map(|m| key(&resolve_with(dir, m))));
    keys.extend(t.overlays.iter().map(|o| key(&resolve_with(dir, &o.file))));
    // Masks are matched to zones by position: a change in which zones have one reloads too.
    keys.push((PathBuf::from(t.zones.iter().map(|z| if z.mask.is_some() { '1' } else { '0' }).collect::<String>()), 0, None));
    if cache.template.as_ref().is_none_or(|(k, _, _)| *k != keys) {
        cache.template = None;
        let full = mockup::load_template_images(&req.template_file, t, engines)?;
        let size = [full.photo.width(), full.photo.height()];
        let w = size[0].clamp(1, PREVIEW_WIDTH);
        let h = ((f64::from(size[1]) * f64::from(w) / f64::from(size[0].max(1))).round() as u32).max(1);
        let small = |img: &RgbaImage| image::imageops::resize(img, w, h, image::imageops::FilterType::Triangle);
        let small_gray = |img: &image::GrayImage| image::imageops::resize(img, w, h, image::imageops::FilterType::Triangle);
        let images = TemplateImages {
            photo: small(&full.photo),
            zone_masks: full.zone_masks.iter().map(|m| m.as_ref().map(small_gray)).collect(),
            occlusion: full.occlusion.as_ref().map(small_gray),
            overlays: full.overlays.iter().map(small).collect(),
        };
        cache.template = Some((keys, images, size));
    }
    let Some((_, images, size)) = &cache.template else { return Err("photo indisponible".into()) };
    let w = images.photo.width();
    let s = f64::from(w) / f64::from(size[0].max(1));
    // The template at the preview's scale.
    let mut small = t.clone();
    for z in small.zones.iter_mut() {
        z.corners = z.corners.map(|[x, y]| [x * s, y * s]);
    }
    let mut designs = Vec::new();
    if let (Some(project), Some(scene)) = (&req.project, &req.scene) {
        for z in &t.zones {
            let Some(fill) = scene.fill(&z.id) else {
                designs.push(None);
                continue;
            };
            let content = match mockup::content_file(project, &fill.content)? {
                Some((file, label)) => {
                    let k = key(&file);
                    let img = cache.designs.entry(k).or_insert_with(|| design(engines, &file)).as_ref().map_err(|e| format!("« {label} » : {e}"))?;
                    Some(img.clone())
                }
                None => None,
            };
            let background = fill.background.as_ref().map(|p| mockup::paint_rgb(project, p)).transpose()?;
            designs.push(Some(ZoneDesign { content, background, fit: fill.fit, scale: fill.scale, rotation: fill.rotation, offset: fill.offset }));
        }
    }
    let img = compose(&small, images, &designs, w, Quality::PREVIEW)?;
    Ok((img, *size))
}

/// A design drawn once for the preview: vectors at [`DESIGN_SIDE`], pictures reduced to it.
fn design(engines: &Engines, file: &Path) -> Result<RgbaImage, String> {
    let natural = rasterize(engines, file, 1.0)?;
    let long = natural.width().max(natural.height()).max(1) as f32;
    let k = DESIGN_SIDE / long;
    if master_kind(file) == Some("vector") && (k - 1.0).abs() > 0.05 {
        rasterize(engines, file, k)
    } else if k < 1.0 {
        let (w, h) = (((natural.width() as f32) * k).round().max(1.0) as u32, ((natural.height() as f32) * k).round().max(1.0) as u32);
        Ok(image::imageops::resize(&natural, w, h, image::imageops::FilterType::Triangle))
    } else {
        Ok(natural)
    }
}
