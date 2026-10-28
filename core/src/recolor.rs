//! Colour treatments of a logo for delivery: black, white, greyscale and inverted versions made
//! from the colour master, as Akrivi's Logo Exports and most agencies hand over. They are computed,
//! not drawn: a treatment the designer supplied as a master of its own (an asset of the same role
//! with that variant) is delivered instead, and a computed one is named as such in the guide.
//!
//! SVG masters are recoloured in their source (every colour value, gradients included, shapes
//! without a fill painted black by default); raster masters pixel by pixel, alpha kept.

use image::RgbaImage;
use serde::{Deserialize, Serialize};

use crate::model::Variant;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum Treatment {
    /// The master as drawn.
    #[default]
    Original,
    Black,
    White,
    Greyscale,
    /// For dark backgrounds: the near-black colours turn white, the others stay.
    Inverted,
}

impl Treatment {
    pub const ALL: [Treatment; 5] = [
        Treatment::Original,
        Treatment::Inverted,
        Treatment::Black,
        Treatment::White,
        Treatment::Greyscale,
    ];

    pub fn is_original(&self) -> bool {
        *self == Treatment::Original
    }

    pub fn label_fr(self) -> &'static str {
        match self {
            Treatment::Original => "Couleur",
            Treatment::Black => "Noir",
            Treatment::White => "Blanc",
            Treatment::Greyscale => "Niveaux de gris",
            Treatment::Inverted => "Inversé",
        }
    }

    /// For folder and file names.
    pub fn slug(self) -> &'static str {
        match self {
            Treatment::Original => "couleur",
            Treatment::Black => "noir",
            Treatment::White => "blanc",
            Treatment::Greyscale => "niveaux-de-gris",
            Treatment::Inverted => "inverse",
        }
    }

    /// What the client uses it for (the file guide).
    pub fn usage_fr(self) -> &'static str {
        match self {
            Treatment::Original => "la version de référence, sur fond clair",
            Treatment::Black => "impression en une couleur, tampons, gravure, fonds clairs",
            Treatment::White => "sur photo ou fond foncé, marquage en réserve",
            Treatment::Greyscale => "documents imprimés en noir et blanc",
            Treatment::Inverted => "sur fond foncé, en gardant les couleurs de la marque",
        }
    }

    /// The designer's variant that stands for this treatment, when they drew one.
    pub fn variant(self) -> Option<Variant> {
        match self {
            Treatment::Original => None,
            Treatment::Black => Some(Variant::Black),
            Treatment::White => Some(Variant::White),
            Treatment::Greyscale => Some(Variant::Monochrome),
            Treatment::Inverted => Some(Variant::Inverted),
        }
    }

    pub fn map(self, [r, g, b]: [u8; 3]) -> [u8; 3] {
        let l = luminance([r, g, b]);
        match self {
            Treatment::Original => [r, g, b],
            // White areas (knockouts drawn in white) stay white.
            Treatment::Black => {
                if l > 0.95 {
                    [255, 255, 255]
                } else {
                    [0, 0, 0]
                }
            }
            Treatment::White => [255, 255, 255],
            Treatment::Greyscale => {
                let y = (0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b))
                    .round()
                    .clamp(0.0, 255.0) as u8;
                [y, y, y]
            }
            // Only the near-blacks (ink, text): a saturated brand colour keeps its hue.
            Treatment::Inverted => {
                if l < 0.06 {
                    [255, 255, 255]
                } else {
                    [r, g, b]
                }
            }
        }
    }
}

/// WCAG relative luminance.
pub fn luminance(c: [u8; 3]) -> f64 {
    let lin = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// The SVG with every colour mapped by `t`: `#rgb`, `#rrggbb`, `rgb(…)` and the names black and
/// white, in attributes and styles alike. Shapes without a fill are black in SVG: the root gets an
/// explicit black fill first, so they follow the treatment too.
pub fn recolor_svg(svg: &str, t: Treatment) -> String {
    if t == Treatment::Original {
        return svg.to_string();
    }
    let svg = default_fill(svg);
    let mut out = String::with_capacity(svg.len());
    let chars: Vec<char> = svg.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '#' {
            let digits: String = chars[i + 1..]
                .iter()
                .take_while(|d| d.is_ascii_hexdigit())
                .collect();
            let rgb = match digits.len() {
                6 => u32::from_str_radix(&digits, 16)
                    .ok()
                    .map(|v| [(v >> 16) as u8, (v >> 8) as u8, v as u8]),
                3 => u32::from_str_radix(&digits, 16).ok().map(|v| {
                    let (r, g, b) = (
                        ((v >> 8) & 0xf) as u8,
                        ((v >> 4) & 0xf) as u8,
                        (v & 0xf) as u8,
                    );
                    [r * 17, g * 17, b * 17]
                }),
                _ => None,
            };
            if let Some(rgb) = rgb {
                out.push_str(&hex(t.map(rgb)));
                i += 1 + digits.len();
                continue;
            }
        }
        if (c == 'r' || c == 'R')
            && chars[i..]
                .iter()
                .take(4)
                .collect::<String>()
                .eq_ignore_ascii_case("rgb(")
            && let Some(end) = chars[i..].iter().position(|x| *x == ')')
        {
            {
                let inner: String = chars[i + 4..i + end].iter().collect();
                let parts: Vec<u8> = inner
                    .split(',')
                    .filter_map(|p| p.trim().parse::<f64>().ok())
                    .map(|v| v.clamp(0.0, 255.0) as u8)
                    .collect();
                if let [r, g, b] = parts[..] {
                    out.push_str(&hex(t.map([r, g, b])));
                    i += end + 1;
                    continue;
                }
            }
        }
        out.push(c);
        i += 1;
    }
    // Named colours, as attribute values and in styles.
    for (name, rgb) in [("black", [0u8, 0, 0]), ("white", [255, 255, 255])] {
        let to = hex(t.map(rgb));
        for (a, b) in [
            ("\"", "\""),
            ("'", "'"),
            (":", ";"),
            (":", "\""),
            (": ", ";"),
        ] {
            out = out.replace(&format!("{a}{name}{b}"), &format!("{a}{to}{b}"));
        }
    }
    out
}

/// Gives the root `<svg>` an explicit `fill="#000000"` when it has none, so shapes left with SVG's
/// default black are recoloured like the others.
fn default_fill(svg: &str) -> String {
    let Some(start) = svg.find("<svg") else {
        return svg.to_string();
    };
    let Some(len) = svg[start..].find('>') else {
        return svg.to_string();
    };
    let tag = &svg[start..start + len];
    if tag.contains(" fill=") {
        return svg.to_string();
    }
    let at = if tag.ends_with('/') {
        start + len - 1
    } else {
        start + len
    };
    format!("{} fill=\"#000000\"{}", &svg[..at], &svg[at..])
}

/// The image with every pixel's colour mapped by `t`, alpha kept.
pub fn recolor_image(img: &mut RgbaImage, t: Treatment) {
    if t == Treatment::Original {
        return;
    }
    for p in img.pixels_mut() {
        if p.0[3] == 0 {
            continue;
        }
        let [r, g, b] = t.map([p.0[0], p.0[1], p.0[2]]);
        p.0 = [r, g, b, p.0[3]];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect fill="#D31C3B" width="5" height="5"/><path d="M0 0h1v1z"/><circle style="fill:#1b1a17;stroke:rgb(255, 255, 255)" r="2"/><stop stop-color="#abc"/></svg>"##;

    #[test]
    fn black_turns_every_colour_black_but_keeps_white_knockouts() {
        let s = recolor_svg(LOGO, Treatment::Black);
        assert!(s.contains(r##"fill="#000000""##) && !s.contains("#D31C3B") && !s.contains("#abc"));
        assert!(s.contains("stroke:#ffffff"), "{s}");
        // The path without a fill follows the root's explicit black.
        assert!(s.starts_with(r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" fill="#000000">"##), "{s}");
    }

    #[test]
    fn white_greyscale_and_inverted() {
        let w = recolor_svg(LOGO, Treatment::White);
        assert!(w.contains(r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" fill="#ffffff">"##) && w.contains("fill:#ffffff"), "{w}");
        let g = recolor_svg(LOGO, Treatment::Greyscale);
        // D3 1C 3B → 0.2126·211 + 0.7152·28 + 0.0722·59 ≈ 69 (0x45).
        assert!(g.contains("#454545"), "{g}");
        let i = recolor_svg(LOGO, Treatment::Inverted);
        // The red stays, the near-black becomes white.
        assert!(i.contains("#d31c3b") && i.contains("fill:#ffffff"), "{i}");
        assert_eq!(recolor_svg(LOGO, Treatment::Original), LOGO);
    }

    #[test]
    fn a_recoloured_svg_still_parses_and_raster_pixels_keep_their_alpha() {
        for t in Treatment::ALL {
            assert!(
                resvg::usvg::Tree::from_str(
                    &recolor_svg(LOGO, t),
                    &resvg::usvg::Options::default()
                )
                .is_ok(),
                "{t:?}"
            );
        }
        let mut img = RgbaImage::from_pixel(2, 1, image::Rgba([211, 28, 59, 128]));
        img.put_pixel(1, 0, image::Rgba([0, 0, 0, 0]));
        recolor_image(&mut img, Treatment::White);
        assert_eq!(img.get_pixel(0, 0).0, [255, 255, 255, 128]);
        assert_eq!(img.get_pixel(1, 0).0, [0, 0, 0, 0]);
    }
}
