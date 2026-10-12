//! Cavalry-inspired neutrals and a 4/8/16/24 spacing scale. Colors are per-app, not globals.
use crate::settings::{Settings, Theme};
use egui::{Color32, FontId, RichText, Vec2};
pub const XS: f32 = 4.0;
pub const SM: f32 = 8.0;
pub const MD: f32 = 16.0;
pub const LG: f32 = 24.0;
pub const CONTROL: f32 = 36.0;
pub const ROW: f32 = 64.0;
pub const HEADER: f32 = 56.0;
pub const RADIUS: u8 = 8;
pub const CARD_RADIUS: u8 = 12;
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub bg: Color32,
    pub panel: Color32,
    pub surface: Color32,
    pub raised: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub border: Color32,
    /// Decorative dividers, not text/control focus boundaries.
    pub line: Color32,
    pub selected: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub accent_text: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub focus: Color32,
    pub dark: bool,
}
pub fn rgb(v: [u8; 3]) -> Color32 {
    Color32::from_rgb(v[0], v[1], v[2])
}
pub fn mix(a: Color32, b: Color32, k: f32) -> Color32 {
    rgb(std::array::from_fn(|i| {
        (a[i] as f32 * (1.0_f32 - k) + b[i] as f32 * k).round() as u8
    }))
}
pub fn luminance(c: Color32) -> f32 {
    let linear = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
}
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}
pub fn ink(bg: Color32) -> Color32 {
    if contrast(Color32::WHITE, bg) > contrast(Color32::BLACK, bg) {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}
fn readable(c: Color32, bg: Color32, ratio: f32) -> Color32 {
    let target = ink(bg);
    for i in 0..=20 {
        let candidate = mix(c, target, i as f32 / 20.0);
        if contrast(candidate, bg) >= ratio {
            return candidate;
        }
    }
    target
}
pub fn palette(s: &Settings) -> Palette {
    let (bg, accent) = match s.theme {
        Theme::Cavalry => (rgb([24, 25, 28]), rgb([110, 216, 164])),
        Theme::Light => (rgb([242, 244, 247]), rgb([25, 112, 75])),
        Theme::Custom => (rgb(s.background), rgb(s.accent)),
    };
    let global_ink = ink(bg);
    let dark = global_ink == Color32::WHITE;
    let towards = global_ink;
    let safe_surface = |amount| {
        let candidate = mix(bg, towards, amount);
        if contrast(global_ink, candidate) >= 4.6 {
            candidate
        } else {
            bg
        }
    };
    // Presets have deliberately different elevation directions: light cards are white,
    // not a darkened version of the canvas. Custom colors retain contrast-safe surfaces.
    let (panel, surface, raised, selected) = match s.theme {
        Theme::Cavalry => (
            rgb([33, 35, 39]),
            rgb([42, 45, 50]),
            rgb([52, 56, 62]),
            rgb([35, 57, 47]),
        ),
        Theme::Light => (
            Color32::WHITE,
            rgb([243, 245, 247]),
            rgb([231, 235, 239]),
            rgb([226, 241, 232]),
        ),
        Theme::Custom => (
            safe_surface(0.020),
            safe_surface(0.045),
            safe_surface(0.075),
            safe_surface(0.065),
        ),
    };
    let readable_all = |color: Color32, minimum: f32| {
        for step in 0..=40 {
            let candidate = mix(color, global_ink, step as f32 / 40.0);
            if [bg, panel, surface, raised, selected]
                .iter()
                .all(|base| contrast(candidate, *base) >= minimum)
            {
                return candidate;
            }
        }
        global_ink
    };
    let text = readable_all(
        if dark {
            rgb([240, 242, 243])
        } else {
            rgb([29, 32, 35])
        },
        7.0,
    );
    let muted = readable_all(mix(text, raised, 0.43), 4.5);
    Palette {
        bg,
        panel,
        surface,
        raised,
        text,
        muted,
        border: readable(mix(text, surface, 0.72), surface, 3.0),
        line: mix(text, surface, if dark { 0.90 } else { 0.86 }),
        selected,
        accent,
        on_accent: ink(accent),
        accent_text: readable_all(accent, 4.5),
        danger: readable_all(
            if dark {
                rgb([255, 151, 151])
            } else {
                rgb([177, 33, 50])
            },
            4.5,
        ),
        success: readable_all(
            if dark {
                rgb([73, 220, 147])
            } else {
                rgb([0, 115, 66])
            },
            4.5,
        ),
        warning: readable_all(rgb([218, 180, 69]), 4.5),
        focus: readable_all(accent, 3.0),
        dark,
    }
}
pub fn scrollbar_opacity(p: Palette) -> f32 {
    (10..=20)
        .map(|i| i as f32 / 20.0)
        .find(|alpha| {
            [p.bg, p.panel, p.surface, p.raised]
                .iter()
                .all(|bg| contrast(mix(*bg, p.text, *alpha), *bg) >= 3.0)
        })
        .unwrap_or(1.0)
}
pub fn font(size: f32) -> FontId {
    FontId::proportional(size)
}
pub fn medium(size: f32) -> FontId {
    FontId::new(
        size,
        egui::FontFamily::Name(super::fonts::FAMILY_MEDIUM.into()),
    )
}
pub fn text(label: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(label).font(font(size)).color(color)
}
pub fn heading(label: impl Into<String>, size: f32, p: Palette) -> RichText {
    RichText::new(label).font(medium(size)).color(p.text)
}
pub fn apply(ctx: &egui::Context) {
    apply_settings(ctx, &Settings::default());
}
pub fn apply_settings(ctx: &egui::Context, s: &Settings) {
    let p = palette(s);
    // Select the egui style bank explicitly before editing it. Native theme notifications
    // must not silently replace custom colors with the default system light/dark style.
    ctx.set_theme(if p.dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    let mut style = (*ctx.style()).clone();
    style.visuals = if p.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    style.visuals.override_text_color = None;
    style.visuals.panel_fill = p.bg;
    style.visuals.window_fill = p.panel;
    style.visuals.window_stroke = egui::Stroke::new(1.0_f32, p.line);
    style.visuals.window_corner_radius = egui::CornerRadius::same(CARD_RADIUS);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(RADIUS);
    style.visuals.extreme_bg_color = p.bg;
    style.visuals.faint_bg_color = p.panel;
    style.visuals.hyperlink_color = p.accent_text;
    style.visuals.selection.bg_fill = p.accent;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, p.on_accent);
    style.visuals.warn_fg_color = p.warning;
    style.visuals.error_fg_color = p.danger;
    for widget in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.bg_fill = p.surface;
        widget.weak_bg_fill = p.surface;
        widget.bg_stroke = egui::Stroke::new(1.0_f32, p.line);
        widget.fg_stroke = egui::Stroke::new(1.0_f32, p.text);
        // Checkbox marks are smaller than action buttons: a radius of 8 turns them into circles.
        widget.corner_radius = egui::CornerRadius::same(3);
        // A hovered stock widget must not grow outside our fixed rounded bounds.
        widget.expansion = 0.0;
    }
    style.visuals.widgets.hovered.bg_fill = p.raised;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, p.focus);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0_f32, p.focus);
    style.spacing.item_spacing = Vec2::splat(SM);
    style.spacing.button_padding = Vec2::new(MD, SM);
    style.spacing.interact_size = Vec2::new(40.0, CONTROL);
    style.spacing.scroll = egui::style::ScrollStyle::floating();
    // Reserve the full gutter even when the handle is thin; never cover row actions.
    style.spacing.scroll.floating_allocated_width = 12.0;
    style.spacing.scroll.floating_width = 6.0;
    let handle_opacity = scrollbar_opacity(p);
    style.spacing.scroll.dormant_handle_opacity = handle_opacity;
    style.spacing.scroll.active_handle_opacity = (handle_opacity + 0.15).min(1.0);
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.foreground_color = true;
    style.visuals.widgets.noninteractive.fg_stroke.color = p.text;
    style.spacing.scroll.handle_min_length = 40.0;
    style.animation_time = if s.reduce_motion { 0.0 } else { 0.14 };
    style.text_styles.insert(egui::TextStyle::Body, font(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, medium(14.0));
    style.text_styles.insert(egui::TextStyle::Small, font(12.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, medium(22.0));
    ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(if p.dark {
        egui::SystemTheme::Dark
    } else {
        egui::SystemTheme::Light
    }));
    ctx.set_style(style);
    ctx.set_zoom_factor(s.scale_percent as f32 / 100.0);
}
#[derive(Clone, Copy)]
pub enum Button {
    Primary,
    Secondary,
    Danger,
    Quiet,
    QuietDanger,
}
/// Finite state animation; no idle timers or layout transforms. All custom controls
/// use the same reduced-motion setting as stock egui widgets.
pub fn motion(ui: &egui::Ui, id: egui::Id, active: bool) -> f32 {
    ui.ctx().animate_bool_with_time_and_easing(
        id,
        active,
        ui.style().animation_time,
        egui::emath::easing::cubic_out,
    )
}
pub fn button(ui: &mut egui::Ui, label: &str, kind: Button, p: Palette) -> egui::Response {
    let primary = matches!(kind, Button::Primary);
    let danger = matches!(kind, Button::Danger | Button::QuietDanger);
    let ink = if primary {
        p.on_accent
    } else if danger {
        p.danger
    } else if matches!(kind, Button::Quiet) {
        p.muted
    } else {
        p.text
    };
    let galley = ui.painter().layout(
        label.into(),
        medium(14.0),
        ink,
        (ui.available_width() - 2.0 * MD).max(32.0),
    );
    let size = Vec2::new(
        (galley.size().x + 2.0 * MD).max(64.0),
        (galley.size().y + 2.0 * SM).max(CONTROL),
    );
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    let hover = motion(
        ui,
        response.id.with("hover"),
        ui.is_enabled() && (response.hovered() || response.has_focus()),
    );
    let press = motion(
        ui,
        response.id.with("press"),
        ui.is_enabled() && response.is_pointer_button_down_on(),
    );
    let fill = if primary {
        let toward = if p.on_accent == Color32::WHITE {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        mix(p.accent, toward, hover * 0.06 + press * 0.08)
    } else {
        match kind {
            Button::Quiet | Button::QuietDanger => p.raised.gamma_multiply(hover),
            _ => mix(p.surface, p.raised, (hover * 0.65 + press * 0.35).min(1.0)),
        }
    };
    let border = if response.has_focus() {
        egui::Stroke::new(2.0_f32, p.focus)
    } else if danger && !matches!(kind, Button::QuietDanger) {
        egui::Stroke::new(1.0_f32, p.danger)
    } else if matches!(kind, Button::Secondary) {
        egui::Stroke::new(1.0_f32, p.line)
    } else {
        egui::Stroke::NONE
    };
    ui.painter()
        .rect(rect, RADIUS, fill, border, egui::StrokeKind::Inside);
    ui.painter().galley(
        rect.center() - galley.size() / 2.0 + Vec2::new(0.0, press),
        galley,
        ink,
    );
    if ui.is_enabled() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}
/// Small square inputs must not inherit the large button radius or low-contrast divider.
pub fn checkbox(ui: &mut egui::Ui, value: &mut bool, label: &str, p: Palette) -> egui::Response {
    ui.scope(|ui| {
        let widget = &mut ui.visuals_mut().widgets.inactive;
        widget.bg_stroke = egui::Stroke::new(1.0_f32, p.border);
        widget.corner_radius = egui::CornerRadius::same(3);
        ui.checkbox(value, label)
    })
    .inner
}
pub fn card(p: Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(p.panel)
        .corner_radius(CARD_RADIUS)
        .inner_margin(MD)
}
pub fn modal<R>(
    ctx: &egui::Context,
    id: &str,
    p: Palette,
    width: f32,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::ModalResponse<R> {
    egui::Modal::new(egui::Id::new(id))
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .stroke(egui::Stroke::new(1.0_f32, p.line))
                .shadow(egui::Shadow {
                    offset: [0, 12],
                    blur: 40,
                    spread: 2,
                    color: Color32::from_black_alpha(80),
                })
                .corner_radius(12)
                .inner_margin(LG),
        )
        .show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 96.0).clamp(260.0, width));
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            contents(ui)
        })
}
pub fn footer<R>(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.add_space(MD);
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), CONTROL),
        egui::Layout::right_to_left(egui::Align::Center),
        contents,
    )
    .inner
}
pub fn label_truncated(
    ui: &egui::Ui,
    label: &str,
    font: FontId,
    color: Color32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(label.to_owned(), font, color);
    job.wrap.max_width = width.max(8.0);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    ui.painter().layout_job(job)
}

pub fn icon_button(
    ui: &mut egui::Ui,
    label: &str,
    icon: super::icons::Icon,
    danger: bool,
    p: Palette,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(CONTROL), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    let hover = motion(
        ui,
        response.id.with("hover"),
        ui.is_enabled() && (response.hovered() || response.has_focus()),
    );
    let press = motion(
        ui,
        response.id.with("press"),
        ui.is_enabled() && response.is_pointer_button_down_on(),
    );
    let color = mix(p.muted, if danger { p.danger } else { p.text }, hover);
    ui.painter()
        .rect_filled(rect, RADIUS, p.raised.gamma_multiply(hover));
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            RADIUS,
            egui::Stroke::new(2.0_f32, p.focus),
            egui::StrokeKind::Inside,
        );
    }
    super::icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(rect.center() + Vec2::new(0.0, press), Vec2::splat(18.0)),
        icon,
        color,
    );
    let response = response.on_hover_text(label);
    if ui.is_enabled() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}
pub fn nav_button(
    ui: &mut egui::Ui,
    label: &str,
    icon: super::icons::Icon,
    selection: Option<bool>,
    badge: Option<usize>,
    p: Palette,
) -> egui::Response {
    let selected = selection.unwrap_or(false);
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), CONTROL),
        egui::Sense::click(),
    );
    response.widget_info(|| {
        if selection.is_some() {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                ui.is_enabled(),
                selected,
                label,
            )
        } else {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
        }
    });
    let hover = motion(
        ui,
        response.id.with("hover"),
        ui.is_enabled() && (response.hovered() || response.has_focus()),
    );
    if selected {
        ui.painter().rect_filled(rect, RADIUS, p.selected);
    } else {
        ui.painter()
            .rect_filled(rect, RADIUS, p.surface.gamma_multiply(hover));
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            RADIUS,
            egui::Stroke::new(2.0_f32, p.focus),
            egui::StrokeKind::Inside,
        );
    }
    let color = if selected {
        p.text
    } else {
        mix(p.muted, p.text, hover)
    };
    super::icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(
            egui::pos2(rect.left() + 18.0, rect.center().y),
            Vec2::splat(16.0),
        ),
        icon,
        if selected { p.accent_text } else { color },
    );
    let badge_width = badge
        .map(|n| {
            let g = ui
                .painter()
                .layout_no_wrap(n.to_string(), font(11.0), p.muted);
            let width = g.size().x;
            ui.painter().galley(
                egui::pos2(
                    rect.right() - 12.0 - width,
                    rect.center().y - g.size().y / 2.0,
                ),
                g,
                p.muted,
            );
            width + MD
        })
        .unwrap_or(0.0);
    let mut job = egui::text::LayoutJob::simple(
        label.into(),
        if selected { medium(13.0) } else { font(13.0) },
        color,
        (rect.width() - 44.0 - badge_width).max(32.0),
    );
    job.wrap.max_rows = 2;
    let g = ui.painter().layout_job(job);
    ui.painter().galley(
        egui::pos2(rect.left() + 36.0, rect.center().y - g.size().y / 2.0),
        g,
        color,
    );
    let response = response.on_hover_text(label);
    if ui.is_enabled() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}
/// Equal hit cells and a single bounded selection plate avoid overlapping pill corners.
pub fn segmented(
    ui: &mut egui::Ui,
    labels: &[&str],
    current: usize,
    width: f32,
    p: Palette,
) -> (usize, Vec<egui::Rect>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, CONTROL), egui::Sense::hover());
    let id = ui.make_persistent_id("library-filter");
    let cell_width = (rect.width() - 2.0 * XS) / labels.len() as f32;
    let cells: Vec<_> = (0..labels.len())
        .map(|n| {
            egui::Rect::from_min_size(
                rect.min + Vec2::new(XS + n as f32 * cell_width, 0.0),
                Vec2::new(cell_width, CONTROL),
            )
        })
        .collect();
    let mut chosen = current;
    let responses: Vec<_> = cells
        .iter()
        .enumerate()
        .map(|(n, cell)| {
            let r = ui.interact(*cell, id.with(n), egui::Sense::click());
            r.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    ui.is_enabled(),
                    n == current,
                    labels[n],
                )
            });
            if r.clicked() {
                chosen = n;
            }
            r
        })
        .collect();
    ui.painter().rect_filled(rect, RADIUS, p.surface);
    let x = ui.ctx().animate_value_with_time(
        id.with("selection-x"),
        cells[chosen].left(),
        ui.style().animation_time,
    );
    let plate = egui::Rect::from_min_size(
        egui::pos2(x, rect.top() + XS),
        Vec2::new(cell_width, CONTROL - 2.0 * XS),
    )
    .intersect(rect.shrink(XS));
    ui.painter().rect(
        plate,
        RADIUS - 2,
        if p.dark { p.raised } else { p.panel },
        egui::Stroke::new(1.0_f32, p.line),
        egui::StrokeKind::Inside,
    );
    for (n, r) in responses.into_iter().enumerate() {
        let hover = motion(ui, r.id.with("hover"), r.hovered() || r.has_focus());
        let color = if n == chosen {
            p.text
        } else {
            mix(p.muted, p.text, hover)
        };
        let g = ui.painter().layout_no_wrap(
            labels[n].into(),
            if n == chosen {
                medium(13.0)
            } else {
                font(13.0)
            },
            color,
        );
        ui.painter()
            .galley(cells[n].center() - g.size() / 2.0, g, color);
        if r.has_focus() {
            ui.painter().rect_stroke(
                cells[n].shrink2(Vec2::new(0.0, 2.0)),
                RADIUS - 2,
                egui::Stroke::new(2.0_f32, p.focus),
                egui::StrokeKind::Inside,
            );
        }
        r.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    (chosen, cells)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn motion_is_finite_and_reduced_motion_changes_in_one_frame() {
        for reduced in [false, true] {
            let ctx = egui::Context::default();
            crate::ui::fonts::setup(&ctx);
            apply_settings(
                &ctx,
                &Settings {
                    reduce_motion: reduced,
                    ..Default::default()
                },
            );
            let mut intermediate = false;
            let mut previous = 0.0;
            for frame in 0..60 {
                let target = (4..32).contains(&frame);
                let mut value = 0.0;
                let output = ctx.run(
                    egui::RawInput {
                        time: Some(frame as f64 / 60.0),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            value = motion(ui, egui::Id::new("tested-transition"), target)
                        });
                    },
                );
                assert!((0.0..=1.0).contains(&value));
                if reduced {
                    assert_eq!(value, if target { 1.0 } else { 0.0 });
                }
                if (0.0..1.0).contains(&value) && value > 0.0 {
                    intermediate = true;
                }
                if (5..32).contains(&frame) {
                    assert!(value >= previous);
                }
                if frame == 30 {
                    assert_eq!(value, 1.0);
                }
                if frame == 59 {
                    assert_eq!(value, 0.0);
                    assert_eq!(
                        output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
                        std::time::Duration::MAX,
                        "animation must stop requesting frames"
                    );
                }
                previous = value;
            }
            assert_eq!(intermediate, !reduced);
        }
    }
    #[test]
    fn primary_shape_and_contrast_stay_stable_across_hover_press_and_themes() {
        for theme in [Theme::Cavalry, Theme::Light, Theme::Custom] {
            let ctx = egui::Context::default();
            crate::ui::fonts::setup(&ctx);
            let settings = Settings {
                theme,
                accent: [128, 128, 128],
                ..Default::default()
            };
            apply_settings(&ctx, &settings);
            let p = palette(&settings);
            let mut baseline = None;
            for frame in 0..20 {
                let mut events = vec![];
                if let Some(rect) = baseline {
                    let rect: egui::Rect = rect;
                    events.push(egui::Event::PointerMoved(rect.center()));
                    if frame == 8 || frame == 16 {
                        events.push(egui::Event::PointerButton {
                            pos: rect.center(),
                            button: egui::PointerButton::Primary,
                            pressed: frame == 8,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                }
                let mut rect = egui::Rect::NOTHING;
                let output = ctx.run(
                    egui::RawInput {
                        time: Some(frame as f64 / 60.0),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            rect = button(ui, "Select files", Button::Primary, p).rect
                        });
                    },
                );
                if let Some(before) = baseline {
                    assert_eq!(rect, before, "hover may not expand the click target");
                }
                baseline = Some(rect);
                let fills: Vec<_> = output
                    .shapes
                    .iter()
                    .filter_map(|s| match &s.shape {
                        egui::Shape::Rect(r) if r.rect == rect && r.fill.a() > 0 => Some(r),
                        _ => None,
                    })
                    .collect();
                assert_eq!(fills.len(), 1, "paint only one rounded button shell");
                assert_eq!(fills[0].corner_radius, egui::CornerRadius::same(RADIUS));
                assert!(contrast(p.on_accent, fills[0].fill) >= 4.5);
            }
        }
        let light = palette(&Settings {
            theme: Theme::Light,
            ..Default::default()
        });
        let dark = palette(&Settings::default());
        assert_eq!(light.panel, Color32::WHITE);
        assert!(luminance(light.panel) > luminance(light.bg));
        assert!(luminance(dark.panel) > luminance(dark.bg));
    }
    #[test]
    fn custom_buttons_support_keyboard_activation_and_ignore_disabled_actions() {
        for enabled in [true, false] {
            let ctx = egui::Context::default();
            crate::ui::fonts::setup(&ctx);
            apply(&ctx);
            let p = palette(&Settings::default());
            let mut id = egui::Id::NULL;
            let mut activated = false;
            for frame in 0..4 {
                if frame == 2 {
                    ctx.memory_mut(|m| m.request_focus(id));
                }
                let events = if frame == 3 {
                    vec![egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: Some(egui::Key::Enter),
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }]
                } else {
                    vec![]
                };
                let _ = ctx.run(
                    egui::RawInput {
                        time: Some(frame as f64),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.add_enabled_ui(enabled, |ui| {
                                let r = button(ui, "Confirm", Button::Primary, p);
                                id = r.id;
                                activated |= r.clicked();
                            });
                        });
                    },
                );
            }
            assert_eq!(activated, enabled);
        }
    }
    #[test]
    fn small_checkbox_does_not_become_a_radio_circle() {
        let ctx = egui::Context::default();
        crate::ui::fonts::setup(&ctx);
        apply(&ctx);
        let p = palette(&Settings::default());
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                checkbox(ui, &mut false, "Reduce motion", p);
            });
        });
        let mark = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Rect(r)
                    if r.rect.width() > 10.0
                        && r.rect.width() < 24.0
                        && (r.rect.width() - r.rect.height()).abs() < 0.5 =>
                {
                    Some(r)
                }
                _ => None,
            })
            .expect("checkbox input shape");
        assert_eq!(mark.corner_radius, egui::CornerRadius::same(3));
        assert!(contrast(mark.stroke.color, p.panel) >= 2.9);
    }
    #[test]
    fn presets_and_extreme_custom_colors_have_readable_text() {
        for theme in [Theme::Cavalry, Theme::Light, Theme::Custom] {
            for background in [
                [0, 0, 0],
                [255, 255, 255],
                [125, 125, 125],
                [255, 0, 255],
                [0, 255, 0],
                [20, 60, 90],
            ] {
                for accent in [
                    [0, 0, 0],
                    [255, 255, 255],
                    [128, 128, 128],
                    [255, 255, 0],
                    [0, 0, 255],
                ] {
                    let p = palette(&Settings {
                        theme,
                        background,
                        accent,
                        ..Default::default()
                    });
                    for bg in [p.bg, p.panel, p.surface, p.raised, p.selected] {
                        for fg in [p.text, p.muted, p.danger, p.success, p.accent_text] {
                            assert!(
                                contrast(fg, bg) >= 4.5 - 0.01,
                                "{fg:?} {bg:?}: {}",
                                contrast(fg, bg)
                            );
                        }
                    }
                    assert!(contrast(p.on_accent, p.accent) >= 4.5);
                    for bg in [p.bg, p.panel, p.surface, p.raised] {
                        assert!(contrast(mix(bg, p.text, scrollbar_opacity(p)), bg) >= 3.0);
                    }
                }
            }
        }
    }
    #[test]
    fn native_system_theme_events_cannot_override_the_saved_color_scheme() {
        let ctx = egui::Context::default();
        crate::ui::fonts::setup(&ctx);
        for theme in [Theme::Cavalry, Theme::Light, Theme::Custom] {
            let settings = Settings {
                theme,
                background: [62, 42, 80],
                ..Default::default()
            };
            apply_settings(&ctx, &settings);
            let p = palette(&settings);
            for system in [egui::Theme::Light, egui::Theme::Dark] {
                let _ = ctx.run(
                    egui::RawInput {
                        system_theme: Some(system),
                        ..Default::default()
                    },
                    |ctx| {
                        assert_eq!(ctx.style().visuals.selection.bg_fill, p.accent);
                        assert_eq!(ctx.style().visuals.extreme_bg_color, p.bg);
                        assert_eq!(ctx.style().visuals.dark_mode, p.dark);
                    },
                );
            }
        }
    }
}
