//! Non-confidential test data for Mockup Studio: a drawn "photo" of a business card lying at an
//! angle on a table, with a pen across one corner, and the pen's foreground mask. Deterministic
//! (no randomness), so tests and the demo always get the same pixels.

use image::{GrayImage, RgbaImage};

/// The card's corners on the photo (top-left, top-right, bottom-right, bottom-left).
pub const CARD: [[f64; 2]; 4] = [[430.0, 250.0], [1190.0, 330.0], [1120.0, 760.0], [330.0, 660.0]];
pub const SIZE: (u32, u32) = (1600, 1000);
/// A point of the card (zone coordinates) the pen lies over.
pub const PEN_UV: (f64, f64) = (0.85, 0.62);
const PEN_HALF_WIDTH: f64 = 15.0;
const PEN_HALF_LENGTH: f64 = 330.0;

/// The photo and the pen's mask (white = in front of the card).
pub fn business_card() -> (RgbaImage, GrayImage) {
    let (w, h) = SIZE;
    let card = CARD;
    let shadow = card.map(|[x, y]| [x + 16.0, y + 24.0]);
    let (a, b) = pen_segment();
    let mut photo = RgbaImage::new(w, h);
    let mut mask = GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            // Table: a soft green-grey with grain and a vignette.
            let n = noise(x, y) * 6.0;
            let vignette = 1.0 - 0.18 * (((px / f64::from(w) - 0.5).powi(2) + (py / f64::from(h) - 0.5).powi(2)) * 2.0);
            let mut c = [(176.0 + n) * vignette, (190.0 + n) * vignette, (168.0 + n) * vignette];
            // The card's shadow, soft.
            let s = (inside_distance(&shadow, px, py) / 22.0 + 0.5).clamp(0.0, 1.0) * 0.32;
            c = c.map(|v| v * (1.0 - s));
            // The card: off-white paper, lighter at the top, with a little grain.
            let cov = (inside_distance(&card, px, py) + 0.5).clamp(0.0, 1.0);
            let paper = 236.0 + 8.0 * (1.0 - py / f64::from(h)) + noise(x + 7, y + 3) * 2.0;
            let paper = [paper, paper - 1.0, paper - 5.0];
            for i in 0..3 {
                c[i] = c[i] * (1.0 - cov) + paper[i] * cov;
            }
            // The pen, in front.
            let pen = (PEN_HALF_WIDTH - segment_distance(a, b, px, py) + 0.5).clamp(0.0, 1.0);
            if pen > 0.0 {
                let across = segment_side(a, b, px, py) / PEN_HALF_WIDTH;
                let shade = 34.0 + 40.0 * (1.0 - (across - 0.35).abs()).max(0.0).powi(4);
                for v in c.iter_mut() {
                    *v = *v * (1.0 - pen) + shade * pen;
                }
                mask.put_pixel(x, y, image::Luma([(pen * 255.0).round() as u8]));
            }
            photo.put_pixel(x, y, image::Rgba([c[0].round().clamp(0.0, 255.0) as u8, c[1].round().clamp(0.0, 255.0) as u8, c[2].round().clamp(0.0, 255.0) as u8, 255]));
        }
    }
    (photo, mask)
}

/// The pen: a segment through the card point [`PEN_UV`].
pub fn pen_segment() -> ([f64; 2], [f64; 2]) {
    let p = card_point(PEN_UV.0, PEN_UV.1);
    let d = [std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2];
    ([p[0] - d[0] * PEN_HALF_LENGTH, p[1] - d[1] * PEN_HALF_LENGTH], [p[0] + d[0] * PEN_HALF_LENGTH, p[1] + d[1] * PEN_HALF_LENGTH])
}

/// A point of the card, from zone coordinates (0–1 each way), on the photo.
pub fn card_point(u: f64, v: f64) -> [f64; 2] {
    match crate::composite::zone_homography(&CARD, 1.0) {
        Ok(h) => {
            let (x, y) = h.apply(u, v);
            [x, y]
        }
        Err(_) => [0.0, 0.0],
    }
}

/// Whether the photo point is under the pen (with a margin).
pub fn under_pen(x: f64, y: f64, margin: f64) -> bool {
    let (a, b) = pen_segment();
    segment_distance(a, b, x, y) < PEN_HALF_WIDTH + margin
}

fn noise(x: u32, y: u32) -> f64 {
    let mut v = x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263);
    v = (v ^ (v >> 13)).wrapping_mul(1_274_126_177);
    f64::from(v ^ (v >> 16)) / f64::from(u32::MAX) * 2.0 - 1.0
}

/// Signed distance to a convex clockwise polygon's edges: positive inside (approximate outside).
fn inside_distance(q: &[[f64; 2]; 4], x: f64, y: f64) -> f64 {
    (0..4)
        .map(|i| {
            let (a, b) = (q[i], q[(i + 1) % 4]);
            let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
            let len = ex.hypot(ey);
            -(((x - a[0]) * ey - (y - a[1]) * ex) / len)
        })
        .fold(f64::MAX, f64::min)
}

fn segment_distance(a: [f64; 2], b: [f64; 2], x: f64, y: f64) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let t = (((x - a[0]) * dx + (y - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    (x - a[0] - t * dx).hypot(y - a[1] - t * dy)
}

fn segment_side(a: [f64; 2], b: [f64; 2], x: f64, y: f64) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    ((x - a[0]) * dy - (y - a[1]) * dx) / dx.hypot(dy)
}
