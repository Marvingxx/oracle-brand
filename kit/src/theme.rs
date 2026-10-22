//! Oracle's look: the A structure (hairlines, square layout), Lexend, a 4 px rounding, the brand
//! red, light and dark.

use std::sync::Arc;

use egui::epaint::text::VariationCoords;
use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak, Stroke, TextStyle, Visuals};

pub const RED: Color32 = Color32::from_rgb(0xd3, 0x1c, 0x3b);
pub const RADIUS: CornerRadius = CornerRadius::same(4);

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    pub fn label(self) -> &'static str {
        match self {
            ThemeChoice::System => "Système",
            ThemeChoice::Light => "Clair",
            ThemeChoice::Dark => "Sombre",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tokens {
    pub dark: bool,
    pub bg: Color32,
    pub panel: Color32,
    pub sunken: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub line: Color32,
    pub accent_ink: Color32,
    pub good: Color32,
    pub warn: Color32,
    pub bad: Color32,
}

pub const LIGHT: Tokens = Tokens {
    dark: false,
    bg: Color32::from_rgb(0xf2, 0xf1, 0xee),
    panel: Color32::from_rgb(0xf7, 0xf6, 0xf3),
    sunken: Color32::from_rgb(0xe6, 0xe4, 0xdf),
    ink: Color32::from_rgb(0x10, 0x10, 0x14),
    muted: Color32::from_rgb(0x5c, 0x5b, 0x57),
    line: Color32::from_rgba_premultiplied(16, 16, 20, 42),
    accent_ink: Color32::from_rgb(0xa3, 0x12, 0x2b),
    good: Color32::from_rgb(0x0a, 0x75, 0x63),
    warn: Color32::from_rgb(0x8a, 0x58, 0x00),
    bad: Color32::from_rgb(0xb3, 0x26, 0x1e),
};

pub const DARK: Tokens = Tokens {
    dark: true,
    bg: Color32::from_rgb(0x12, 0x13, 0x16),
    panel: Color32::from_rgb(0x18, 0x19, 0x1d),
    sunken: Color32::from_rgb(0x0a, 0x0a, 0x0c),
    ink: Color32::from_rgb(0xf2, 0xf1, 0xee),
    muted: Color32::from_rgb(0xa6, 0xa5, 0xa1),
    line: Color32::from_rgba_premultiplied(40, 40, 40, 40),
    accent_ink: Color32::from_rgb(0xf0, 0x56, 0x6f),
    good: Color32::from_rgb(0x3f, 0xd6, 0xb8),
    warn: Color32::from_rgb(0xff, 0xc1, 0x4d),
    bad: Color32::from_rgb(0xff, 0x62, 0x59),
};

pub fn tokens(ctx: &egui::Context) -> Tokens {
    if ctx.theme() == egui::Theme::Dark { DARK } else { LIGHT }
}

pub fn apply(ctx: &egui::Context, choice: ThemeChoice) {
    // Both looks are installed; egui follows the system when the choice is "Système".
    ctx.set_visuals_of(egui::Theme::Light, visuals(LIGHT));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(DARK));
    ctx.set_theme(match choice {
        ThemeChoice::System => egui::ThemePreference::System,
        ThemeChoice::Light => egui::ThemePreference::Light,
        ThemeChoice::Dark => egui::ThemePreference::Dark,
    });
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.interact_size.y = 30.0;
        s.text_styles.insert(TextStyle::Heading, FontId::new(26.0, FontFamily::Name("display".into())));
        s.text_styles.insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Button, FontId::new(13.5, FontFamily::Name("medium".into())));
        s.text_styles.insert(TextStyle::Small, FontId::new(11.5, FontFamily::Proportional));
    });
}

fn visuals(t: Tokens) -> Visuals {
    let mut v = if t.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.bg;
    v.window_fill = t.panel;
    v.extreme_bg_color = t.sunken;
    v.faint_bg_color = t.panel;
    v.window_corner_radius = CornerRadius::same(6);
    v.menu_corner_radius = RADIUS;
    v.window_stroke = Stroke::new(1.0, t.line);
    v.override_text_color = Some(t.ink);
    v.hyperlink_color = t.accent_ink;
    v.selection.bg_fill = RED;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = RADIUS;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.line);
    v.widgets.inactive.weak_bg_fill = t.panel;
    v.widgets.inactive.bg_fill = t.panel;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.line);
    v.widgets.hovered.weak_bg_fill = t.sunken;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.muted);
    v.widgets.active.weak_bg_fill = t.sunken;
    v
}

pub fn install_fonts(ctx: &egui::Context) {
    const LEXEND: &[u8] = include_bytes!("../assets/Lexend-Variable.ttf");
    const INTER: &[u8] = include_bytes!("../assets/Inter-Variable.ttf");
    let mut fonts = FontDefinitions::default();
    let mut fallback: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let inter = FontTweak { coords: VariationCoords::new([(*b"wght", 400.0)]), ..Default::default() };
    fonts.font_data.insert("Inter-Symbols".into(), Arc::new(FontData::from_static(INTER).tweak(inter)));
    fallback.insert(0, "Inter-Symbols".into());
    for (name, wght, family) in [
        ("Lexend", 400.0, FontFamily::Proportional),
        ("Lexend-Medium", 500.0, FontFamily::Name("medium".into())),
        ("Lexend-Display", 600.0, FontFamily::Name("display".into())),
    ] {
        let tweak = FontTweak { coords: VariationCoords::new([(*b"wght", wght)]), ..Default::default() };
        fonts.font_data.insert(name.into(), Arc::new(FontData::from_static(LEXEND).tweak(tweak)));
        let mut stack = vec![name.to_string()];
        stack.extend(fallback.iter().cloned());
        fonts.families.insert(family, stack);
    }
    ctx.set_fonts(fonts);
}

/// An upper-case, letter-spaced label (the A structure's section labels).
pub fn label(ui: &mut egui::Ui, text: &str) {
    let t = tokens(ui.ctx());
    ui.label(egui::RichText::new(text.to_uppercase()).size(11.0).color(t.muted).family(FontFamily::Name("medium".into())).extra_letter_spacing(1.2));
}

/// The primary (filled red) button.
pub fn primary(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(egui::Button::new(egui::RichText::new(text).color(Color32::WHITE).family(FontFamily::Name("medium".into()))).fill(RED).min_size(egui::vec2(0.0, 34.0)))
}
