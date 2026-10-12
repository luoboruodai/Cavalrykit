//! Google Material Design 3(baseline light scheme)→ egui 落地。
//!
//! 结构:色令牌 → 字阶 → 状态层/动效 → 全局样式 → 组件(filled/tonal/text/outlined
//!      按钮、chip、segmented、搜索框、图标块、列表行、对话框、snackbar)。
//! 体系约束:tonal surface 阶梯做层级(#FEF7FF→#F3EDF7→#ECE6F0→#E7E0EC);pill 按钮;
//!          状态层 hover 8% / pressed 12% primary 叠罩;圆角 md 12 / xl 28(对话框);
//!          无渐变无硬影;主色紫 #6750A4 只作 CTA/选中。

use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Pos2, Response, RichText, Sense,
    Stroke, StrokeKind, Ui, Vec2,
};

use super::fonts::{FAMILY_MEDIUM, FAMILY_SEMIBOLD};

// ==================== Tokens — MD3 baseline light scheme ====================
pub const PRIMARY: Color32 = Color32::from_rgb(0x67, 0x50, 0xa4); // 主紫(CTA/选中)
pub const ON_PRIMARY: Color32 = Color32::WHITE;
pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0xea, 0xdd, 0xff); // 淡紫底
pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0x21, 0x00, 0x5d);
pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(0xe8, 0xde, 0xf8);
pub const SURFACE: Color32 = Color32::from_rgb(0xfe, 0xf7, 0xff);
pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(0xf7, 0xf2, 0xfa);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(0xf3, 0xed, 0xf7);
pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(0xec, 0xe6, 0xf0); // 对话框
pub const SURFACE_VARIANT: Color32 = Color32::from_rgb(0xe7, 0xe0, 0xec); // 图标块底
pub const ON_SURFACE: Color32 = Color32::from_rgb(0x1d, 0x1b, 0x20);
pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(0x49, 0x45, 0x4f);
pub const OUTLINE: Color32 = Color32::from_rgb(0x79, 0x74, 0x7e);
pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(0xca, 0xc4, 0xd0);
pub const ERROR: Color32 = Color32::from_rgb(0xb3, 0x26, 0x1e);
pub const ERROR_CONTAINER: Color32 = Color32::from_rgb(0xf9, 0xde, 0xdc);
pub const INVERSE_SURFACE: Color32 = Color32::from_rgb(0x32, 0x2f, 0x35); // snackbar
pub const INVERSE_ON_SURFACE: Color32 = Color32::from_rgb(0xf5, 0xef, 0xf7);
pub const SUCCESS_CONTAINER: Color32 = Color32::from_rgb(0xc4, 0xee, 0xd0);
pub const ON_SUCCESS_CONTAINER: Color32 = Color32::from_rgb(0x00, 0x21, 0x0a);

// ==================== Tokens — 状态层 ====================
/// primary 8% 悬停罩
pub const STATE_HOVER: Color32 = Color32::from_rgba_premultiplied(0x0d, 0x0a, 0x14, 0x14);
/// primary 12% 按下罩
pub const STATE_PRESS: Color32 = Color32::from_rgba_premultiplied(0x13, 0x0e, 0x1f, 0x1f);
/// on-surface 8%(中性悬停)
pub const STATE_HOVER_NEUTRAL: Color32 = Color32::from_rgba_premultiplied(0x02, 0x02, 0x02, 0x0f); // on-surface 6%(要可见)

// ==================== Tokens — Radii(MD3 角阶) ====================
pub const RADIUS_XS: u8 = 4;
pub const RADIUS_SM: u8 = 8;
pub const RADIUS_MD: u8 = 12;
pub const RADIUS_LG: u8 = 16;
pub const RADIUS_XL: u8 = 28; // 对话框
pub const PILL: f32 = 0.5; // 满丸 = 高/2

// ==================== 字阶(MD3 Type Scale) ====================
pub const TITLE_LG: f32 = 22.0; // 页标题 Regular
pub const TITLE_MD: f32 = 16.0; // Medium
pub const TITLE_SM: f32 = 14.0; // Medium
pub const BODY_LG: f32 = 16.0;
pub const BODY_MD: f32 = 14.0;
pub const BODY_SM: f32 = 12.0;
pub const LABEL_LG: f32 = 14.0; // Medium(按钮/链接)
pub const LABEL_MD: f32 = 12.0;
pub const LABEL_SM: f32 = 11.0;

pub fn font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn font_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FAMILY_MEDIUM.into()))
}

pub fn font_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FAMILY_SEMIBOLD.into()))
}

/// Body(400)
pub fn body(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text.into()).font(font(size))
}

/// Label/Title Medium(500)
pub fn medium(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text.into()).font(font_medium(size))
}

/// 强调(700,少量用)
pub fn bold(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text.into()).font(font_bold(size))
}

// ==================== 动效(MD3 easing-standard 200ms) ====================
pub const ANIM_HOVER: f32 = 0.25;

/// 悬停/按下混合系数
pub fn interact_t(ui: &Ui, id: egui::Id, hovered: bool) -> f32 {
    if hovered && ui.ctx().input(|i| i.pointer.primary_down()) {
        return 1.0;
    }
    ui.ctx().animate_bool_with_time(id, hovered, ANIM_HOVER)
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    // 预乘空间插值(含 alpha)
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

pub fn fade(c: Color32, t: f32) -> Color32 {
    c.gamma_multiply(t.clamp(0.0, 1.0))
}

pub fn with_alpha(c: Color32, t: f32) -> Color32 {
    // 预乘空间:缩放 alpha 必须连同 rgb 一起缩(linear_multiply),
    // from_rgba_unmultiplied 会把预乘通道读成实色(深色罩变黑块的教训)
    c.linear_multiply(t.clamp(0.0, 1.0))
}

fn point_hand(ui: &Ui, hovered: bool) {
    if hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}

// ==================== 全局样式 ====================

/// level-1 柔影(轻微有色晕)
pub fn elevation_1() -> egui::Shadow {
    egui::Shadow {
        offset: [0, 2],
        blur: 6,
        spread: 0,
        color: Color32::from_black_alpha(24),
    }
}

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let v = &mut style.visuals;
    v.dark_mode = false;

    v.panel_fill = SURFACE;
    v.window_fill = SURFACE;
    v.extreme_bg_color = SURFACE_CONTAINER_HIGH;
    v.faint_bg_color = SURFACE_CONTAINER;
    v.hyperlink_color = PRIMARY;
    v.error_fg_color = ERROR;
    v.warn_fg_color = ERROR;
    v.window_stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);
    v.window_shadow = elevation_1();
    v.popup_shadow = elevation_1();
    v.window_corner_radius = CornerRadius::same(RADIUS_XL);
    v.menu_corner_radius = CornerRadius::same(RADIUS_SM);

    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.fg_stroke = Stroke::new(1.0_f32, ON_SURFACE);
    }
    v.widgets.noninteractive.bg_fill = SURFACE;
    v.widgets.noninteractive.weak_bg_fill = SURFACE;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);
    v.widgets.inactive.bg_fill = SURFACE_CONTAINER;
    v.widgets.inactive.weak_bg_fill = SURFACE_CONTAINER;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);
    v.widgets.hovered.bg_fill = SURFACE_CONTAINER_HIGH;
    v.widgets.hovered.weak_bg_fill = SURFACE_CONTAINER_HIGH;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, OUTLINE);
    v.widgets.active.bg_fill = SURFACE_CONTAINER_HIGH;
    v.widgets.active.weak_bg_fill = SURFACE_CONTAINER_HIGH;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, OUTLINE);

    v.selection.bg_fill = PRIMARY_CONTAINER;
    v.selection.stroke = Stroke::new(1.0_f32, PRIMARY);

    use egui::TextStyle::*;
    style
        .text_styles
        .insert(Body, FontId::new(BODY_MD, FontFamily::Proportional));
    style
        .text_styles
        .insert(Small, FontId::new(BODY_SM, FontFamily::Proportional));
    style
        .text_styles
        .insert(Button, FontId::new(LABEL_LG, FontFamily::Proportional));
    style
        .text_styles
        .insert(Heading, FontId::new(TITLE_LG, FontFamily::Proportional));
    style
        .text_styles
        .insert(Monospace, FontId::new(BODY_SM, FontFamily::Monospace));

    style.spacing.button_padding = Vec2::splat(0.0);
    style.spacing.item_spacing = Vec2::new(12.0, 8.0);
    style.spacing.scroll.foreground_color = false;
    style.spacing.scroll.bar_width = 10.0;
    style.spacing.scroll.handle_min_length = 48.0;

    ctx.set_style(style);
}

// ==================== 按钮(MD3:filled / tonal / outlined / text) ====================

#[derive(Clone, Copy, PartialEq)]
pub enum Btn {
    /// 实底紫:主动作
    Filled,
    /// tonal 淡紫:次强调
    Tonal,
    /// 描边:中性动作
    Outlined,
    /// 纯文字(可自定义色):行内动作
    Text(Color32),
}

pub fn button(ui: &mut Ui, text: impl Into<String>, variant: Btn, height: f32) -> Response {
    let (text_color, font) = match variant {
        Btn::Filled => (ON_PRIMARY, font_medium(LABEL_LG)),
        Btn::Tonal => (ON_PRIMARY_CONTAINER, font_medium(LABEL_LG)),
        Btn::Outlined => (PRIMARY, font_medium(LABEL_LG)),
        Btn::Text(c) => (c, font_medium(LABEL_LG)),
    };
    let text = text.into();
    let galley = ui.painter().layout_no_wrap(text.clone(), font, text_color);
    let pad_x = 20.0_f32;
    let size = Vec2::new(galley.size().x + pad_x * 2.0, height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), &text)
    });
    point_hand(ui, response.hovered());
    let t = interact_t(ui, response.id, response.hovered());
    let pressed = response.hovered() && ui.ctx().input(|i| i.pointer.primary_down());
    let radius = CornerRadius::same((height / 2.0).round() as u8);
    let painter = ui.painter_at(rect);

    let fill = match variant {
        Btn::Filled => PRIMARY,
        Btn::Tonal => SECONDARY_CONTAINER,
        Btn::Outlined => Color32::TRANSPARENT,
        Btn::Text(_) => Color32::TRANSPARENT,
    };
    painter.rect_filled(rect, radius, fill);
    // 状态层叠罩
    if pressed {
        painter.rect_filled(rect, radius, STATE_PRESS);
    } else if t > 0.0 {
        painter.rect_filled(rect, radius, with_alpha(STATE_HOVER, t));
    }
    if variant == Btn::Outlined {
        painter.rect(
            rect,
            radius,
            Color32::TRANSPARENT,
            Stroke::new(1.0_f32, OUTLINE),
            StrokeKind::Inside,
        );
    }
    painter.galley(
        Pos2::new(rect.min.x + pad_x, rect.center().y - galley.size().y / 2.0),
        galley,
        text_color,
    );
    response
}

// ==================== Chip(状态指示) ====================

/// outlined 状态 chip:圆点 + 文字,圆角 8
pub fn status_chip(ui: &mut Ui, running: bool, text: impl Into<String>) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.into(), font_medium(LABEL_MD), ON_SURFACE_VARIANT);
    let (pad_x, gap, dot_r, h) = (12.0_f32, 8.0_f32, 5.0_f32, 32.0_f32);
    let size = Vec2::new(galley.size().x + pad_x * 2.0 + gap + dot_r * 2.0, h);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        CornerRadius::same((32.0_f32 / 2.0).round() as u8),
        SURFACE,
        Stroke::new(1.0_f32, OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    let dot = if running {
        Color32::from_rgb(0x1e, 0x87, 0x46) // 语义绿
    } else {
        OUTLINE
    };
    let mut x = rect.min.x + pad_x;
    painter.circle_filled(Pos2::new(x + dot_r, rect.center().y), dot_r, dot);
    x += dot_r * 2.0 + gap;
    painter.galley(
        Pos2::new(x, rect.center().y - galley.size().y / 2.0),
        galley,
        ON_SURFACE_VARIANT,
    );
}

// ==================== Segmented 筛选(M3 segmented buttons) ====================

/// segmented 组(主题色丸式):SURFACE 白块底盘(与搜索框/列表卡同 treatment),
/// 选中 = 内缩主题紫丸(PRIMARY 底白字)滑动过槽,无分隔线;返回新选中下标
pub fn segmented(ui: &mut Ui, labels: &[&str], selected: usize) -> usize {
    let h = 36.0_f32;
    let pad_x = 22.0_f32;
    let track_r = RADIUS_MD;
    let pill_inset = 4.0_f32; // 紫丸相对槽位四边均匀内缩(嵌套圆角:8+4=12,两轴一致)
                              // 等宽段:取最宽标签统一槽宽
    let mut text_w = 0.0_f32;
    for l in labels {
        let g = ui
            .painter()
            .layout_no_wrap((*l).to_owned(), font_medium(LABEL_LG), ON_SURFACE);
        text_w = text_w.max(g.size().x);
    }
    let seg_w = text_w + pad_x * 2.0;
    let total_w = seg_w * labels.len() as f32;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, h), Sense::hover());
    let painter = ui.painter_at(rect);
    // 底盘:白块 + 细边(同搜索框/列表卡)
    painter.rect(
        rect,
        CornerRadius::same(track_r),
        SURFACE,
        Stroke::new(1.0_f32, OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    let inner_painter = ui.painter_at(rect);

    let mut new_selected = selected;
    // 滑动紫丸:animate_value 平滑过槽(选中切换动效)
    let anim =
        ui.ctx()
            .animate_value_with_time(egui::Id::new("ck-seg-slide"), selected as f32, 0.22);
    let idx_f = anim;
    let i0 = idx_f.floor() as usize;
    let i1 = (i0 + 1).min(labels.len() - 1);
    let frac = idx_f - i0 as f32;
    let ind_x = rect.min.x + (i0 as f32 + (i1 as f32 - i0 as f32) * frac) * seg_w;
    let slot = egui::Rect::from_min_size(
        Pos2::new(ind_x, rect.min.y),
        Vec2::new(seg_w, rect.height()),
    );
    // 选中态:内缩主题紫丸(PRIMARY 底)
    let pill = slot.shrink(pill_inset);
    {
        let clipped = inner_painter.with_clip_rect(rect);
        clipped.rect_filled(pill, CornerRadius::same(RADIUS_SM), PRIMARY);
    }

    for (i, l) in labels.iter().enumerate() {
        let seg = egui::Rect::from_min_size(
            Pos2::new(rect.min.x + i as f32 * seg_w, rect.min.y),
            Vec2::new(seg_w, h),
        );
        let resp = ui.interact(seg, egui::Id::new(("ck-seg", *l)), Sense::click());
        point_hand(ui, resp.hovered());
        let active = i == selected;
        resp.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                ui.is_enabled(),
                active,
                *l,
            )
        });
        // 悬停/按下罩(未选中项,与紫丸同内缩同圆角)
        let t = interact_t(ui, resp.id, resp.hovered());
        if !active && t > 0.0 {
            inner_painter.rect_filled(
                seg.shrink(pill_inset),
                CornerRadius::same(RADIUS_SM),
                with_alpha(STATE_HOVER_NEUTRAL, t),
            );
        }
        // 文字:选中白(随滑动淡入),未选中 variant 灰(悬停提亮)
        let sel_t =
            ui.ctx()
                .animate_bool_with_time(egui::Id::new(("ck-seg-sel", *l)), active, 0.22);
        let mut col = lerp_color(ON_SURFACE_VARIANT, ON_PRIMARY, sel_t);
        if !active && t > 0.0 {
            col = lerp_color(col, ON_SURFACE, t * 0.7);
        }
        let g = ui
            .painter()
            .layout_no_wrap((*l).to_owned(), font_medium(LABEL_LG), col);
        painter.galley(
            Pos2::new(
                seg.center().x - g.size().x / 2.0,
                seg.center().y - g.size().y / 2.0,
            ),
            g,
            col,
        );
        if resp.clicked() {
            new_selected = i;
        }
    }
    new_selected
}

/// 紧凑状态 chip(品牌行同行):小点 + 小字,满丸描边
pub fn status_chip_compact(ui: &mut Ui, running: bool, text: impl Into<String>) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.into(), font_medium(LABEL_SM), ON_SURFACE_VARIANT);
    let (pad_x, gap, dot_r, h) = (8.0_f32, 5.0_f32, 3.5_f32, 22.0_f32);
    let size = Vec2::new(galley.size().x + pad_x * 2.0 + gap + dot_r * 2.0, h);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        CornerRadius::same((h / 2.0).round() as u8),
        SURFACE,
        Stroke::new(1.0_f32, OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    let dot = if running {
        Color32::from_rgb(0x1e, 0x87, 0x46)
    } else {
        OUTLINE
    };
    painter.circle_filled(
        Pos2::new(rect.min.x + pad_x + dot_r, rect.center().y),
        dot_r,
        dot,
    );
    painter.galley(
        Pos2::new(
            rect.min.x + pad_x + dot_r * 2.0 + gap,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        ON_SURFACE_VARIANT,
    );
}

// ==================== 搜索框(M3 text field,圆角 8,outline 边) ====================

/// 底 → put → 描边/图标 三层顺序(占位提示必须在最上)
pub fn search_input(ui: &mut Ui, text: &mut String, width: f32) -> Response {
    let height = 40.0_f32;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, CornerRadius::same(RADIUS_SM), SURFACE);

    let edit = egui::TextEdit::singleline(text)
        .id(egui::Id::new("ck-search"))
        .frame(false)
        .font(font(BODY_MD))
        .desired_width(f32::INFINITY)
        .margin(egui::Margin::ZERO)
        .hint_text(
            RichText::new("搜索装备库")
                .font(font(BODY_MD))
                .color(with_alpha(ON_SURFACE_VARIANT, 0.55)),
        );
    // 文字垂直居中:按字体行高居中放置 TextEdit
    let text_h = ui.ctx().fonts(|f| f.row_height(&font(BODY_MD)));
    let edit_y = rect.min.y + (rect.height() - text_h) / 2.0;
    let edit_rect = egui::Rect::from_min_size(
        Pos2::new(rect.min.x + 40.0, edit_y),
        Vec2::new(rect.width() - 46.0, text_h + 4.0),
    );
    let resp = ui.put(edit_rect, edit);

    let hot = resp.has_focus();
    let hov = rect_contains_pointer(ui, rect);
    let hov_t = ui
        .ctx()
        .animate_bool_with_time(egui::Id::new("ck-search-hov"), hov, ANIM_HOVER);
    // 悬停 = 粗主题色高亮(MD3 基调):2px 主紫边;底不染(MD3 悬停不靠填色)
    let stroke = if hot || hov_t > 0.5 {
        Stroke::new(2.0_f32, PRIMARY)
    } else {
        Stroke::new(lerp_f(1.0, 2.0, hov_t), lerp_color(OUTLINE, PRIMARY, hov_t))
    };
    painter.rect(
        rect,
        CornerRadius::same(RADIUS_SM),
        Color32::TRANSPARENT,
        stroke,
        StrokeKind::Inside,
    );
    // 放大镜图标:悬停/聚焦转主紫(微反应)
    let icon_col = if hot {
        PRIMARY
    } else {
        lerp_color(ON_SURFACE_VARIANT, PRIMARY, hov_t)
    };
    let c = Pos2::new(rect.min.x + 20.0, rect.center().y);
    painter.circle_stroke(c, 5.5, Stroke::new(1.8_f32, icon_col));
    painter.line_segment(
        [c + Vec2::new(3.6, 3.6), c + Vec2::new(7.5, 7.5)],
        Stroke::new(1.8_f32, icon_col),
    );
    resp
}

fn lerp_f(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

// ==================== 图标块(40px,surface-variant 底) ====================

pub fn icon_tile(painter: &egui::Painter, center: Pos2, glyph: &str) {
    let tile = egui::Rect::from_center_size(center, Vec2::splat(40.0));
    painter.rect_filled(tile, CornerRadius::same(RADIUS_MD), PRIMARY);
    let g = painter.layout_no_wrap(glyph.to_owned(), font_medium(12.0), ON_PRIMARY);
    painter.galley(tile.center() - g.size() / 2.0, g, ON_PRIMARY);
}

// ==================== 对话框 / snackbar ====================

/// 对话框卡(surface-container-high,圆角 28)
pub fn dialog_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(SURFACE_CONTAINER_HIGH)
        .corner_radius(CornerRadius::same(RADIUS_XL))
        .inner_margin(24.0)
        .stroke(Stroke::NONE)
        .shadow(elevation_1())
}

/// 遮罩(32% 黑)
pub fn dim_screen(ctx: &egui::Context, id: &str, alpha: f32) {
    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Foreground)
        .interactable(true)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            ui.set_min_size(screen.size());
            ui.painter().rect_filled(
                screen,
                0,
                Color32::from_black_alpha((alpha.clamp(0.0, 0.32) * 255.0) as u8),
            );
        });
}

/// 居中对话框容器(t:入场进度)
pub fn modal_area(ctx: &egui::Context, id: &str, t: f32, add_contents: impl FnOnce(&mut Ui)) {
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, (1.0 - t) * 12.0))
        .show(ctx, add_contents);
}

/// 虚线描边(拖放区内框):沿 rect 画断续线
pub fn dashed_rect(painter: &egui::Painter, rect: egui::Rect, radius: u8, color: Color32) {
    let dash = 6.0_f32;
    let gap = 4.0_f32;
    // egui 无虚线 API:按线段步进
    let draw_dashes = |from: Pos2, to: Pos2| {
        let len = (to - from).length();
        let dir = (to - from).normalized();
        let mut d = 0.0;
        while d < len {
            let seg_len = dash.min(len - d);
            painter.line_segment(
                [from + dir * d, from + dir * (d + seg_len)],
                Stroke::new(1.5_f32, color),
            );
            d += dash + gap;
        }
    };
    let r = rect.shrink(4.0); // 内缩,圆角忽略(虚线感已足)
    draw_dashes(
        Pos2::new(r.min.x + radius as f32, r.min.y),
        Pos2::new(r.max.x - radius as f32, r.min.y),
    );
    draw_dashes(
        Pos2::new(r.max.x, r.min.y + radius as f32),
        Pos2::new(r.max.x, r.max.y - radius as f32),
    );
    draw_dashes(
        Pos2::new(r.max.x - radius as f32, r.max.y),
        Pos2::new(r.min.x + radius as f32, r.max.y),
    );
    draw_dashes(
        Pos2::new(r.min.x, r.max.y - radius as f32),
        Pos2::new(r.min.x, r.min.y + radius as f32),
    );
    // 四角用圆弧衔接
    for (cx, cy, a0, a1) in [
        (
            r.min.x + radius as f32,
            r.min.y + radius as f32,
            180.0_f32,
            270.0_f32,
        ),
        (
            r.max.x - radius as f32,
            r.min.y + radius as f32,
            270.0_f32,
            360.0_f32,
        ),
        (
            r.max.x - radius as f32,
            r.max.y - radius as f32,
            0.0_f32,
            90.0_f32,
        ),
        (
            r.min.x + radius as f32,
            r.max.y - radius as f32,
            90.0_f32,
            180.0_f32,
        ),
    ] {
        let n = 6;
        let pts: Vec<Pos2> = (0..=n)
            .map(|i| {
                let a = (a0 + (a1 - a0) * i as f32 / n as f32).to_radians();
                Pos2::new(cx + radius as f32 * a.cos(), cy + radius as f32 * a.sin())
            })
            .collect();
        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], Stroke::new(1.5_f32, color));
        }
    }
}

/// 指针是否在 rect 上(自绘框 hover 检测用)
fn rect_contains_pointer(ui: &Ui, rect: egui::Rect) -> bool {
    ui.ctx()
        .input(|i| i.pointer.latest_pos().is_some_and(|p| rect.contains(p)))
}
