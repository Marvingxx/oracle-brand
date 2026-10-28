//! The centre: the preview with the zones drawn over it. In a template, the selected zone's four
//! corners are dragged into place (arrow keys nudge the last one by a pixel, Maj by ten); in a
//! scene, dragging inside a zone moves its design. The wheel zooms about the pointer; dragging the
//! background (or with the middle button) pans.

use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Vec2, pos2, vec2};
use oracle_brand::composite::zone_homography;
use oracle_brand::mockup::Content;

use crate::{App, Mode, theme};

#[derive(Clone, Copy, Debug)]
pub struct View {
    /// 1 = the photo fits the canvas.
    pub zoom: f32,
    pub pan: Vec2,
    /// Photo pixels → screen: `origin + p × scale`, as last drawn.
    origin: Pos2,
    scale: f32,
    drag: Drag,
    /// The corner the arrow keys move.
    pub corner: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Drag {
    None,
    Corner(usize),
    Design { last: Pos2 },
    Pan,
}

impl Default for View {
    fn default() -> View {
        View { zoom: 1.0, pan: Vec2::ZERO, origin: Pos2::ZERO, scale: 1.0, drag: Drag::None, corner: None }
    }
}

impl View {
    fn screen_of(&self, p: [f64; 2]) -> Pos2 {
        self.origin + vec2(p[0] as f32, p[1] as f32) * self.scale
    }

    fn photo_of(&self, s: Pos2) -> [f64; 2] {
        let p = (s - self.origin) / self.scale.max(1e-6);
        [f64::from(p.x), f64::from(p.y)]
    }

    /// The topmost zone under a screen point.
    pub fn zone_at(&self, mode: &Mode, s: Pos2) -> Option<usize> {
        let (_, t) = mode.template()?;
        let p = self.photo_of(s);
        t.zones.iter().enumerate().rev().find(|(_, z)| inside(&z.corners, p)).map(|(i, _)| i)
    }
}

fn inside(q: &[[f64; 2]; 4], p: [f64; 2]) -> bool {
    let mut sign = 0.0;
    for i in 0..4 {
        let (a, b) = (q[i], q[(i + 1) % 4]);
        let c = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        if c != 0.0 {
            if sign != 0.0 && (c > 0.0) != (sign > 0.0) {
                return false;
            }
            sign = c;
        }
    }
    true
}

pub fn canvas(app: &mut App, ui: &mut egui::Ui) {
    let t = theme::tokens(ui.ctx());
    let rect = ui.max_rect();
    let Some((_, template)) = app.mode.template() else {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect.shrink(40.0)), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(rect.height() * 0.3);
                ui.label(egui::RichText::new("Choisissez une scène ou un modèle à gauche.").color(t.muted).size(15.0));
                ui.label(egui::RichText::new("Pour un nouveau modèle, déposez ici une photo de support (depuis la Bibliothèque ou l'Explorateur).").color(t.muted));
            });
        });
        return;
    };
    let zones: Vec<[[f64; 2]; 4]> = template.zones.iter().map(|z| z.corners).collect();
    let names: Vec<String> = template.zones.iter().map(|z| z.name.clone()).collect();
    let preview = app.preview.as_ref();
    let Some([pw, ph]) = preview.and_then(|p| p.photo_size) else {
        let msg = match preview.and_then(|p| p.error.clone()) {
            Some(e) => e,
            None => "Chargement de la photo…".into(),
        };
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect.shrink(40.0)), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(rect.height() * 0.35);
                ui.label(egui::RichText::new(msg).color(t.muted));
            });
        });
        return;
    };
    let (pw, ph) = (pw as f32, ph as f32);
    let fit = ((rect.width() - 48.0) / pw).min((rect.height() - 48.0) / ph).max(0.01);
    let scale = fit * app.view.zoom;
    let origin = rect.center() - vec2(pw, ph) * scale / 2.0 + app.view.pan;
    app.view.origin = origin;
    app.view.scale = scale;

    let response = ui.interact(rect, ui.id().with("mockup-canvas"), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    let photo_rect = Rect::from_min_size(origin, vec2(pw, ph) * scale);
    if let Some(tex) = preview.and_then(|p| p.texture.as_ref()) {
        painter.image(tex.id(), photo_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
    painter.rect_stroke(photo_rect, 0.0, Stroke::new(1.0, t.line), egui::StrokeKind::Outside);

    let editing_template = matches!(app.mode, Mode::Template { .. });
    // Zones.
    for (i, q) in zones.iter().enumerate() {
        let pts: Vec<Pos2> = q.iter().map(|p| app.view.screen_of(*p)).collect();
        let selected = i == app.zone;
        let valid = zone_homography(q, 1.0).is_ok();
        let color = if !valid { t.bad } else if selected { t.accent } else { Color32::from_white_alpha(200) };
        if editing_template {
            let fill = if selected { t.accent.gamma_multiply(0.18) } else { Color32::from_white_alpha(28) };
            painter.add(Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
        }
        if editing_template || selected {
            let stroke = Stroke::new(if selected { 1.5 } else { 1.0 }, color);
            painter.add(Shape::closed_line(pts.clone(), stroke));
        }
        if editing_template {
            let c = pts.iter().fold(Vec2::ZERO, |a, p| a + p.to_vec2()) / 4.0;
            painter.text(c.to_pos2(), egui::Align2::CENTER_CENTER, &names[i], egui::FontId::new(12.0, egui::FontFamily::Name("medium".into())), color);
        }
        if editing_template && selected {
            for (k, p) in pts.iter().enumerate() {
                let active = app.view.corner == Some(k);
                painter.circle(*p, if active { 7.0 } else { 6.0 }, Color32::WHITE, Stroke::new(2.0, color));
            }
        }
    }
    if let Some(e) = preview.and_then(|p| p.error.clone()) {
        painter.text(rect.left_top() + vec2(16.0, 14.0), egui::Align2::LEFT_TOP, e, egui::FontId::proportional(13.0), t.bad);
    } else if preview.is_some_and(|p| p.busy()) {
        painter.text(rect.right_top() + vec2(-16.0, 14.0), egui::Align2::RIGHT_TOP, "Aperçu…", egui::FontId::proportional(12.0), t.muted);
    }

    // Zoom about the pointer.
    if let Some(hover) = response.hover_pos() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y + i.zoom_delta().ln() * 200.0);
        if scroll.abs() > 0.1 {
            let before = app.view.photo_of(hover);
            let factor = (scroll / 300.0).exp();
            app.view.zoom = (app.view.zoom * factor).clamp(0.2, 16.0);
            let new_scale = fit * app.view.zoom;
            let new_origin = rect.center() - vec2(pw, ph) * new_scale / 2.0 + app.view.pan;
            let after = new_origin + vec2(before[0] as f32, before[1] as f32) * new_scale;
            app.view.pan += hover - after;
        }
    }

    // Drags.
    if response.drag_started() {
        let pos = response.interact_pointer_pos().unwrap_or_default();
        let middle = ui.input(|i| i.pointer.middle_down());
        app.view.drag = Drag::Pan;
        if !middle {
            if editing_template {
                let near = zones.get(app.zone).and_then(|q| q.iter().map(|p| app.view.screen_of(*p).distance(pos)).enumerate().filter(|(_, d)| *d < 12.0).min_by(|a, b| a.1.total_cmp(&b.1)));
                if let Some((k, _)) = near {
                    app.view.drag = Drag::Corner(k);
                    app.view.corner = Some(k);
                } else if let Some(z) = app.view.zone_at(&app.mode, pos) {
                    app.zone = z;
                    app.view.corner = None;
                }
            } else if let Some(z) = app.view.zone_at(&app.mode, pos) {
                app.zone = z;
                app.view.drag = Drag::Design { last: pos };
            }
        }
    }
    if response.dragged() {
        let pos = response.interact_pointer_pos().unwrap_or_default();
        match app.view.drag {
            Drag::Pan => app.view.pan += response.drag_delta(),
            Drag::Corner(k) => {
                let p = app.view.photo_of(pos);
                let p = [p[0].clamp(-f64::from(pw), 2.0 * f64::from(pw)).round(), p[1].clamp(-f64::from(ph), 2.0 * f64::from(ph)).round()];
                let zone = app.zone;
                if let Mode::Template { template, .. } = &mut app.mode
                    && let Some(z) = template.zones.get_mut(zone)
                    && z.corners[k] != p
                {
                    z.corners[k] = p;
                    app.touch();
                }
            }
            Drag::Design { last } => {
                let zone = app.zone;
                let (a, b) = (app.view.photo_of(last), app.view.photo_of(pos));
                if let Mode::Scene { scene, template, .. } = &mut app.mode
                    && let Some(z) = template.zones.get(zone)
                    && let Ok(h) = zone_homography(&z.corners, 1.0)
                    && let Some(inv) = h.inverse()
                {
                    let (ua, va) = inv.apply(a[0], a[1]);
                    let (ub, vb) = inv.apply(b[0], b[1]);
                    let fill = scene.fill_mut(&z.id);
                    if !matches!(fill.content, Content::None) {
                        fill.offset[0] += (ub - ua) as f32;
                        fill.offset[1] += (vb - va) as f32;
                        app.touch();
                    }
                }
                app.view.drag = Drag::Design { last: pos };
            }
            Drag::None => {}
        }
    }
    if response.drag_stopped() {
        app.view.drag = Drag::None;
    }
    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
        && let Some(z) = app.view.zone_at(&app.mode, pos)
    {
        app.zone = z;
    }
    if response.double_clicked() {
        app.view.zoom = 1.0;
        app.view.pan = Vec2::ZERO;
    }

    // Brand Kit logos dropped from the inspector onto a zone.
    if let Some(payload) = response.dnd_release_payload::<crate::inspector::AssetPayload>() {
        let pos = ui.input(|i| i.pointer.latest_pos());
        if let Some(z) = pos.and_then(|p| app.view.zone_at(&app.mode, p)) {
            app.zone = z;
        }
        crate::inspector::set_content_asset(app, &payload.0);
    }

    // Arrow keys nudge the active corner.
    if editing_template && let Some(k) = app.view.corner {
        let (dx, dy, step) = ui.input(|i| {
            let step = if i.modifiers.shift { 10.0 } else { 1.0 };
            let dx = f64::from(i.key_pressed(egui::Key::ArrowRight) as u8) - f64::from(i.key_pressed(egui::Key::ArrowLeft) as u8);
            let dy = f64::from(i.key_pressed(egui::Key::ArrowDown) as u8) - f64::from(i.key_pressed(egui::Key::ArrowUp) as u8);
            (dx, dy, step)
        });
        if (dx != 0.0 || dy != 0.0) && ui.memory(|m| m.focused().is_none()) {
            let zone = app.zone;
            if let Mode::Template { template, .. } = &mut app.mode
                && let Some(z) = template.zones.get_mut(zone)
            {
                z.corners[k][0] += dx * step;
                z.corners[k][1] += dy * step;
                app.touch();
            }
        }
    }

    // Zoom buttons, bottom right.
    let bar = Rect::from_min_size(rect.right_bottom() + vec2(-206.0, -44.0), vec2(190.0, 30.0));
    ui.scope_builder(egui::UiBuilder::new().max_rect(bar), |ui| {
        ui.horizontal(|ui| {
            if ui.small_button("−").on_hover_text("Dézoomer").clicked() {
                app.view.zoom = (app.view.zoom / 1.25).max(0.2);
            }
            ui.label(egui::RichText::new(format!("{:.0} %", scale * 100.0)).size(12.0));
            if ui.small_button("+").on_hover_text("Zoomer").clicked() {
                app.view.zoom = (app.view.zoom * 1.25).min(16.0);
            }
            if ui.small_button("Ajuster").clicked() {
                app.view.zoom = 1.0;
                app.view.pan = Vec2::ZERO;
            }
        });
    });
}
