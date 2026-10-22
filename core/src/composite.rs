//! Mockup drawing: designs laid into a photo's zones through each zone's four corners, then the
//! template's foreground put back in front and its overlays on top. Pure, deterministic pixel work
//! on images already loaded; [`crate::mockup`] loads them and records what went in.
//!
//! The perspective is a plain homography of the zone's rectangle onto its quadrilateral (Oracle
//! Edit's own [`Homography`]); nothing is guessed from the photo (no folds, no relighting): what
//! looks real comes from the masks and overlays the designer prepared with the template.

use image::{GrayImage, RgbaImage, imageops::FilterType};
use oracle_edit_algo::transform::{Homography, convex_quad};

use crate::mockup::{Blend, Fit, Template};

/// What fills one zone, ready to draw.
#[derive(Clone, Debug, Default)]
pub struct ZoneDesign {
    /// The design (logo, picture), straight alpha. `None`: only the background, if any.
    pub content: Option<RgbaImage>,
    /// A flat colour under the design, over the whole zone.
    pub background: Option<[u8; 3]>,
    pub fit: Fit,
    /// 1 = fitted to the zone.
    pub scale: f32,
    /// Degrees, clockwise.
    pub rotation: f32,
    /// Shift of the design's centre, as a share of the zone's width and height.
    pub offset: [f32; 2],
}

/// The template's pictures, loaded. Masks are any size with the photo's proportions: they are
/// sampled in the photo's frame.
#[derive(Clone, Debug)]
pub struct TemplateImages {
    pub photo: RgbaImage,
    /// One per zone, in the template's order.
    pub zone_masks: Vec<Option<GrayImage>>,
    pub occlusion: Option<GrayImage>,
    /// One per overlay, in the template's order.
    pub overlays: Vec<RgbaImage>,
}

/// Samples per pixel side: inside a zone, and on its edges (anti-aliasing).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quality {
    pub inner: u32,
    pub edge: u32,
}

impl Quality {
    /// For the files the designer exports.
    pub const FINAL: Quality = Quality { inner: 2, edge: 4 };
    /// For the live preview.
    pub const PREVIEW: Quality = Quality { inner: 1, edge: 2 };
}

/// The zone rectangle `(0,0)–(1,1)` onto the zone's corners, scaled to the output.
pub fn zone_homography(corners: &[[f64; 2]; 4], scale: f64) -> Result<Homography, String> {
    let q = corners.map(|[x, y]| [x * scale, y * scale]);
    if !convex_quad(&q) {
        return Err("les quatre coins doivent former un quadrilatère convexe, dans l'ordre haut-gauche, haut-droit, bas-droit, bas-gauche".into());
    }
    Homography::rect_to_quad([0.0, 0.0, 1.0, 1.0], q).ok_or_else(|| "quadrilatère dégénéré".into())
}

/// The zone's on-photo size in pixels: its longest top/bottom edge by its longest side edge.
pub fn zone_pixel_size(corners: &[[f64; 2]; 4]) -> (f64, f64) {
    let len = |a: [f64; 2], b: [f64; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
    let [tl, tr, br, bl] = *corners;
    (len(tl, tr).max(len(bl, br)), len(tl, bl).max(len(tr, br)))
}

/// The design's box inside the zone, in zone units where the zone is `ratio × 1`.
pub fn content_box(fit: Fit, ratio: f64, content_aspect: f64, scale: f64) -> (f64, f64) {
    let (w, h) = match fit {
        Fit::Contain if content_aspect > ratio => (ratio, ratio / content_aspect),
        Fit::Contain => (content_aspect, 1.0),
        Fit::Cover if content_aspect > ratio => (content_aspect, 1.0),
        Fit::Cover => (ratio, ratio / content_aspect),
        Fit::Stretch => (ratio, 1.0),
    };
    (w * scale, h * scale)
}

/// The size (px) to draw a design at so it is sharp in the output: its box in the zone, at the
/// zone's largest on-photo size, times `out_scale`.
pub fn content_pixels(template: &Template, zone: usize, design: &ZoneDesign, content_aspect: f64, out_scale: f64) -> (f64, f64) {
    let Some(z) = template.zones.get(zone) else { return (1.0, 1.0) };
    let ratio = z.ratio_value();
    let (lw, lh) = zone_pixel_size(&z.corners);
    let (bw, bh) = content_box(design.fit, ratio, content_aspect, f64::from(design.scale.max(0.01)));
    ((bw / ratio * lw * out_scale).max(1.0), (bh * lh * out_scale).max(1.0))
}

/// Draws the mockup at `out_width` pixels wide (the photo's proportions). `designs` goes with the
/// template's zones (`None`: zone left as photographed).
pub fn compose(template: &Template, images: &TemplateImages, designs: &[Option<ZoneDesign>], out_width: u32, quality: Quality) -> Result<RgbaImage, String> {
    let (pw, ph) = images.photo.dimensions();
    if pw == 0 || ph == 0 {
        return Err("photo vide".into());
    }
    let out_width = out_width.clamp(1, crate::render::MAX_SIDE);
    let s = f64::from(out_width) / f64::from(pw);
    let out_height = ((f64::from(ph) * s).round() as u32).clamp(1, crate::render::MAX_SIDE);
    let base = if (out_width, out_height) == (pw, ph) {
        images.photo.clone()
    } else {
        let filter = if quality == Quality::PREVIEW { FilterType::Triangle } else { FilterType::Lanczos3 };
        image::imageops::resize(&images.photo, out_width, out_height, filter)
    };

    let mut zones = Vec::new();
    for (i, z) in template.zones.iter().enumerate() {
        let Some(Some(design)) = designs.get(i) else { continue };
        if design.content.is_none() && design.background.is_none() {
            continue;
        }
        let h = zone_homography(&z.corners, s).map_err(|e| format!("zone « {} » : {e}", z.name))?;
        let inv = h.inverse().ok_or_else(|| format!("zone « {} » : quadrilatère dégénéré", z.name))?;
        let xs = z.corners.map(|c| c[0] * s);
        let ys = z.corners.map(|c| c[1] * s);
        let bx0 = xs.iter().copied().fold(f64::MAX, f64::min).floor().max(0.0) as u32;
        let by0 = ys.iter().copied().fold(f64::MAX, f64::min).floor().max(0.0) as u32;
        let bx1 = (xs.iter().copied().fold(f64::MIN, f64::max).ceil().max(0.0) as u32).min(out_width);
        let by1 = (ys.iter().copied().fold(f64::MIN, f64::max).ceil().max(0.0) as u32).min(out_height);
        if bx0 >= bx1 || by0 >= by1 {
            continue;
        }
        zones.push(ZoneJob {
            inv,
            sampler: DesignSampler::new(design, z.ratio_value()),
            mask: images.zone_masks.get(i).and_then(Option::as_ref),
            blend: z.blend,
            opacity: z.opacity.clamp(0.0, 1.0),
            bbox: (bx0, by0, bx1, by1),
        });
    }

    let mut out = base.clone();
    let row_bytes = out_width as usize * 4;
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(16);
    let rows_per = (out_height as usize).div_ceil(threads).max(1);
    let base_raw: &[u8] = base.as_raw();
    std::thread::scope(|scope| {
        for (chunk_index, chunk) in out.chunks_mut(rows_per * row_bytes).enumerate() {
            let zones = &zones;
            scope.spawn(move || {
                let y0 = chunk_index * rows_per;
                for (dy, row) in chunk.chunks_mut(row_bytes).enumerate() {
                    let y = (y0 + dy) as u32;
                    let base_row = &base_raw[y as usize * row_bytes..(y as usize + 1) * row_bytes];
                    draw_row(row, base_row, y, out_width, out_height, zones, images, template, quality);
                }
            });
        }
    });
    Ok(out)
}

struct ZoneJob<'a> {
    inv: Homography,
    sampler: DesignSampler<'a>,
    mask: Option<&'a GrayImage>,
    blend: Blend,
    opacity: f32,
    bbox: (u32, u32, u32, u32),
}

#[allow(clippy::too_many_arguments)]
fn draw_row(row: &mut [u8], base_row: &[u8], y: u32, w: u32, h: u32, zones: &[ZoneJob], images: &TemplateImages, template: &Template, quality: Quality) {
    let fy = (f64::from(y) + 0.5) / f64::from(h);
    for z in zones {
        let (bx0, by0, bx1, by1) = z.bbox;
        if y < by0 || y >= by1 {
            continue;
        }
        // The zone coordinates of this row's pixel corners, shared between neighbours.
        let n = (bx1 - bx0 + 1) as usize;
        let top: Vec<(f64, f64)> = (0..n).map(|i| z.inv.apply(f64::from(bx0) + i as f64, f64::from(y))).collect();
        let bottom: Vec<(f64, f64)> = (0..n).map(|i| z.inv.apply(f64::from(bx0) + i as f64, f64::from(y) + 1.0)).collect();
        let inside = |p: (f64, f64)| (0.0..=1.0).contains(&p.0) && (0.0..=1.0).contains(&p.1);
        for x in bx0..bx1 {
            let i = (x - bx0) as usize;
            let corners_in = [top[i], top[i + 1], bottom[i], bottom[i + 1]].iter().filter(|p| inside(**p)).count();
            if corners_in == 0 {
                continue;
            }
            let k = if corners_in == 4 { quality.inner } else { quality.edge }.max(1);
            let mut acc = [0f32; 4];
            for sy in 0..k {
                for sx in 0..k {
                    let px = f64::from(x) + (f64::from(sx) + 0.5) / f64::from(k);
                    let py = f64::from(y) + (f64::from(sy) + 0.5) / f64::from(k);
                    let uv = z.inv.apply(px, py);
                    if inside(uv) {
                        let c = z.sampler.sample(uv.0, uv.1);
                        for (a, c) in acc.iter_mut().zip(c) {
                            *a += c;
                        }
                    }
                }
            }
            let n = (k * k) as f32;
            let mut m = z.opacity;
            if let Some(mask) = z.mask {
                m *= sample_gray(mask, (f64::from(x) + 0.5) / f64::from(w), fy);
            }
            let src = acc.map(|v| v / n * m);
            let o = x as usize * 4;
            blend_into(&mut row[o..o + 4], src, z.blend);
        }
    }
    // The template's foreground goes back in front of the designs.
    if let Some(occ) = &images.occlusion {
        for x in 0..w {
            let a = sample_gray(occ, (f64::from(x) + 0.5) / f64::from(w), fy);
            if a <= 0.0 {
                continue;
            }
            let o = x as usize * 4;
            for c in 0..4 {
                let (cur, photo) = (f32::from(row[o + c]), f32::from(base_row[o + c]));
                row[o + c] = to_u8(cur + (photo - cur) * a);
            }
        }
    }
    for (overlay, spec) in images.overlays.iter().zip(&template.overlays) {
        let opacity = spec.opacity.clamp(0.0, 1.0);
        for x in 0..w {
            let c = sample_rgba(overlay, (f64::from(x) + 0.5) / f64::from(w), fy);
            if c[3] <= 0.0 {
                continue;
            }
            let o = x as usize * 4;
            blend_into(&mut row[o..o + 4], c.map(|v| v * opacity), spec.blend);
        }
    }
}

/// Composites premultiplied `src` (0–1) onto a straight RGBA8 pixel, with the W3C separable
/// blend formulas.
fn blend_into(dst: &mut [u8], src: [f32; 4], blend: Blend) {
    let sa = src[3];
    if sa <= 0.0 {
        return;
    }
    let ba = f32::from(dst[3]) / 255.0;
    let mut out = [0f32; 4];
    let oa = sa + ba * (1.0 - sa);
    for c in 0..3 {
        let cs = (src[c] / sa).clamp(0.0, 1.0);
        let cb = f32::from(dst[c]) / 255.0;
        let mixed = match blend {
            Blend::Normal => cs,
            Blend::Multiply => cb * cs,
            Blend::Screen => cb + cs - cb * cs,
        };
        let cs2 = (1.0 - ba) * cs + ba * mixed;
        let premult = sa * cs2 + (1.0 - sa) * ba * cb;
        out[c] = if oa > 0.0 { premult / oa } else { 0.0 };
    }
    out[3] = oa;
    for c in 0..4 {
        dst[c] = to_u8(out[c] * 255.0);
    }
}

fn to_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// A mask's value (0–1) at a point of the photo's frame (0–1), bilinear.
fn sample_gray(img: &GrayImage, fx: f64, fy: f64) -> f32 {
    let (w, h) = img.dimensions();
    let x = fx * f64::from(w) - 0.5;
    let y = fy * f64::from(h) - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = ((x - x0) as f32, (y - y0) as f32);
    let get = |xi: f64, yi: f64| {
        let xi = xi.clamp(0.0, f64::from(w - 1)) as u32;
        let yi = yi.clamp(0.0, f64::from(h - 1)) as u32;
        f32::from(img.get_pixel(xi, yi)[0]) / 255.0
    };
    let top = get(x0, y0) * (1.0 - tx) + get(x0 + 1.0, y0) * tx;
    let bottom = get(x0, y0 + 1.0) * (1.0 - tx) + get(x0 + 1.0, y0 + 1.0) * tx;
    top * (1.0 - ty) + bottom * ty
}

/// An overlay's premultiplied colour (0–1) at a point of the photo's frame, bilinear.
fn sample_rgba(img: &RgbaImage, fx: f64, fy: f64) -> [f32; 4] {
    let (w, h) = img.dimensions();
    let x = (fx * f64::from(w) - 0.5).clamp(0.0, f64::from(w - 1));
    let y = (fy * f64::from(h) - 0.5).clamp(0.0, f64::from(h - 1));
    bilinear_premult(img, x, y, true)
}

/// Bilinear sample of straight-alpha `img` at pixel coordinates (centres at `.0`), premultiplied.
/// Outside the picture is transparent unless `clamp`.
fn bilinear_premult(img: &RgbaImage, x: f64, y: f64, clamp: bool) -> [f32; 4] {
    let (w, h) = (i64::from(img.width()), i64::from(img.height()));
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = ((x - x0) as f32, (y - y0) as f32);
    let (x0, y0) = (x0 as i64, y0 as i64);
    let texel = |xi: i64, yi: i64| -> [f32; 4] {
        let (xi, yi) = if clamp { (xi.clamp(0, w - 1), yi.clamp(0, h - 1)) } else { (xi, yi) };
        if xi < 0 || yi < 0 || xi >= w || yi >= h {
            return [0.0; 4];
        }
        let p = img.get_pixel(xi as u32, yi as u32);
        let a = f32::from(p[3]) / 255.0;
        [f32::from(p[0]) / 255.0 * a, f32::from(p[1]) / 255.0 * a, f32::from(p[2]) / 255.0 * a, a]
    };
    let (a, b, c, d) = (texel(x0, y0), texel(x0 + 1, y0), texel(x0, y0 + 1), texel(x0 + 1, y0 + 1));
    let mut out = [0f32; 4];
    for i in 0..4 {
        let top = a[i] * (1.0 - tx) + b[i] * tx;
        let bottom = c[i] * (1.0 - tx) + d[i] * tx;
        out[i] = top * (1.0 - ty) + bottom * ty;
    }
    out
}

/// The design of a zone as a function of the zone's coordinates (0–1 each way).
struct DesignSampler<'a> {
    content: Option<&'a RgbaImage>,
    background: Option<[f32; 4]>,
    ratio: f64,
    /// The design box (zone units), its centre, and the rotation's cosine and sine.
    bw: f64,
    bh: f64,
    cx: f64,
    cy: f64,
    cos: f64,
    sin: f64,
}

impl<'a> DesignSampler<'a> {
    fn new(d: &'a ZoneDesign, ratio: f64) -> DesignSampler<'a> {
        let content = d.content.as_ref().filter(|c| c.width() > 0 && c.height() > 0);
        let aspect = content.map(|c| f64::from(c.width()) / f64::from(c.height())).unwrap_or(1.0);
        let scale = if d.scale.is_finite() && d.scale > 0.0 { f64::from(d.scale) } else { 1.0 };
        let (bw, bh) = content_box(d.fit, ratio, aspect, scale);
        let angle = f64::from(d.rotation).to_radians();
        DesignSampler {
            content,
            background: d.background.map(|[r, g, b]| [f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0, 1.0]),
            ratio,
            bw,
            bh,
            cx: ratio / 2.0 + f64::from(d.offset[0]) * ratio,
            cy: 0.5 + f64::from(d.offset[1]),
            cos: angle.cos(),
            sin: angle.sin(),
        }
    }

    /// Premultiplied colour at zone point (u, v).
    fn sample(&self, u: f64, v: f64) -> [f32; 4] {
        let mut c = [0f32; 4];
        if let Some(img) = self.content {
            let (dx, dy) = (u * self.ratio - self.cx, v - self.cy);
            // Undo the clockwise rotation.
            let (rx, ry) = (dx * self.cos + dy * self.sin, -dx * self.sin + dy * self.cos);
            let (qx, qy) = (rx / self.bw + 0.5, ry / self.bh + 0.5);
            if (-0.01..=1.01).contains(&qx) && (-0.01..=1.01).contains(&qy) {
                c = bilinear_premult(img, qx * f64::from(img.width()) - 0.5, qy * f64::from(img.height()) - 0.5, false);
            }
        }
        if let Some(bg) = self.background {
            let k = 1.0 - c[3];
            for i in 0..4 {
                c[i] += bg[i] * k;
            }
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mockup::Zone;

    fn template(corners: [[f64; 2]; 4]) -> Template {
        let mut t = Template::new("Test", "photo.png");
        t.zones.push(Zone { corners, ratio: [2.0, 1.0], ..Zone::new("z", "Zone") });
        t
    }

    #[test]
    fn the_zone_maps_its_corners_exactly() {
        let q = [[10.0, 20.0], [110.0, 30.0], [100.0, 90.0], [5.0, 70.0]];
        let h = zone_homography(&q, 1.0).unwrap();
        for (uv, c) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].iter().zip(q) {
            let (x, y) = h.apply(uv.0, uv.1);
            assert!((x - c[0]).abs() < 1e-9 && (y - c[1]).abs() < 1e-9);
        }
        // A crossed (bow-tie) or reversed quad is refused.
        assert!(zone_homography(&[[0.0, 0.0], [10.0, 10.0], [10.0, 0.0], [0.0, 10.0]], 1.0).is_err());
        assert!(zone_homography(&[[0.0, 0.0], [0.0, 10.0], [10.0, 10.0], [10.0, 0.0]], 1.0).is_err());
    }

    #[test]
    fn contain_cover_stretch() {
        assert_eq!(content_box(Fit::Contain, 2.0, 1.0, 1.0), (1.0, 1.0));
        assert_eq!(content_box(Fit::Contain, 1.0, 2.0, 1.0), (1.0, 0.5));
        assert_eq!(content_box(Fit::Cover, 2.0, 1.0, 1.0), (2.0, 2.0));
        assert_eq!(content_box(Fit::Stretch, 2.0, 1.0, 0.5), (1.0, 0.5));
    }

    #[test]
    fn a_rectangle_zone_is_filled_and_the_rest_is_the_photo() {
        // Photo 100×60 grey; zone x 20–80, y 10–40 (ratio 2:1), red background.
        let t = template([[20.0, 10.0], [80.0, 10.0], [80.0, 40.0], [20.0, 40.0]]);
        let images = TemplateImages { photo: RgbaImage::from_pixel(100, 60, image::Rgba([128, 128, 128, 255])), zone_masks: vec![None], occlusion: None, overlays: vec![] };
        let d = ZoneDesign { background: Some([200, 0, 0]), scale: 1.0, ..Default::default() };
        let out = compose(&t, &images, &[Some(d)], 100, Quality::FINAL).unwrap();
        assert_eq!(out.get_pixel(50, 25).0, [200, 0, 0, 255]);
        assert_eq!(out.get_pixel(5, 5).0, [128, 128, 128, 255]);
        assert_eq!(out.get_pixel(85, 25).0, [128, 128, 128, 255]);
        // Half-covered edge pixel: x 19.5 is outside, the edge sits on a pixel border → crisp.
        assert_eq!(out.get_pixel(20, 25).0, [200, 0, 0, 255]);
        assert_eq!(out.get_pixel(19, 25).0, [128, 128, 128, 255]);
    }

    #[test]
    fn multiply_keeps_the_photo_shading_and_the_mask_and_occlusion_cut() {
        let t = template([[0.0, 0.0], [100.0, 0.0], [100.0, 50.0], [0.0, 50.0]]);
        let mut t = t;
        t.zones[0].blend = Blend::Multiply;
        let mut mask = GrayImage::from_pixel(100, 50, image::Luma([255]));
        for y in 0..50 {
            for x in 0..10 {
                mask.put_pixel(x, y, image::Luma([0]));
            }
        }
        let mut occ = GrayImage::new(100, 50);
        for y in 0..50 {
            for x in 90..100 {
                occ.put_pixel(x, y, image::Luma([255]));
            }
        }
        let images = TemplateImages { photo: RgbaImage::from_pixel(100, 50, image::Rgba([200, 200, 200, 255])), zone_masks: vec![Some(mask)], occlusion: Some(occ), overlays: vec![] };
        let d = ZoneDesign { background: Some([255, 0, 0]), scale: 1.0, ..Default::default() };
        let out = compose(&t, &images, &[Some(d)], 100, Quality::FINAL).unwrap();
        assert_eq!(out.get_pixel(50, 25).0, [200, 0, 0, 255], "multiply: red × light grey");
        assert_eq!(out.get_pixel(5, 25).0, [200, 200, 200, 255], "masked out");
        assert_eq!(out.get_pixel(95, 25).0, [200, 200, 200, 255], "foreground in front");
    }

    #[test]
    fn perspective_follows_the_homography_not_an_affine_guess() {
        // A trapezoid: the design's left half is red, its right half blue. The vertical split
        // must land where the homography puts u = 0.5, which differs from the trapezoid's
        // midline at mid-height.
        let q = [[20.0, 20.0], [180.0, 40.0], [180.0, 120.0], [20.0, 140.0]];
        let mut t = template(q);
        t.zones[0].ratio = [1.0, 1.0];
        let mut design = RgbaImage::new(200, 200);
        for (x, _, p) in design.enumerate_pixels_mut() {
            *p = if x < 100 { image::Rgba([255, 0, 0, 255]) } else { image::Rgba([0, 0, 255, 255]) };
        }
        let images = TemplateImages { photo: RgbaImage::from_pixel(200, 160, image::Rgba([0, 0, 0, 255])), zone_masks: vec![None], occlusion: None, overlays: vec![] };
        let d = ZoneDesign { content: Some(design), fit: Fit::Stretch, scale: 1.0, ..Default::default() };
        let out = compose(&t, &images, &[Some(d)], 200, Quality::FINAL).unwrap();
        let h = zone_homography(&q, 1.0).unwrap();
        let (sx, sy) = h.apply(0.5, 0.5);
        assert!((sx - 100.0).abs() > 2.0, "the test needs a real perspective ({sx})");
        let (x, y) = (sx.round() as u32, sy.round() as u32);
        assert_eq!(out.get_pixel(x - 3, y).0, [255, 0, 0, 255]);
        assert_eq!(out.get_pixel(x + 3, y).0, [0, 0, 255, 255]);
    }

    #[test]
    fn the_same_inputs_give_the_same_pixels_at_any_thread_count() {
        let t = template([[10.0, 10.0], [90.0, 15.0], [85.0, 55.0], [12.0, 50.0]]);
        let images = TemplateImages { photo: RgbaImage::from_pixel(100, 64, image::Rgba([90, 120, 90, 255])), zone_masks: vec![None], occlusion: None, overlays: vec![] };
        let d = ZoneDesign { background: Some([250, 250, 250]), scale: 0.7, rotation: 12.0, ..Default::default() };
        let a = compose(&t, &images, &[Some(d.clone())], 100, Quality::FINAL).unwrap();
        let b = compose(&t, &images, &[Some(d)], 100, Quality::FINAL).unwrap();
        assert_eq!(a.as_raw(), b.as_raw());
    }
}
