//! Oracle's look for the brand apps: the A structure (hairlines, square layout), Lexend, square controls
//! and Edit surfaces, each app's icon accent, light and dark. Shared by the four brand apps.

use std::sync::Arc;

use egui::epaint::text::VariationCoords;
use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak, Stroke,
    TextStyle, Visuals,
};

pub const RADIUS: CornerRadius = CornerRadius::ZERO;

#[derive(Clone, Copy, Default)]
pub enum Accent {
    #[default]
    Neutral,
    Assets,
    Font,
}

impl Accent {
    fn color(self, dark: bool) -> Color32 {
        match self {
            Self::Neutral if dark => Color32::from_rgb(0xf2, 0xf1, 0xee),
            Self::Neutral => Color32::from_rgb(0x1d, 0x1d, 0x1b),
            Self::Assets => Color32::from_rgb(0x0e, 0x8a, 0x6f),
            Self::Font => Color32::from_rgb(0x6d, 0x4a, 0xff),
        }
    }
}

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
    pub accent: Color32,
    pub on_accent: Color32,
    pub accent_ink: Color32,
    pub good: Color32,
    pub warn: Color32,
    pub bad: Color32,
}

pub const LIGHT: Tokens = Tokens {
    dark: false,
    bg: oracle_suite_style::Palette::LIGHT.void,
    panel: oracle_suite_style::Palette::LIGHT.ink,
    sunken: oracle_suite_style::Palette::LIGHT.void,
    ink: oracle_suite_style::Palette::LIGHT.text,
    muted: oracle_suite_style::Palette::LIGHT.text_dim,
    line: oracle_suite_style::Palette::LIGHT.line,
    accent: Color32::from_rgb(0x1d, 0x1d, 0x1b),
    on_accent: Color32::WHITE,
    accent_ink: Color32::from_rgb(0x1d, 0x1d, 0x1b),
    good: Color32::from_rgb(0x0a, 0x75, 0x63),
    warn: Color32::from_rgb(0x8a, 0x58, 0x00),
    bad: Color32::from_rgb(0xb3, 0x26, 0x1e),
};

pub const DARK: Tokens = Tokens {
    dark: true,
    bg: oracle_suite_style::Palette::DARK.void,
    panel: oracle_suite_style::Palette::DARK.ink,
    sunken: oracle_suite_style::Palette::DARK.void,
    ink: oracle_suite_style::Palette::DARK.text,
    muted: oracle_suite_style::Palette::DARK.text_dim,
    line: oracle_suite_style::Palette::DARK.line,
    accent: Color32::from_rgb(0xf2, 0xf1, 0xee),
    on_accent: Color32::from_rgb(0x1d, 0x1d, 0x1b),
    accent_ink: Color32::from_rgb(0xf2, 0xf1, 0xee),
    good: Color32::from_rgb(0x3f, 0xd6, 0xb8),
    warn: Color32::from_rgb(0xff, 0xc1, 0x4d),
    bad: Color32::from_rgb(0xff, 0x62, 0x59),
};

pub fn tokens(ctx: &egui::Context) -> Tokens {
    let accent = ctx
        .data(|d| d.get_temp::<Accent>(egui::Id::new("oracle-app-accent")))
        .unwrap_or_default();
    themed_tokens(ctx.theme() == egui::Theme::Dark, accent)
}

fn themed_tokens(dark: bool, accent: Accent) -> Tokens {
    let mut t = if dark { DARK } else { LIGHT };
    t.accent = accent.color(dark);
    t.accent_ink = t.accent;
    t.on_accent = oracle_suite_style::Palette::for_dark(dark)
        .with_accent(t.accent)
        .on_accent;
    t
}

pub fn apply(ctx: &egui::Context, choice: ThemeChoice, accent: Accent) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("oracle-app-accent"), accent));
    // Both looks are installed; egui follows the system when the choice is "Système".
    ctx.set_visuals_of(egui::Theme::Light, visuals(themed_tokens(false, accent)));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(themed_tokens(true, accent)));
    ctx.set_theme(match choice {
        ThemeChoice::System => egui::ThemePreference::System,
        ThemeChoice::Light => egui::ThemePreference::Light,
        ThemeChoice::Dark => egui::ThemePreference::Dark,
    });
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.interact_size.y = 30.0;
        s.text_styles.insert(
            TextStyle::Heading,
            FontId::new(26.0, FontFamily::Name("display".into())),
        );
        s.text_styles
            .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        s.text_styles.insert(
            TextStyle::Button,
            FontId::new(13.5, FontFamily::Name("medium".into())),
        );
        s.text_styles.insert(
            TextStyle::Small,
            FontId::new(11.5, FontFamily::Proportional),
        );
        s.spacing.slider_width = 120.0;
        s.spacing.slider_rail_height = 4.0;
        s.visuals.slider_trailing_fill = true;
        s.visuals.handle_shape = egui::style::HandleShape::Circle;
    });
    oracle_suite_style::chrome::finish(ctx, false);
}

fn visuals(t: Tokens) -> Visuals {
    let mut v = if t.dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.panel_fill = t.bg;
    v.window_fill = t.panel;
    v.extreme_bg_color = t.sunken;
    v.faint_bg_color = t.panel;
    v.window_corner_radius = CornerRadius::same(2);
    v.menu_corner_radius = RADIUS;
    v.window_stroke = Stroke::new(1.0, t.line);
    // Keep text contextual so selected buttons use selection.stroke for contrast.
    v.override_text_color = None;
    v.hyperlink_color = t.accent_ink;
    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::new(1.0, t.on_accent);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = RADIUS;
        w.fg_stroke.color = t.ink;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.line);
    v.widgets.inactive.weak_bg_fill = t.panel;
    v.widgets.inactive.bg_fill = t.panel;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.line);
    v.widgets.hovered.weak_bg_fill = t.sunken;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.muted);
    v.widgets.active.weak_bg_fill = t.sunken;
    v.widgets.active.bg_stroke = Stroke::new(1.0, t.accent);
    v
}

pub fn install_fonts(ctx: &egui::Context) {
    const INTER: &[u8] = include_bytes!("../assets/Inter-Variable.ttf");
    let mut fonts = FontDefinitions::default();
    let mut fallback: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let inter = FontTweak {
        coords: VariationCoords::new([(*b"wght", 400.0)]),
        ..Default::default()
    };
    fonts.font_data.insert(
        "Inter-Symbols".into(),
        Arc::new(FontData::from_static(INTER).tweak(inter)),
    );
    fallback.insert(0, "Inter-Symbols".into());
    for (name, wght, family) in [
        ("Inter", 400.0, FontFamily::Proportional),
        ("Inter-Medium", 500.0, FontFamily::Name("medium".into())),
        ("Inter-Display", 600.0, FontFamily::Name("display".into())),
    ] {
        let tweak = FontTweak {
            coords: VariationCoords::new([(*b"wght", wght)]),
            ..Default::default()
        };
        fonts.font_data.insert(
            name.into(),
            Arc::new(FontData::from_static(INTER).tweak(tweak)),
        );
        let mut stack = vec![name.to_string()];
        stack.extend(fallback.iter().cloned());
        fonts.families.insert(family, stack);
    }
    oracle_suite_style::fonts::add_lexend(&mut fonts);
    ctx.set_fonts(fonts);
}

/// An upper-case, letter-spaced label (the A structure's section labels).
pub fn label(ui: &mut egui::Ui, text: &str) {
    let t = tokens(ui.ctx());
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .size(11.0)
            .color(t.muted)
            .family(FontFamily::Name("medium".into()))
            .extra_letter_spacing(1.2),
    );
}

/// The primary button, filled with the app's accent.
pub fn primary(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let t = tokens(ui.ctx());
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .color(t.on_accent)
                .family(FontFamily::Name("medium".into())),
        )
        .fill(t.accent)
        .min_size(egui::vec2(0.0, 24.0)),
    )
}
