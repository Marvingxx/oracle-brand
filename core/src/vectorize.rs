//! A logo received as an image (PNG, JPEG…) becomes an SVG master, traced by VTracer
//! (visioncortex, MIT OR Apache-2.0): colour images, not only black and white. The image master is
//! never touched: the SVG is a new asset beside it, which the exports, the grids and the colour
//! treatments can use like a drawn master. A tracing is a starting point to check, not a redrawing.

use std::path::{Path, PathBuf};

use image::RgbaImage;
use serde::{Deserialize, Serialize};

use crate::model::Asset;
use crate::project::Project;

/// The largest side traced; a bigger image is reduced first (time grows with the pixels).
pub const MAX_SIDE: u32 = 2400;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Detail {
    /// Few flat colours and clean curves: logos, pictograms.
    #[default]
    Logo,
    /// More colours and finer shapes: illustrations, textured marks.
    Detailed,
}

impl Detail {
    pub fn label_fr(self) -> &'static str {
        match self {
            Detail::Logo => "Logo (aplats nets)",
            Detail::Detailed => "Détaillé",
        }
    }

    fn config(self) -> vtracer::Config {
        match self {
            Detail::Logo => vtracer::Config {
                filter_speckle: 8,
                color_precision: 5,
                layer_difference: 24,
                corner_threshold: 60,
                length_threshold: 4.0,
                splice_threshold: 45,
                path_precision: Some(2),
                ..vtracer::Config::default()
            },
            Detail::Detailed => vtracer::Config {
                filter_speckle: 2,
                color_precision: 7,
                layer_difference: 8,
                path_precision: Some(3),
                ..vtracer::Config::default()
            },
        }
    }
}

/// The image as SVG markup.
pub fn trace(img: &RgbaImage, detail: Detail) -> Result<String, String> {
    if img.width() == 0 || img.height() == 0 {
        return Err("image vide".into());
    }
    let img = if img.width().max(img.height()) > MAX_SIDE {
        let k = f64::from(MAX_SIDE) / f64::from(img.width().max(img.height()));
        let (w, h) = (
            ((f64::from(img.width()) * k).round() as u32).max(1),
            ((f64::from(img.height()) * k).round() as u32).max(1),
        );
        image::imageops::resize(img, w, h, image::imageops::FilterType::Lanczos3)
    } else {
        img.clone()
    };
    let (width, height) = (img.width() as usize, img.height() as usize);
    let color = vtracer::ColorImage {
        pixels: img.into_raw(),
        width,
        height,
    };
    // The tracer is outside code: a failure inside it must not take the app down with it.
    let traced = std::panic::catch_unwind(move || vtracer::convert(color, detail.config()))
        .map_err(|_| "la vectorisation a échoué sur cette image".to_string())??;
    Ok(traced.to_string())
}

/// A file name beside `master` that doesn't exist yet: `<nom>-vectorise.svg`, `-2`, `-3`…
fn free_path(master: &Path) -> PathBuf {
    let dir = master.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = master
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "logo".into());
    let mut n = 1;
    loop {
        let name = if n == 1 {
            format!("{stem}-vectorise.svg")
        } else {
            format!("{stem}-vectorise-{n}.svg")
        };
        let p = dir.join(name);
        if !p.exists() {
            return p;
        }
        n += 1;
    }
}

/// Traces the image master of `asset_id` into a new SVG asset (same role and variant) and returns
/// its id. The project is changed in memory; the caller saves it.
pub fn vectorize(project: &mut Project, asset_id: &str, detail: Detail) -> Result<String, String> {
    let asset = project
        .data
        .assets
        .iter()
        .find(|a| a.id == asset_id)
        .cloned()
        .ok_or_else(|| format!("logo {asset_id} introuvable"))?;
    let master = project.resolve(&asset.source);
    if crate::render::master_kind(&master) == Some("vector") {
        return Err("ce logo est déjà vectoriel".into());
    }
    let img = image::open(&master)
        .map_err(|e| format!("{} : {e}", master.display()))?
        .to_rgba8();
    let svg = trace(&img, detail)?;
    let out = free_path(&master);
    let tmp = out.with_extension("svg.tmp");
    std::fs::write(&tmp, svg.as_bytes()).map_err(|e| format!("{} : {e}", tmp.display()))?;
    std::fs::rename(&tmp, &out).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{} : {e}", out.display())
    })?;
    let id = project.data.new_id("asset");
    let label = if asset.label.is_empty() {
        "vectorisé".to_string()
    } else {
        format!("{} (vectorisé)", asset.label)
    };
    project.data.assets.push(Asset {
        id: id.clone(),
        role: asset.role,
        label,
        variant: asset.variant,
        source: project.store_path(&out),
        source_hash: String::new(),
    });
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A red disc and a black bar on white.
    fn logo() -> RgbaImage {
        RgbaImage::from_fn(200, 100, |x, y| {
            let (dx, dy) = (x as f64 - 50.0, y as f64 - 50.0);
            if dx * dx + dy * dy < 35.0 * 35.0 {
                image::Rgba([211, 28, 59, 255])
            } else if (110..190).contains(&x) && (35..65).contains(&y) {
                image::Rgba([27, 26, 23, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        })
    }

    #[test]
    fn a_colour_image_becomes_svg_with_its_colours() {
        let svg = trace(&logo(), Detail::Logo).unwrap();
        assert!(svg.contains("<svg") && svg.contains("<path"));
        let lower = svg.to_lowercase();
        // The red and the near-black are both traced as their own shapes.
        assert!(
            lower.contains("#d31c3b") || lower.contains("#d21c3b") || lower.contains("#d31d3b"),
            "{}",
            &svg[..svg.len().min(600)]
        );
        assert!(resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default()).is_ok());
    }

    #[test]
    fn an_empty_image_is_refused_and_a_huge_one_reduced() {
        assert!(trace(&RgbaImage::new(0, 0), Detail::Logo).is_err());
        let big = RgbaImage::from_pixel(MAX_SIDE * 2, 10, image::Rgba([0, 0, 0, 255]));
        let svg = trace(&big, Detail::Logo).unwrap();
        assert!(
            svg.contains(&format!("width=\"{MAX_SIDE}\"")),
            "{}",
            &svg[..svg.len().min(300)]
        );
    }

    #[test]
    fn vectorizing_adds_an_asset_beside_the_master_and_never_touches_it() {
        let tmp = tempfile::tempdir().unwrap();
        let mut p = Project::create(&tmp.path().join("p"), "Orbe").unwrap();
        logo().save(p.dir.join("logo.png")).unwrap();
        let before = std::fs::read(p.dir.join("logo.png")).unwrap();
        p.data.assets.push(Asset {
            id: "logo".into(),
            role: crate::model::Role::PrimaryLogo,
            label: "Logo principal".into(),
            variant: crate::model::Variant::Color,
            source: "logo.png".into(),
            source_hash: String::new(),
        });
        let id = vectorize(&mut p, "logo", Detail::Logo).unwrap();
        let a = p.data.assets.iter().find(|a| a.id == id).unwrap();
        assert_eq!(a.label, "Logo principal (vectorisé)");
        assert_eq!(a.role, crate::model::Role::PrimaryLogo);
        assert!(p.resolve(&a.source).ends_with("logo-vectorise.svg"));
        assert_eq!(
            crate::render::master_kind(&p.resolve(&a.source)),
            Some("vector")
        );
        assert_eq!(std::fs::read(p.dir.join("logo.png")).unwrap(), before);
        // A second tracing doesn't overwrite the first; an SVG master is refused.
        let again = vectorize(&mut p, "logo", Detail::Detailed).unwrap();
        assert!(
            p.resolve(&p.data.assets.iter().find(|a| a.id == again).unwrap().source)
                .ends_with("logo-vectorise-2.svg")
        );
        assert!(vectorize(&mut p, &id, Detail::Logo).is_err());
    }
}
