//! Turning a master file into one deliverable. Oracle's own engines do the drawing:
//! `oracle-vector-cli` for vector masters (SVG, AI, PDF, EPS, `.oracle-vector`…), `oracle-edit-cli`
//! for PSD and `.oracle-edit` (or `.pcraft`); plain PNG/JPEG/WebP/TIFF masters are read directly. This module adds what
//! a delivery needs on top: exact sizes, margins, a background, the pixel density, and atomic writes
//! that never leave a half-written file in place of a good one.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use image::{ImageEncoder, RgbaImage, imageops::FilterType};

use crate::model::{ExportPreset, Format, parse_hex};

/// Masters Oracle Vector reads.
const VECTOR_EXT: &[&str] = &["svg", "svgz", "ai", "ait", "pdf", "eps", "epsf", "oracle-vector", "emf", "wmf", "dxf"];
/// Masters read directly.
const RASTER_EXT: &[&str] = &["png", "jpg", "jpeg", "webp", "tif", "tiff", "gif", "bmp"];
/// Masters Oracle Edit converts first.
const EDIT_EXT: &[&str] = &["psd", "psb", "oracle-edit", "pcraft"];
/// Largest raster output side, so a typo in a preset can't ask for gigapixels.
pub const MAX_SIDE: u32 = 20_000;

/// The command-line engines of the Oracle apps, where they were found.
#[derive(Clone, Debug, Default)]
pub struct Engines {
    pub vector_cli: Option<PathBuf>,
    pub edit_cli: Option<PathBuf>,
}

impl Engines {
    /// Looks for the CLIs: `ORACLE_VECTOR_CLI` / `ORACLE_EDIT_CLI`, the installed apps, then the
    /// development builds next to this repository.
    pub fn discover() -> Engines {
        Engines { vector_cli: find_cli("ORACLE_VECTOR_CLI", "Oracle Vector", "oracle-vector"), edit_cli: find_cli("ORACLE_EDIT_CLI", "Oracle Edit", "oracle-edit") }
    }
}

fn find_cli(env: &str, app_name: &str, id: &str) -> Option<PathBuf> {
    let exe = format!("{id}-cli{}", std::env::consts::EXE_SUFFIX);
    if let Some(p) = std::env::var_os(env).map(PathBuf::from).filter(|p| p.is_file()) {
        return Some(p);
    }
    let mut candidates: Vec<PathBuf> = ["ProgramFiles", "ProgramW6432", "LOCALAPPDATA"]
        .iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .flat_map(|root| [root.join(app_name).join(&exe), root.join("Programs").join(app_name).join(&exe)])
        .collect();
    // Development builds: <suite>/<id>/target/…/release/.
    for base in [std::env::current_exe().ok(), Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")))].into_iter().flatten() {
        for anc in base.ancestors().take(6) {
            for t in ["target/x86_64-pc-windows-msvc/release", "target/release"] {
                candidates.push(anc.join(id).join(t).join(&exe));
            }
        }
    }
    candidates.into_iter().find(|p| p.is_file())
}

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn temp_path(ext: &str) -> PathBuf {
    let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("oracle-brand-{}-{n}.{ext}", std::process::id()))
}

fn ext_of(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

/// What kind of master a file is, for the export plan.
pub fn master_kind(path: &Path) -> Option<&'static str> {
    let e = ext_of(path);
    if VECTOR_EXT.contains(&e.as_str()) {
        Some("vector")
    } else if RASTER_EXT.contains(&e.as_str()) {
        Some("raster")
    } else if EDIT_EXT.contains(&e.as_str()) {
        Some("document")
    } else {
        None
    }
}

fn run_cli(cli: &Path, args: &[&std::ffi::OsStr]) -> Result<(), String> {
    let mut cmd = Command::new(cli);
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().map_err(|e| format!("impossible de lancer {} : {e}", cli.display()))?;
    if out.status.success() {
        Ok(())
    } else {
        let msg = String::from_utf8_lossy(&out.stderr);
        let msg = if msg.trim().is_empty() { String::from_utf8_lossy(&out.stdout).to_string() } else { msg.to_string() };
        Err(format!("{} a échoué : {}", cli.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), msg.trim()))
    }
}

/// A vector deliverable (SVG or PDF) from a master. An SVG master exported as SVG is copied byte for
/// byte, so the delivered file is exactly the designer's.
pub fn vector_export(engines: &Engines, src: &Path, format: Format, out: &Path) -> Result<(), String> {
    let kind = master_kind(src).ok_or_else(|| format!("{} : format de fichier non pris en charge", src.display()))?;
    if kind != "vector" {
        return Err(format!("{} n'est pas un fichier vectoriel : pas d'export {} possible sans vectorisation", src.display(), format.ext().to_uppercase()));
    }
    let tmp = temp_path(format.ext());
    let result = if format == Format::Svg && ext_of(src) == "svg" {
        std::fs::copy(src, &tmp).map(|_| ()).map_err(|e| format!("{} : {e}", src.display()))
    } else {
        let cli = engines.vector_cli.as_deref().ok_or("Oracle Vector (oracle-vector-cli) est introuvable : il est nécessaire pour cet export")?;
        run_cli(cli, &["convert".as_ref(), src.as_os_str(), tmp.as_os_str()])
    };
    let result = result.and_then(|()| commit(&tmp, out));
    let _ = std::fs::remove_file(&tmp);
    result
}

/// The master's own size in pixels at scale 1.
pub fn intrinsic_size(engines: &Engines, src: &Path) -> Result<(u32, u32), String> {
    let img = rasterize(engines, src, 1.0)?;
    Ok(img.dimensions())
}

/// The master drawn at `scale` (pixels per source unit), with transparency.
pub fn rasterize(engines: &Engines, src: &Path, scale: f32) -> Result<RgbaImage, String> {
    if !src.is_file() {
        return Err(format!("{} est introuvable", src.display()));
    }
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    match master_kind(src) {
        Some("raster") => {
            let img = image::open(src).map_err(|e| format!("{} illisible : {e}", src.display()))?.to_rgba8();
            if (scale - 1.0).abs() < f32::EPSILON {
                Ok(img)
            } else {
                let (w, h) = scaled(img.dimensions(), scale)?;
                Ok(image::imageops::resize(&img, w, h, FilterType::Lanczos3))
            }
        }
        Some("document") => {
            let cli = engines.edit_cli.as_deref().ok_or("Oracle Edit (oracle-edit-cli) est introuvable : il est nécessaire pour lire ce fichier")?;
            let tmp = temp_path("png");
            run_cli(cli, &["convert".as_ref(), src.as_os_str(), tmp.as_os_str()])?;
            let img = image::open(&tmp).map_err(|e| format!("rendu de {} illisible : {e}", src.display()))?.to_rgba8();
            let _ = std::fs::remove_file(&tmp);
            if (scale - 1.0).abs() < f32::EPSILON {
                Ok(img)
            } else {
                let (w, h) = scaled(img.dimensions(), scale)?;
                Ok(image::imageops::resize(&img, w, h, FilterType::Lanczos3))
            }
        }
        Some(_) => match engines.vector_cli.as_deref() {
            Some(cli) => {
                let tmp = temp_path("png");
                let s = format!("{scale}");
                let r = run_cli(cli, &["convert".as_ref(), src.as_os_str(), tmp.as_os_str(), "--scale".as_ref(), s.as_ref()]);
                let img = r.and_then(|()| image::open(&tmp).map_err(|e| format!("rendu de {} illisible : {e}", src.display())));
                let _ = std::fs::remove_file(&tmp);
                Ok(img?.to_rgba8())
            }
            // Without Oracle Vector, an SVG master is still drawn (resvg); other vector formats need it.
            None if ext_of(src) == "svg" => render_svg(src, scale),
            None => Err("Oracle Vector (oracle-vector-cli) est introuvable : il est nécessaire pour dessiner ce fichier".into()),
        },
        None => Err(format!("{} : format de fichier non pris en charge", src.display())),
    }
}

fn scaled((w, h): (u32, u32), scale: f32) -> Result<(u32, u32), String> {
    let (sw, sh) = ((w as f32 * scale).round(), (h as f32 * scale).round());
    if sw < 1.0 || sh < 1.0 || sw > MAX_SIDE as f32 || sh > MAX_SIDE as f32 {
        return Err(format!("taille demandée hors limites : {sw}×{sh} px (maximum {MAX_SIDE})"));
    }
    Ok((sw as u32, sh as u32))
}

/// An SVG drawn with resvg (the fallback, and for thumbnails).
pub fn render_svg(src: &Path, scale: f32) -> Result<RgbaImage, String> {
    use resvg::{tiny_skia, usvg};
    let data = std::fs::read(src).map_err(|e| format!("{} : {e}", src.display()))?;
    let tree = usvg::Tree::from_data(&data, &usvg::Options::default()).map_err(|e| format!("SVG invalide ({}) : {e}", src.display()))?;
    let size = tree.size();
    let (w, h) = scaled((size.width().ceil() as u32, size.height().ceil() as u32), scale)?;
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or("image vide")?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let mut out = RgbaImage::new(w, h);
    for (dst, px) in out.pixels_mut().zip(pixmap.pixels()) {
        let c = px.demultiply();
        *dst = image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Ok(out)
}

/// The raster deliverable for `preset` from a master: sized, with margins and background.
pub fn raster_export(engines: &Engines, src: &Path, preset: &ExportPreset) -> Result<RgbaImage, String> {
    let (iw, ih) = intrinsic_size(engines, src)?;
    let (iw, ih) = (iw.max(1) as f32, ih.max(1) as f32);
    let margin = preset.margin.clamp(0.0, 0.45);
    // The canvas, and the box the logo fits in.
    let (cw, ch, bw, bh) = match (preset.width, preset.height) {
        (Some(w), Some(h)) => {
            let m = margin * w.min(h) as f32;
            (w as f32, h as f32, w as f32 - 2.0 * m, h as f32 - 2.0 * m)
        }
        (Some(w), None) => {
            let bw = w as f32 * (1.0 - 2.0 * margin);
            let bh = bw * ih / iw;
            (w as f32, bh + 2.0 * margin * w as f32, bw, bh)
        }
        (None, Some(h)) => {
            let bh = h as f32 * (1.0 - 2.0 * margin);
            let bw = bh * iw / ih;
            (bw + 2.0 * margin * h as f32, h as f32, bw, bh)
        }
        (None, None) => {
            let s = if preset.scale.is_finite() && preset.scale > 0.0 { preset.scale } else { 1.0 };
            let (bw, bh) = (iw * s, ih * s);
            let m = margin * bw.min(bh) / (1.0 - 2.0 * margin).max(0.1);
            (bw + 2.0 * m, bh + 2.0 * m, bw, bh)
        }
    };
    let (cw, ch) = (cw.round() as u32, ch.round() as u32);
    if cw == 0 || ch == 0 || cw > MAX_SIDE || ch > MAX_SIDE {
        return Err(format!("taille hors limites : {cw}×{ch} px (maximum {MAX_SIDE})"));
    }
    // Fit the logo in its box, keeping its proportions.
    let fit = (bw / iw).min(bh / ih);
    let logo = rasterize(engines, src, fit)?;
    let (lw, lh) = ((iw * fit).round().max(1.0) as u32, (ih * fit).round().max(1.0) as u32);
    let logo = if logo.dimensions() == (lw, lh) { logo } else { image::imageops::resize(&logo, lw, lh, FilterType::Lanczos3) };
    let mut canvas = match &preset.background {
        Some(hex) => {
            let [r, g, b] = parse_hex(hex).ok_or_else(|| format!("couleur de fond invalide : {hex}"))?;
            RgbaImage::from_pixel(cw, ch, image::Rgba([r, g, b, 255]))
        }
        None => RgbaImage::new(cw, ch),
    };
    let x = ((cw as i64 - lw as i64) / 2).max(0);
    let y = ((ch as i64 - lh as i64) / 2).max(0);
    image::imageops::overlay(&mut canvas, &logo, x, y);
    Ok(canvas)
}

/// Encodes `img` for `preset` and writes it atomically to `out`.
pub fn write_raster(img: &RgbaImage, preset: &ExportPreset, out: &Path) -> Result<(), String> {
    let tmp = temp_path(preset.format.ext());
    let bytes = encode(img, preset)?;
    std::fs::write(&tmp, bytes).map_err(|e| format!("{} : {e}", tmp.display()))?;
    let r = commit(&tmp, out);
    let _ = std::fs::remove_file(&tmp);
    r
}

fn encode(img: &RgbaImage, preset: &ExportPreset) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let (w, h) = img.dimensions();
    match preset.format {
        Format::Png => {
            image::codecs::png::PngEncoder::new(&mut buf).write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgba8).map_err(|e| e.to_string())?;
            if let Some(dpi) = preset.dpi {
                buf = png_with_dpi(&buf, dpi)?;
            }
        }
        Format::Jpeg => {
            // JPEG has no transparency: the preset's background is required (checked when planning);
            // whatever is still transparent is flattened onto it.
            let bg = preset.background.as_deref().and_then(parse_hex).ok_or("le JPEG demande une couleur de fond")?;
            let rgb = flatten(img, bg);
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, preset.quality.unwrap_or(92).clamp(1, 100));
            if let Some(dpi) = preset.dpi {
                enc.set_pixel_density(image::codecs::jpeg::PixelDensity::dpi(dpi.min(u16::MAX as u32) as u16));
            }
            enc.encode_image(&rgb).map_err(|e| e.to_string())?;
        }
        Format::Webp => {
            image::codecs::webp::WebPEncoder::new_lossless(&mut buf).write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgba8).map_err(|e| e.to_string())?;
        }
        Format::Svg | Format::Pdf | Format::Eps | Format::Ai => return Err("format vectoriel : pas d'encodage d'image".into()),
    }
    Ok(buf)
}

fn flatten(img: &RgbaImage, [r, g, b]: [u8; 3]) -> image::RgbImage {
    let mut out = image::RgbImage::new(img.width(), img.height());
    for (d, s) in out.pixels_mut().zip(img.pixels()) {
        let a = s[3] as u32;
        let mix = |c: u8, bg: u8| ((c as u32 * a + bg as u32 * (255 - a) + 127) / 255) as u8;
        *d = image::Rgb([mix(s[0], r), mix(s[1], g), mix(s[2], b)]);
    }
    out
}

/// Inserts a `pHYs` chunk (pixels per metre) after IHDR.
fn png_with_dpi(png: &[u8], dpi: u32) -> Result<Vec<u8>, String> {
    // Signature (8) + IHDR chunk (4 len + 4 type + 13 data + 4 crc).
    let ihdr_end = 8 + 25;
    if png.len() < ihdr_end || &png[12..16] != b"IHDR" {
        return Err("PNG inattendu".into());
    }
    let ppm = (dpi as f64 / 0.0254).round() as u32;
    let mut data = Vec::with_capacity(9);
    data.extend_from_slice(&ppm.to_be_bytes());
    data.extend_from_slice(&ppm.to_be_bytes());
    data.push(1); // unit: metre
    let mut chunk = Vec::with_capacity(21);
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(b"pHYs");
    chunk.extend_from_slice(&data);
    let crc = crc32(&chunk[4..]);
    chunk.extend_from_slice(&crc.to_be_bytes());
    let mut out = Vec::with_capacity(png.len() + chunk.len());
    out.extend_from_slice(&png[..ihdr_end]);
    out.extend_from_slice(&chunk);
    out.extend_from_slice(&png[ihdr_end..]);
    Ok(out)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Moves a finished temporary file into place. The destination is replaced only now, once the new
/// file is complete; on failure the previous file stays.
fn commit(tmp: &Path, out: &Path) -> Result<(), String> {
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{} : {e}", dir.display()))?;
    }
    // A rename across volumes fails: copy, then sync.
    if std::fs::rename(tmp, out).is_err() {
        let staged = out.with_file_name(format!(".oracle-{}.part", uuid::Uuid::new_v4()));
        {
            let bytes = std::fs::read(tmp).map_err(|e| format!("{} : {e}", tmp.display()))?;
            let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&staged).map_err(|e| format!("{} : {e}", staged.display()))?;
            f.write_all(&bytes).map_err(|e| format!("{} : {e}", staged.display()))?;
            f.sync_all().map_err(|e| format!("{} : {e}", staged.display()))?;
        }
        std::fs::rename(&staged, out).map_err(|e| format!("{} : {e}", out.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_dpi_chunk_is_valid() {
        let img = RgbaImage::new(2, 2);
        let preset = ExportPreset { dpi: Some(300), ..crate::presets::defaults()[2].clone() };
        let bytes = encode(&img, &preset).unwrap();
        assert_eq!(&bytes[37..41], b"pHYs");
        // Still a readable PNG.
        assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 2);
        // CRC of the IEND chunk, a known value.
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
    }

    #[test]
    fn jpeg_flattens_onto_the_background() {
        let mut img = RgbaImage::new(4, 4);
        img.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
        let flat = flatten(&img, [255, 255, 255]);
        assert_eq!(flat.get_pixel(0, 0).0, [255, 255, 255]);
    }
}
