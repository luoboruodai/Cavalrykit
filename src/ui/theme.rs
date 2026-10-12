//! Cavalry-inspired neutrals and a 4/8/16/24 spacing scale. Colors are per-app, not globals.
use crate::settings::{Settings, Theme};
use egui::{Color32, FontId, RichText, Vec2};
pub const XS: f32 = 4.0;
pub const SM: f32 = 8.0;
pub const MD: f32 = 16.0;
pub const LG: f32 = 24.0;
pub const CONTROL: f32 = 36.0;
pub const ROW: f32 = 64.0;
pub const HEADER: f32 = 64.0;
pub const SIDEBAR: f32 = 208.0;
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
        Theme::Cavalry => (rgb([25, 27, 30]), rgb([110, 216, 164])),
        Theme::Light => (rgb([249, 250, 251]), rgb([31, 119, 82])),
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
    let surface = safe_surface(if dark { 0.045 } else { 0.025 });
    let panel = safe_surface(0.020);
    let raised = safe_surface(if dark { 0.075 } else { 0.05 });
    let selected = safe_surface(if dark { 0.075 } else { 0.065 });
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
        focus: readable(accent, surface, 3.0),
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
    style.visuals.window_fill = p.raised;
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
        widget.corner_radius = egui::CornerRadius::same(3);
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
    style.animation_time = if s.reduce_motion { 0.0 } else { 0.12 };
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
pub fn button(ui: &mut egui::Ui, label: &str, kind: Button, p: Palette) -> egui::Response {
    let (fill, ink, stroke) = match kind {
        Button::Primary => (p.accent, p.on_accent, egui::Stroke::NONE),
        Button::Danger => (p.surface, p.danger, egui::Stroke::new(1.0_f32, p.danger)),
        Button::Secondary => (p.surface, p.text, egui::Stroke::new(1.0_f32, p.line)),
        Button::Quiet => (Color32::TRANSPARENT, p.muted, egui::Stroke::NONE),
        Button::QuietDanger => (Color32::TRANSPARENT, p.danger, egui::Stroke::NONE),
    };
    let response = ui
        .add(
            egui::Button::new(RichText::new(label).font(medium(14.0)).color(ink))
                .fill(fill)
                .stroke(stroke)
                .corner_radius(6)
                .min_size(Vec2::new(64.0, CONTROL)),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.has_focus() || response.hovered() {
        ui.painter().rect_stroke(
            response.rect,
            6,
            egui::Stroke::new(
                if response.has_focus() {
                    2.0_f32
                } else {
                    1.0_f32
                },
                if matches!(kind, Button::Danger | Button::QuietDanger) {
                    p.danger
                } else {
                    p.focus
                },
            ),
            egui::StrokeKind::Inside,
        );
    }
    response
}
pub fn card(p: Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(p.panel)
        .corner_radius(8)
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
    let active = response.hovered() || response.has_focus();
    let color = if active && danger { p.danger } else { p.muted };
    if active {
        ui.painter().rect_filled(rect, 6, p.raised);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            6,
            egui::Stroke::new(2.0_f32, p.focus),
            egui::StrokeKind::Inside,
        );
    }
    super::icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(rect.center(), Vec2::splat(18.0)),
        icon,
        color,
    );
    response
        .on_hover_text(label)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
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
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), egui::Sense::click());
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
    if selected || response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, 6, if selected { p.selected } else { p.surface });
    }
    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min + Vec2::new(0.0, 12.0), Vec2::new(2.0_f32, 16.0)),
            1,
            p.accent_text,
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            6,
            egui::Stroke::new(2.0_f32, p.focus),
            egui::StrokeKind::Inside,
        );
    }
    let color = if selected { p.text } else { p.muted };
    super::icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            Vec2::splat(16.0),
        ),
        icon,
        if selected { p.accent_text } else { p.muted },
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
                    rect.center().y - g.size().y / 2.0_f32,
                ),
                g,
                p.muted,
            );
            width + MD
        })
        .unwrap_or(0.0);
    // Navigation must remain identifiable in longer translations; only filenames truncate.
    let mut job = egui::text::LayoutJob::simple(
        label.into(),
        if selected { medium(13.0) } else { font(13.0) },
        color,
        (rect.width() - 48.0 - badge_width).max(32.0),
    );
    job.wrap.max_rows = 2;
    let g = ui.painter().layout_job(job);
    ui.painter().galley(
        egui::pos2(rect.left() + 36.0, rect.center().y - g.size().y / 2.0_f32),
        g,
        color,
    );
    response
        .on_hover_text(label)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[cfg(test)]
mod tests {
    use super::*;
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
