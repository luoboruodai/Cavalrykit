//! Cavalry-inspired neutrals and a 4/8/16/24 spacing scale. Colors are per-app, not globals.
use crate::settings::{Settings, Theme};
use egui::{Color32, FontId, RichText, Vec2};
pub const XS: f32 = 4.0;
pub const SM: f32 = 8.0;
pub const MD: f32 = 16.0;
pub const LG: f32 = 24.0;
pub const CONTROL: f32 = 40.0;
pub const SHORTCUT_HEIGHT: f32 = 40.0;
pub const SEARCH_HEIGHT: f32 = 44.0;
pub const ROW: f32 = 64.0;
pub const LIST_GUTTER: f32 = 16.0;
pub const ROW_INSET: f32 = 4.0;
pub const HEADER: f32 = 40.0;
pub const RADIUS: u8 = 6;
pub const CARD_RADIUS: u8 = 8;
#[derive(Clone, Copy, Debug, PartialEq)]
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
/// Frame-by-frame blend between two complete palettes (theme switching). The `dark`
/// contrast direction flips at the midpoint; every surface/ink color eases together.
pub fn lerp_palette(a: Palette, b: Palette, k: f32) -> Palette {
    Palette {
        bg: mix(a.bg, b.bg, k),
        panel: mix(a.panel, b.panel, k),
        surface: mix(a.surface, b.surface, k),
        raised: mix(a.raised, b.raised, k),
        text: mix(a.text, b.text, k),
        muted: mix(a.muted, b.muted, k),
        border: mix(a.border, b.border, k),
        line: mix(a.line, b.line, k),
        selected: mix(a.selected, b.selected, k),
        accent: mix(a.accent, b.accent, k),
        on_accent: mix(a.on_accent, b.on_accent, k),
        accent_text: mix(a.accent_text, b.accent_text, k),
        danger: mix(a.danger, b.danger, k),
        success: mix(a.success, b.success, k),
        warning: mix(a.warning, b.warning, k),
        focus: mix(a.focus, b.focus, k),
        dark: if k < 0.5 { a.dark } else { b.dark },
    }
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
        Theme::Cavalry => (rgb([21, 24, 30]), rgb([7, 132, 48])),
        Theme::Light => (rgb([243, 245, 248]), rgb([17, 121, 61])),
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
            rgb([29, 33, 41]),
            rgb([38, 43, 52]),
            rgb([48, 55, 66]),
            rgb([24, 48, 34]),
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
        line: mix(text, surface, if dark { 0.94 } else { 0.89 }),
        selected,
        accent,
        on_accent: ink(accent),
        // Keep the small active indicator lively; filled actions use a darker green
        // so their white label stays readable instead of copying inaccessible colors.
        accent_text: readable_all(
            if s.theme == Theme::Cavalry {
                rgb([29, 200, 88])
            } else {
                accent
            },
            4.5,
        ),
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
/// One semantic color per kind, reused by outlines, icons, nav, badges and CTAs.
/// Opaque text remains readable; translucent fills are decorative, not alternate greens.
#[derive(Clone, Copy, Debug)]
pub struct Category {
    pub color: Color32,
    pub tint: Color32,
}
pub fn category(p: Palette, plugin: bool) -> Category {
    let candidate = if plugin {
        if p.dark {
            rgb([91, 191, 244])
        } else {
            rgb([0, 103, 163])
        }
    } else {
        p.accent_text
    };
    let target = ink(p.panel);
    let mut color = target;
    for n in 0..=40 {
        let c = mix(candidate, target, n as f32 / 40.0);
        if [p.bg, p.panel, p.surface, p.raised]
            .iter()
            .all(|bg| contrast(c, *bg) >= 4.5)
        {
            color = c;
            break;
        }
    }
    let tentative = mix(p.panel, color, if p.dark { 0.10 } else { 0.075 });
    // An extreme custom mid-gray can consume the whole contrast budget. Never
    // darken/lighten its badge toward the text if that would make the label unreadable.
    let tint = if contrast(color, tentative) >= 4.5 {
        tentative
    } else {
        p.panel
    };
    Category { color, tint }
}
pub fn helper_ink(p: Palette) -> Color32 {
    readable(mix(p.muted, p.panel, 0.42), p.panel, 4.5)
}
pub fn drop_surface(p: Palette) -> Color32 {
    mix(p.panel, p.surface, 0.65)
}
pub fn placeholder_ink(p: Palette) -> Color32 {
    readable(mix(p.muted, p.panel, 0.18), p.panel, 4.5)
}

/// Neutral secondary surfaces mirror the dark hierarchy, not an outlined input.
/// Light mode needs a little more gray mass; hard default outlines overemphasize badges.
pub fn neutral_surface(p: Palette) -> Color32 {
    if p.dark {
        p.surface
    } else {
        mix(p.panel, p.raised, 0.72)
    }
}
pub fn neutral_stroke(p: Palette) -> egui::Stroke {
    if p.dark {
        egui::Stroke::new(1.0_f32, p.line)
    } else {
        egui::Stroke::NONE
    }
}
pub fn count_style(p: Palette, tone: Category) -> (Color32, Color32) {
    let fill = if p.dark {
        tone.tint
    } else {
        mix(p.raised, tone.color, 0.065)
    };
    (fill, readable(tone.color, fill, 4.5))
}
pub fn count_badge(
    ui: &egui::Ui,
    rect: egui::Rect,
    label: &str,
    tone: Category,
    p: Palette,
) -> Color32 {
    let (fill, ink) = count_style(p, tone);
    ui.painter().rect_filled(rect, 6, fill);
    let g = ui.painter().layout_no_wrap(label.into(), medium(12.0), ink);
    ui.painter().galley(centered_ink(rect, &g), g, ink);
    let r = ui.interact(
        rect,
        ui.make_persistent_id("result-count"),
        egui::Sense::hover(),
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, label));
    ink
}
/// A single vertical shortcut card; no second icon target overlays its label.
pub fn shortcut(
    ui: &mut egui::Ui,
    label: &str,
    icon: super::icons::Icon,
    tone: Category,
    p: Palette,
) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), SHORTCUT_HEIGHT),
        egui::Sense::click(),
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let amount = motion(
        ui,
        r.id.with("hover"),
        ui.is_enabled() && (r.hovered() || r.has_focus()),
    );
    let bg = mix(p.surface, p.raised, amount * 0.65);
    let scale = 1.0 + amount * 0.03;
    let card_rect = egui::Rect::from_center_size(rect.center(), rect.size() * (scale / 1.03));
    ui.painter().rect(
        card_rect,
        CARD_RADIUS,
        bg,
        egui::Stroke::new(if r.has_focus() { 2.0_f32 } else { amount }, tone.color),
        egui::StrokeKind::Inside,
    );
    let center = egui::pos2(card_rect.left() + 22.0 * scale, card_rect.center().y);
    super::icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(center, Vec2::splat(18.0 * scale)),
        icon,
        mix(p.muted, tone.color, 0.55 + 0.45 * amount),
    );
    let mut job = egui::text::LayoutJob::simple(
        label.into(),
        font(13.5 * scale),
        mix(p.muted, p.text, amount),
        (card_rect.width() - 56.0 * scale).max(40.0),
    );
    job.wrap.max_rows = 2;
    let g = ui.painter().layout_job(job);
    ui.painter().galley(
        left_ink(card_rect.left() + 40.0 * scale, card_rect.center().y, &g),
        g,
        p.text,
    );
    // The accent rail grows and brightens with hover; the hit region never moves.
    let rail_h = (rect.height() - 16.0) * (0.52 + 0.24 * amount);
    let rail = egui::Rect::from_min_size(
        egui::pos2(card_rect.left(), card_rect.center().y - rail_h / 2.0),
        Vec2::new(2.0 + 2.0 * amount, rail_h),
    );
    ui.painter()
        .rect_filled(rail, 1, mix(p.surface, tone.color, 0.55 + 0.45 * amount));
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}
/// Full-width, soft-tinted action below the drop surface. Icon/label share one hit region.
pub fn import_action(ui: &mut egui::Ui, label: &str, tone: Category, p: Palette) -> egui::Response {
    soft_action(
        ui,
        label,
        super::icons::Icon::Import,
        tone,
        p,
        ui.available_width(),
    )
}
/// Shared material for import and Save. The footer uses intrinsic width, not a full row.
pub fn soft_action(
    ui: &mut egui::Ui,
    label: &str,
    icon: super::icons::Icon,
    tone: Category,
    p: Palette,
    width: f32,
) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(Vec2::new(width, SEARCH_HEIGHT), egui::Sense::click());
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let amount = motion(
        ui,
        r.id.with("hover"),
        ui.is_enabled() && (r.hovered() || r.has_focus()),
    );
    let press = motion(
        ui,
        r.id.with("press"),
        ui.is_enabled() && r.is_pointer_button_down_on(),
    );
    ui.painter().rect(
        rect,
        10,
        mix(tone.tint, p.surface, press * 0.2),
        egui::Stroke::new(if r.has_focus() { 2.0_f32 } else { amount }, tone.color),
        egui::StrokeKind::Inside,
    );
    let g = ui
        .painter()
        .layout_no_wrap(label.into(), medium(14.0), tone.color);
    let width = 18.0 + SM + g.size().x;
    let left = rect.center().x - width / 2.0;
    super::icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(
            egui::pos2(left + 9.0, rect.center().y + press),
            Vec2::splat(18.0),
        ),
        icon,
        tone.color,
    );
    ui.painter().galley(
        left_ink(left + 18.0 + SM, rect.center().y + press, &g),
        g,
        tone.color,
    );
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}
pub fn path_field(
    ui: &mut egui::Ui,
    value: &mut String,
    hint: &str,
    label: &str,
    p: Palette,
) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), CONTROL),
        egui::Sense::hover(),
    );
    let id = ui.make_persistent_id("path-input");
    let focus = ui.memory(|m| m.has_focus(id));
    let hover = motion(
        ui,
        id.with("hover"),
        ui.is_enabled() && (ui.rect_contains_pointer(rect) || focus),
    );
    ui.painter().rect(
        rect,
        8,
        p.surface,
        egui::Stroke::new(
            if focus { 1.5 } else { 0.7 + 0.3 * hover },
            mix(p.line, p.focus, hover),
        ),
        egui::StrokeKind::Inside,
    );
    let inner = rect.shrink2(Vec2::new(12.0, 0.0));
    let text = if value.is_empty() {
        hint
    } else {
        value.as_str()
    };
    let g = ui.painter().layout_no_wrap(text.into(), font(13.0), p.text);
    let shift = g.rect.center().y - ink_bounds(&g).center().y;
    let mut child =
        ui.new_child(egui::UiBuilder::new().max_rect(inner.translate(Vec2::new(0.0, shift))));
    let output = egui::TextEdit::singleline(value)
        .id(id)
        .font(font(13.0))
        .frame(false)
        .margin(egui::Margin::ZERO)
        .min_size(inner.size())
        .desired_width(inner.width())
        .vertical_align(egui::Align::Center)
        .show(&mut child);
    if value.is_empty() {
        let g = label_truncated(ui, hint, font(13.0), helper_ink(p), inner.width());
        ui.painter().galley(
            left_ink(inner.left(), rect.center().y, &g),
            g,
            helper_ink(p),
        );
    }
    output.response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, ui.is_enabled(), label)
    });
    output.response
}
pub fn save_action(ui: &mut egui::Ui, label: &str, p: Palette) -> egui::Response {
    let tone = category(p, false);
    let g = ui
        .painter()
        .layout_no_wrap(label.into(), medium(13.0), tone.color);
    let (rect, r) = ui.allocate_exact_size(
        Vec2::new((g.size().x + 2.0 * MD).max(88.0), CONTROL),
        egui::Sense::click(),
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let hover = motion(
        ui,
        r.id.with("hover"),
        ui.is_enabled() && (r.hovered() || r.has_focus()),
    );
    ui.painter().rect(
        rect,
        8,
        mix(tone.tint, p.surface, hover * 0.18),
        egui::Stroke::new(if r.has_focus() { 1.5 } else { hover * 0.7 }, tone.color),
        egui::StrokeKind::Inside,
    );
    ui.painter().galley(centered_ink(rect, &g), g, tone.color);
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}
pub fn text_action(ui: &mut egui::Ui, label: &str, p: Palette) -> egui::Response {
    let g = ui
        .painter()
        .layout_no_wrap(label.into(), font(12.0), helper_ink(p));
    let (rect, r) = ui.allocate_exact_size(
        Vec2::new(g.size().x + 2.0 * SM, CONTROL),
        egui::Sense::click(),
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let hover = motion(
        ui,
        r.id.with("hover"),
        ui.is_enabled() && (r.hovered() || r.has_focus()),
    );
    if r.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(2.0),
            6,
            egui::Stroke::new(1.0_f32, p.focus),
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().galley_with_override_text_color(
        centered_ink(rect, &g),
        g,
        mix(helper_ink(p), p.muted, hover),
    );
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
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
/// Optical ink metrics, not the font's line box (which includes asymmetric leading).
pub fn ink_bounds(g: &egui::Galley) -> egui::Rect {
    if g.mesh_bounds.is_finite() && g.mesh_bounds.is_positive() {
        g.mesh_bounds
    } else {
        g.rect
    }
}
pub fn centered_ink(rect: egui::Rect, g: &egui::Galley) -> egui::Pos2 {
    rect.center() - ink_bounds(g).center().to_vec2()
}
pub fn left_ink(left: f32, center_y: f32, g: &egui::Galley) -> egui::Pos2 {
    let ink = ink_bounds(g);
    egui::pos2(left - ink.left(), center_y - ink.center().y)
}
pub fn content_font(size: f32) -> FontId {
    FontId::new(
        size,
        egui::FontFamily::Name(super::fonts::FAMILY_CONTENT.into()),
    )
}
/// Row affordances share one finite timeline. Paint moves within stable hit targets.
pub fn row_action_offset(progress: f32) -> Vec2 {
    Vec2::new(-ROW_INSET * (1.0 - progress.clamp(0.0, 1.0)), 0.0)
}
pub fn action_glyph_rect(hit: egui::Rect, group_hover: Option<f32>) -> egui::Rect {
    let progress = group_hover.unwrap_or(0.0).clamp(0.0, 1.0);
    egui::Rect::from_center_size(
        hit.center() + group_hover.map(row_action_offset).unwrap_or(Vec2::ZERO),
        Vec2::splat(18.0 + 1.5 * progress),
    )
}
pub fn row_action_ink(p: Palette, progress: f32) -> Color32 {
    mix(p.muted, p.text, progress.clamp(0.0, 1.0) * 0.65)
}
/// Decoration only: no button, focus target, or independent animation clock.
pub fn decorative_dots(painter: &egui::Painter, center: egui::Pos2, p: Palette, active: f32) {
    let active = active.clamp(0.0, 1.0);
    let center = center + row_action_offset(active);
    let ink = mix(p.muted, p.panel, 0.55 - active * 0.23);
    for y in [-4.0, 0.0, 4.0] {
        let c = center + Vec2::new(0.0, y);
        painter.circle_filled(
            c,
            1.7 + 0.1 * active,
            ink.gamma_multiply(0.15 + 0.05 * active),
        );
        painter.circle_filled(c, 0.9 + 0.15 * active, ink);
    }
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
    apply_palette(ctx, s, palette(s));
}
/// Same style application with an explicit palette, used by the theme transition so
/// stock egui widgets ease through the same intermediate colors as custom painting.
pub fn apply_palette(ctx: &egui::Context, s: &Settings, p: Palette) {
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
    style.animation_time = if s.reduce_motion { 0.0 } else { 0.18 };
    style.text_styles.insert(egui::TextStyle::Body, font(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, medium(13.0));
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
    // Retarget from the painted state instead of consuming a stale reactive-mode delta.
    // Ordinary pointer hover and file drag are deliberately separate state channels.
    let value = super::motion::travel(ui.ctx(), id, if active { 1.0 } else { 0.0 }, 0.24);
    super::diagnostics::sample(
        ui.ctx(),
        "control",
        id,
        if active { 1.0 } else { 0.0 },
        value,
    );
    value
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
        if matches!(kind, Button::Quiet | Button::QuietDanger) {
            font(13.0)
        } else {
            medium(13.0)
        },
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
            Button::Secondary => mix(
                neutral_surface(p),
                p.raised,
                (hover * 0.65 + press * 0.35).min(1.0),
            ),
            _ => mix(p.surface, p.raised, (hover * 0.65 + press * 0.35).min(1.0)),
        }
    };
    let border = if response.has_focus() {
        egui::Stroke::new(2.0_f32, p.focus)
    } else if danger && !matches!(kind, Button::QuietDanger) {
        egui::Stroke::new(1.0_f32, p.danger)
    } else if matches!(kind, Button::Secondary) {
        neutral_stroke(p)
    } else {
        egui::Stroke::NONE
    };
    if primary && ui.is_enabled() {
        ui.painter().add(
            egui::Shadow {
                offset: [0, 3],
                blur: 14,
                spread: 0,
                color: if p.dark {
                    p.accent.gamma_multiply(0.09 + hover * 0.06)
                } else {
                    Color32::from_black_alpha(14)
                },
            }
            .as_shape(rect, RADIUS),
        );
    }
    ui.painter()
        .rect(rect, RADIUS, fill, border, egui::StrokeKind::Inside);
    if primary {
        ui.painter().hline(
            rect.left() + 6.0..=rect.right() - 6.0,
            rect.top() + 0.8,
            egui::Stroke::new(
                1.0_f32,
                Color32::WHITE.gamma_multiply(if p.dark { 0.10 } else { 0.14 }),
            ),
        );
    }
    ui.painter().galley(
        centered_ink(rect, &galley) + Vec2::new(0.0, press),
        galley,
        ink,
    );
    if ui.is_enabled() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

pub fn secondary_icon_button(
    ui: &mut egui::Ui,
    label: &str,
    icon: super::icons::Icon,
    p: Palette,
    group_hover: f32,
) -> (egui::Response, egui::Rect, egui::Rect) {
    let galley = ui
        .painter()
        .layout_no_wrap(label.into(), medium(13.0), p.text);
    let icon_size = 16.0;
    let content_width = icon_size + SM + galley.size().x;
    let size = Vec2::new(
        (content_width + 2.0 * MD).max(64.0),
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
    let amount = (group_hover * 0.35 + hover).clamp(0.0, 1.0);
    let scale = 1.0 + amount * 0.04;
    let visual = egui::Rect::from_center_size(rect.center(), rect.size() * scale);
    let fill = mix(
        neutral_surface(p),
        p.raised,
        (amount * 0.65 + press * 0.35).min(1.0),
    );
    let border = if response.has_focus() {
        egui::Stroke::new(2.0_f32, p.focus)
    } else {
        egui::Stroke::new(
            0.5 + amount * 0.5,
            mix(neutral_stroke(p).color, p.accent, amount * 0.55),
        )
    };
    ui.painter()
        .rect(visual, RADIUS, fill, border, egui::StrokeKind::Inside);
    let left = visual.center().x - content_width * scale / 2.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(left + icon_size * scale / 2.0, visual.center().y + press),
        Vec2::splat(icon_size * scale),
    );
    super::icons::paint(
        ui.painter(),
        icon_rect,
        icon,
        mix(p.muted, p.accent, 0.45 + amount * 0.55),
    );
    ui.painter().galley(
        left_ink(
            left + (icon_size + SM) * scale,
            visual.center().y + press,
            &galley,
        ),
        galley,
        mix(p.text, p.accent, amount * 0.18),
    );
    let response = if ui.is_enabled() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    };
    (response, visual, icon_rect)
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
/// A switch row has one full-row target; the label and the visual track share it.
pub fn toggle_row(ui: &mut egui::Ui, value: &mut bool, label: &str, p: Palette) -> egui::Response {
    let width = ui.available_width();
    let g = ui
        .painter()
        .layout(label.into(), font(14.0), p.text, (width - 64.0).max(48.0));
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, (g.size().y + 2.0 * SM).max(48.0)),
        egui::Sense::click(),
    );
    if response.clicked() {
        *value = !*value;
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *value, label)
    });
    let hover = motion(
        ui,
        response.id.with("row-hover"),
        response.hovered() || response.has_focus(),
    );
    ui.painter()
        .rect_filled(rect, RADIUS, mix(p.panel, p.surface, hover));
    let ink = rect.left_center() + Vec2::new(SM, -g.size().y / 2.0);
    ui.painter().galley(ink, g, p.text);
    let track = egui::Rect::from_center_size(
        egui::pos2(rect.right() - 26.0, rect.center().y),
        Vec2::new(36.0, 20.0),
    );
    let amount = motion(ui, response.id.with("toggle"), *value);
    ui.painter().rect(
        track,
        10,
        mix(p.raised, p.accent, amount),
        egui::Stroke::new(1.0_f32, if *value { p.accent } else { p.border }),
        egui::StrokeKind::Inside,
    );
    ui.painter().circle_filled(
        egui::pos2(track.left() + 10.0 + 16.0 * amount, track.center().y),
        7.0,
        if *value { p.on_accent } else { p.text },
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            RADIUS,
            egui::Stroke::new(2.0_f32, p.focus),
            egui::StrokeKind::Inside,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
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
    modal_state(ctx, id, p, width, false, contents)
}
pub fn modal_state<R>(
    ctx: &egui::Context,
    id: &str,
    p: Palette,
    width: f32,
    closing: bool,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::ModalResponse<R> {
    let id = egui::Id::new(id);
    let progress =
        super::motion::modal_progress(ctx, id, closing, if closing { 0.24 } else { 0.38 });
    let surface = super::motion::phase(progress, 0.0, 0.70);
    let content = super::motion::phase(progress, 0.10, 1.0);
    egui::Modal::new(id)
        .area(
            egui::Modal::default_area(id)
                .fade_in(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 18.0 * (1.0 - progress)]),
        )
        .backdrop_color(Color32::from_black_alpha((90.0 * surface) as u8))
        .frame(
            egui::Frame::new()
                .fill(p.panel.gamma_multiply(surface))
                .stroke(egui::Stroke::new(1.0_f32, p.line.gamma_multiply(surface)))
                .shadow(egui::Shadow {
                    offset: [0, 12],
                    blur: 40,
                    spread: 0,
                    color: Color32::from_black_alpha((90.0 * surface) as u8),
                })
                .corner_radius(14)
                .inner_margin(LG),
        )
        .show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 96.0).clamp(260.0, width));
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.multiply_opacity(content);
            ui.add_enabled_ui(!closing && content >= 0.85, contents)
                .inner
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
    icon_button_guarded(ui, label, icon, danger, p, true, None)
}
pub fn icon_button_guarded(
    ui: &mut egui::Ui,
    label: &str,
    icon: super::icons::Icon,
    danger: bool,
    p: Palette,
    interactive: bool,
    group_hover: Option<f32>,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::splat(CONTROL),
        if interactive {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled() && interactive,
            label,
        )
    });
    let hover = motion(
        ui,
        response.id.with("hover"),
        ui.is_enabled() && interactive && (response.hovered() || response.has_focus()),
    );
    let press = motion(
        ui,
        response.id.with("press"),
        ui.is_enabled() && interactive && response.is_pointer_button_down_on(),
    );
    let base = row_action_ink(p, group_hover.unwrap_or(0.0));
    let color = mix(base, if danger { p.danger } else { p.text }, hover);
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
        action_glyph_rect(rect, group_hover).translate(Vec2::new(0.0, press)),
        icon,
        color,
    );
    let response = response.on_hover_text(label);
    if ui.is_enabled() && interactive {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}
/// A quiet, always visible refresh tile with bounded feedback rather than an idle spinner.
pub fn refresh_button(ui: &mut egui::Ui, label: &str, p: Palette) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(Vec2::splat(SEARCH_HEIGHT), egui::Sense::click());
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let active = motion(
        ui,
        r.id.with("hover"),
        ui.is_enabled() && (r.hovered() || r.has_focus()),
    );
    let press = motion(
        ui,
        r.id.with("press"),
        ui.is_enabled() && r.is_pointer_button_down_on(),
    );
    let tone = category(p, false);
    ui.painter().rect(
        rect,
        10,
        mix(p.panel, tone.tint, active),
        egui::Stroke::new(if r.has_focus() { 1.5 } else { active }, tone.color),
        egui::StrokeKind::Inside,
    );
    let now = ui.input(|i| i.time);
    if r.clicked() {
        ui.data_mut(|d| d.insert_temp(r.id.with("spin"), now));
    }
    let elapsed = ui
        .data(|d| d.get_temp::<f64>(r.id.with("spin")))
        .map(|at| (now - at) as f32);
    let spin = elapsed
        .filter(|t| *t < 0.38 && ui.style().animation_time > 0.0)
        .map(|t| {
            ui.ctx().request_repaint();
            super::motion::cubic_out(t / 0.38) * std::f32::consts::TAU
        })
        .unwrap_or(0.0);
    let center = rect.center() + Vec2::new(0.0, press);
    super::icons::paint_refresh(
        ui.painter(),
        egui::Rect::from_center_size(center, Vec2::splat(20.0)),
        mix(p.muted, tone.color, active),
        spin,
    );
    r.on_hover_text(label)
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
        ui.painter().rect_filled(rect, RADIUS, p.surface);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                rect.min + Vec2::new(0.0, 10.0),
                Vec2::new(2.0, rect.height() - 20.0),
            ),
            1,
            p.accent_text,
        );
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
    ui.painter()
        .galley(left_ink(rect.left() + 36.0, rect.center().y, &g), g, color);
    let response = response.on_hover_text(label);
    if ui.is_enabled() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}
/// Soft segmented selector. Only the inset plate moves; hit cells never move.
pub fn segmented(
    ui: &mut egui::Ui,
    labels: &[&str],
    current: usize,
    width: f32,
    p: Palette,
) -> (usize, Vec<egui::Rect>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, CONTROL), egui::Sense::hover());
    let id = ui.make_persistent_id("library-filter");
    let inner = rect.shrink2(Vec2::new(XS, 0.0));
    let cell_width = inner.width() / labels.len() as f32;
    let cells: Vec<_> = (0..labels.len())
        .map(|n| {
            egui::Rect::from_min_size(
                inner.min + Vec2::new(n as f32 * cell_width, 0.0),
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
    ui.painter().rect_filled(rect, 20, p.panel);
    let position = super::motion::travel(ui.ctx(), id.with("plate-position"), chosen as f32, 0.22);
    let plate = egui::Rect::from_min_size(
        egui::pos2(inner.left() + position * cell_width, rect.top() + XS),
        Vec2::new(cell_width, CONTROL - 2.0 * XS),
    );
    ui.painter().rect(
        plate,
        16,
        if p.dark { p.surface } else { Color32::WHITE },
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
        ui.painter().galley(centered_ink(cells[n], &g), g, color);
        if r.has_focus() {
            ui.painter().rect_stroke(
                cells[n].shrink2(Vec2::new(0.0, XS)),
                16,
                egui::Stroke::new(2.0_f32, category(p, n == 2).color),
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
    fn light_secondary_buttons_keep_dark_geometry_without_the_input_like_outline() {
        let mut bounds = Vec::new();
        for theme in [Theme::Cavalry, Theme::Light] {
            let ctx = egui::Context::default();
            crate::ui::fonts::setup(&ctx);
            let settings = Settings {
                theme,
                ..Default::default()
            };
            apply_settings(&ctx, &settings);
            let p = palette(&settings);
            let mut rect = egui::Rect::NOTHING;
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    rect = button(ui, "取消", Button::Secondary, p).rect;
                });
            });
            let shape = output
                .shapes
                .iter()
                .find_map(|s| {
                    if let egui::Shape::Rect(r) = &s.shape {
                        if r.rect == rect {
                            Some(r)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap();
            bounds.push((rect.size(), shape.corner_radius));
            assert_eq!(shape.fill, neutral_surface(p));
            assert!(contrast(p.text, shape.fill) >= 4.5);
            if theme == Theme::Cavalry {
                assert_eq!(shape.stroke.width, 1.0);
                assert_eq!(shape.fill, p.surface);
            } else {
                assert_eq!(shape.stroke.width, 0.0);
                assert!(luminance(shape.fill) < luminance(p.panel));
            }
        }
        assert_eq!(bounds[0], bounds[1]);
    }
    #[test]
    fn shortcut_accent_rail_grows_and_brightens_without_moving_its_hit_region() {
        let ctx = egui::Context::default();
        crate::ui::fonts::setup(&ctx);
        let settings = Settings::default();
        apply_settings(&ctx, &settings);
        let p = palette(&settings);
        let mut hit = None;
        let mut rails = Vec::new();
        let mut cards = Vec::new();
        for time in [0.0, 0.1, 0.2, 0.5] {
            let events = hit
                .map(|rect: egui::Rect| egui::Event::PointerMoved(rect.center()))
                .into_iter()
                .collect();
            let mut current = egui::Rect::NOTHING;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(240.0, 120.0),
                    )),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        current = shortcut(
                            ui,
                            "脚本目录",
                            crate::ui::icons::Icon::Folder,
                            category(p, false),
                            p,
                        )
                        .rect;
                    });
                },
            );
            if let Some(previous) = hit {
                assert_eq!(current, previous);
            }
            hit = Some(current);
            let card = output
                .shapes
                .iter()
                .find_map(|clipped| {
                    if let egui::Shape::Rect(shape) = &clipped.shape {
                        (shape.stroke.color == category(p, false).color
                            && shape.rect.width() > current.width() - 20.0
                            && shape.rect.height() > current.height() - 10.0)
                            .then_some(shape.rect)
                    } else {
                        None
                    }
                })
                .expect("shortcut card");
            cards.push(card);
            let rail = output
                .shapes
                .iter()
                .find_map(|clipped| {
                    if let egui::Shape::Rect(shape) = &clipped.shape {
                        (shape.rect.width() <= 4.01
                            && shape.rect.height() >= 10.0
                            && shape.rect.left() >= current.left() - 0.01
                            && shape.rect.left() <= current.left() + 4.01)
                            .then_some((shape.rect, shape.fill))
                    } else {
                        None
                    }
                })
                .expect("shortcut accent rail");
            rails.push(rail);
            assert!(!output.shapes.iter().any(|clipped| {
                if let egui::Shape::Rect(shape) = &clipped.shape {
                    shape.rect.width() == 26.0 && shape.rect.height() == 26.0
                } else {
                    false
                }
            }));
        }
        assert_eq!(rails[1].0.height(), rails[0].0.height());
        assert!(rails[2].0.height() > rails[1].0.height());
        assert!((rails[3].0.height() - 18.24).abs() < 0.01);
        assert!(luminance(rails[2].1) > luminance(rails[0].1));
        assert!(hit.is_some_and(|rect| rails.iter().all(|(rail, _)| rect.contains_rect(*rail))));
        assert!(cards[2].width() > cards[0].width());
        assert!(cards[2].height() > cards[0].height());
        assert!(cards[3].width() >= cards[2].width());
        assert!(cards[3].height() >= cards[2].height());
        let hit = hit.unwrap();
        for card in cards {
            assert!(
                hit.expand(0.01).contains_rect(card),
                "card clipped: {card:?} / {hit:?}"
            );
        }
    }

    #[test]
    fn group_glyph_motion_never_overflows_or_confuses_danger_feedback() {
        for theme in [Theme::Cavalry, Theme::Light] {
            let p = palette(&Settings {
                theme,
                ..Default::default()
            });
            let hit = egui::Rect::from_min_size(egui::pos2(20.0, 20.0), Vec2::splat(CONTROL));
            for n in 0..=20 {
                let progress = n as f32 / 20.0;
                let glyph = action_glyph_rect(hit, Some(progress));
                assert!(hit.shrink(4.0).contains_rect(glyph));
                assert_ne!(
                    row_action_ink(p, progress),
                    p.danger,
                    "row hover alone is not a destructive hover"
                );
            }
            assert_eq!(action_glyph_rect(hit, None).center(), hit.center());
        }
    }
    #[test]
    fn reset_hover_has_no_filled_button_and_control_ink_is_optically_centered() {
        for theme in [Theme::Cavalry, Theme::Light] {
            let ctx = egui::Context::default();
            crate::ui::fonts::setup(&ctx);
            apply_settings(
                &ctx,
                &Settings {
                    theme,
                    ..Default::default()
                },
            );
            let p = palette(&Settings {
                theme,
                ..Default::default()
            });
            let mut rect = egui::Rect::NOTHING;
            for i in 0..6 {
                let output = ctx.run(
                    egui::RawInput {
                        time: Some(i as f64),
                        events: if i > 0 {
                            vec![egui::Event::PointerMoved(rect.center())]
                        } else {
                            vec![]
                        },
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .show(ctx, |ui| rect = text_action(ui, "恢复默认设置", p).rect);
                    },
                );
                if i == 5 {
                    for shape in output.shapes {
                        if let egui::Shape::Rect(r) = shape.shape {
                            if rect.contains_rect(r.rect) {
                                assert_eq!(r.fill.a(), 0, "reset hover must not acquire a plate");
                            }
                        }
                    }
                }
            }
            for language in crate::settings::Language::ALL {
                let _ = ctx.run(egui::RawInput::default(), |ctx| {
                    egui::CentralPanel::default()
                        .show(ctx, |ui| {
                            for label in [
                                language.tr(crate::i18n::Key::ScriptsFolder),
                                language.tr(crate::i18n::Key::Plugins),
                                language.tr(crate::i18n::Key::Save),
                            ] {
                                let g =
                                    ui.painter()
                                        .layout_no_wrap(label.into(), font(13.0), p.text);
                                let target = egui::Rect::from_min_size(
                                    egui::pos2(0.0, 0.0),
                                    Vec2::new(160.0, 40.0),
                                );
                                let pos = centered_ink(target, &g);
                                assert!(
                                    (ink_bounds(&g).translate(pos.to_vec2()).center()
                                        - target.center())
                                    .length()
                                        < 0.01
                                );
                            }
                        })
                        .inner
                });
            }
        }
    }
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
    fn pill_filters_have_one_inset_plate_and_fixed_full_height_hit_cells() {
        for theme in [Theme::Cavalry, Theme::Light] {
            let ctx = egui::Context::default();
            crate::ui::fonts::setup(&ctx);
            let settings = Settings {
                theme,
                ..Default::default()
            };
            apply_settings(&ctx, &settings);
            let p = palette(&settings);
            let mut cells = Vec::new();
            let mut area = egui::Rect::NOTHING;
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let start = ui.cursor().min;
                    cells = segmented(ui, &["All", "Scripts", "Plugins"], 0, 240.0, p).1;
                    area = egui::Rect::from_min_size(start, Vec2::new(240.0, CONTROL));
                });
            });
            assert_eq!(cells.len(), 3);
            for r in &cells {
                assert_eq!(r.height(), CONTROL);
                assert!(area.contains_rect(*r));
            }
            let plates: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Rect(r)
                        if r.rect.height() == CONTROL - 2.0 * XS && r.fill.a() > 0 =>
                    {
                        Some(r)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(plates.len(), 1);
            assert!(area.shrink(XS).contains_rect(plates[0].rect));
            assert_eq!(plates[0].corner_radius, egui::CornerRadius::same(16));
        }
    }
    #[test]
    fn green_blue_category_tokens_are_distinct_and_readable_on_their_badges() {
        for theme in [Theme::Cavalry, Theme::Light, Theme::Custom] {
            for background in [[0, 0, 0], [255, 255, 255], [128, 128, 128], [60, 20, 90]] {
                let p = palette(&Settings {
                    theme,
                    background,
                    ..Default::default()
                });
                let green = category(p, false);
                let blue = category(p, true);
                if theme != Theme::Custom {
                    assert_ne!(green.color, blue.color);
                }
                for tone in [green, blue] {
                    for surface in [p.bg, p.panel, p.surface, p.raised, tone.tint] {
                        assert!(
                            contrast(tone.color, surface) >= 4.49,
                            "{theme:?} {:?}",
                            tone.color
                        );
                    }
                }
                assert!(contrast(helper_ink(p), p.panel) >= 4.49);
            }
        }
    }
    #[test]
    fn switch_label_and_track_click_toggle_the_same_state_once() {
        for right in [false, true] {
            let ctx = egui::Context::default();
            crate::ui::fonts::setup(&ctx);
            apply(&ctx);
            let p = palette(&Settings::default());
            let mut value = false;
            let mut rect = egui::Rect::NOTHING;
            for frame in 0..6 {
                let mut events = Vec::new();
                if frame >= 3 {
                    let point = if right {
                        rect.right_center() - Vec2::new(18.0, 0.0)
                    } else {
                        rect.left_center() + Vec2::new(16.0, 0.0)
                    };
                    events.push(egui::Event::PointerMoved(point));
                    if frame == 4 || frame == 5 {
                        events.push(egui::Event::PointerButton {
                            pos: point,
                            button: egui::PointerButton::Primary,
                            pressed: frame == 4,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                }
                let _ = ctx.run(
                    egui::RawInput {
                        time: Some(frame as f64 / 60.0),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.set_width(280.0);
                            rect = toggle_row(ui, &mut value, "Reduce motion", p).rect;
                        });
                    },
                );
            }
            assert!(value);
            assert!(rect.height() >= 48.0);
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
