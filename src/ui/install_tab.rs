//! 安装区只显示固定拖放区；所有结果放在独立模态弹窗。

use egui::{CursorIcon, Pos2, Stroke, Vec2};

use super::app::CavalryKitApp;
use super::theme as t;

impl CavalryKitApp {
    pub(crate) fn show_install_stage(&mut self, ui: &mut egui::Ui) {
        // 拖放卡(comp-b:淡紫底 + 虚线内框 + 居中下载图标/标题/副行)
        let dragging = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let t_anim = ui
            .ctx()
            .animate_bool_with_time(egui::Id::new("ck-dropzone"), dragging, 0.2);

        let available = ui.available_size();
        // 拖放卡高度:无结果时占轨内余量(留 CTA),有结果时收敛
        let zone_h = (available.y - 220.0).clamp(180.0, 340.0);
        let (rect, resp) =
            ui.allocate_exact_size(Vec2::new(available.x, zone_h), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        // 悬停微交互(非拖入):轻微染色 + 虚线边加深
        let hov = t::interact_t(ui, resp.id, resp.hovered());
        // 底:淡紫(悬停微染;拖入时 primary 态更浓)
        let fill = if dragging {
            t::lerp_color(t::SURFACE_CONTAINER_LOW, t::PRIMARY_CONTAINER, t_anim * 0.6)
        } else {
            t::lerp_color(t::SURFACE_CONTAINER_LOW, t::SECONDARY_CONTAINER, hov * 0.5)
        };
        painter.rect_filled(rect, egui::CornerRadius::same(t::RADIUS_MD), fill);
        // 虚线内框:拖入时主色实边;悬停时边色加深
        if dragging {
            painter.rect(
                rect.shrink(4.0),
                egui::CornerRadius::same(t::RADIUS_MD - 4),
                egui::Color32::TRANSPARENT,
                Stroke::new(2.0_f32, t::PRIMARY),
                egui::StrokeKind::Inside,
            );
        } else {
            t::dashed_rect(
                &painter,
                rect,
                t::RADIUS_MD - 2,
                t::lerp_color(t::OUTLINE_VARIANT, t::PRIMARY, hov * 0.6),
            );
        }
        if ui
            .ctx()
            .input(|i| i.pointer.latest_pos().is_some_and(|p| rect.contains(p)))
            || dragging
        {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        // 居中:下载图标 + 标题 + 副行(内容块整体垂直居中;悬停时整体上浮 2px)
        let cx = rect.center().x;
        let content_h = 96.0_f32; // 图标 38 + 16 + 标题 20 + 6 + 副行 16
        let icon_top = rect.center().y - content_h / 2.0 - hov * 2.0;
        // 下载箭头(Material Symbols 式:竖杆 + 箭头 + 底座)
        let col = if dragging {
            t::PRIMARY
        } else {
            t::ON_PRIMARY_CONTAINER
        };
        let stem_top = Pos2::new(cx, icon_top + 6.0);
        let stem_bot = Pos2::new(cx, icon_top + 26.0);
        painter.line_segment([stem_top, stem_bot], Stroke::new(2.5_f32, col));
        painter.line_segment(
            [Pos2::new(cx - 7.0, stem_bot.y - 8.0), stem_bot],
            Stroke::new(2.5_f32, col),
        );
        painter.line_segment(
            [Pos2::new(cx + 7.0, stem_bot.y - 8.0), stem_bot],
            Stroke::new(2.5_f32, col),
        );
        painter.line_segment(
            [
                Pos2::new(cx - 11.0, stem_bot.y + 6.0),
                Pos2::new(cx + 11.0, stem_bot.y + 6.0),
            ],
            Stroke::new(2.5_f32, col),
        );
        painter.line_segment(
            [
                Pos2::new(cx - 11.0, stem_bot.y - 1.0),
                Pos2::new(cx - 11.0, stem_bot.y + 6.0),
            ],
            Stroke::new(2.5_f32, col),
        );
        painter.line_segment(
            [
                Pos2::new(cx + 11.0, stem_bot.y - 1.0),
                Pos2::new(cx + 11.0, stem_bot.y + 6.0),
            ],
            Stroke::new(2.5_f32, col),
        );

        let title = painter.layout_no_wrap(
            "把脚本和插件拖进来".to_owned(),
            t::font_medium(t::TITLE_MD),
            t::ON_PRIMARY_CONTAINER,
        );
        let title_w = title.size().x;
        let title_h = title.size().y;
        painter.galley(
            Pos2::new(cx - title_w / 2.0, stem_bot.y + 16.0),
            title,
            t::ON_PRIMARY_CONTAINER,
        );
        let sub = painter.layout_no_wrap(
            ".jsc / .js / .jsx / 压缩包".to_owned(),
            t::font(t::BODY_SM),
            t::ON_SURFACE_VARIANT,
        );
        painter.galley(
            Pos2::new(cx - sub.size().x / 2.0, stem_bot.y + 16.0 + title_h + 6.0),
            sub,
            t::ON_SURFACE_VARIANT,
        );

        // 流水线状态(卡内底部)
        let status: Option<(String, egui::Color32)> = if !self.conflicts.is_empty() {
            Some((
                format!("等待处理 {} 个冲突…", self.conflicts.len()),
                t::ERROR,
            ))
        } else if let Some(status) = self.busy {
            Some((status.to_owned(), t::PRIMARY))
        } else if !self.queue.is_empty() {
            Some((
                format!("正在准备 {} 个输入…", self.queue.len()),
                t::ON_SURFACE_VARIANT,
            ))
        } else {
            None
        };
        if let Some((s, c)) = status {
            let g = painter.layout_no_wrap(s, t::font(t::LABEL_MD), c);
            painter.galley(
                Pos2::new(cx - g.size().x / 2.0, rect.max.y - 10.0 - g.size().y),
                g,
                c,
            );
        }

        ui.add_space(10.0);

        // 主 CTA:全宽紫 pill
        let btn_w = ui.available_width();
        if ui
            .scope(|ui| {
                // 满宽 filled 按钮
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(btn_w, 40.0), egui::Sense::click());
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        ui.is_enabled(),
                        "选择文件…",
                    )
                });
                if response.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }
                let k = t::interact_t(ui, response.id, response.hovered());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, egui::CornerRadius::same(20), t::PRIMARY);
                if k > 0.0 {
                    painter.rect_filled(
                        rect,
                        egui::CornerRadius::same(20),
                        t::with_alpha(t::ON_PRIMARY, k * 0.08),
                    );
                }
                let g = ui.painter().layout_no_wrap(
                    "选择文件…".to_owned(),
                    t::font_medium(t::LABEL_LG),
                    t::ON_PRIMARY,
                );
                painter.galley(rect.center() - g.size() / 2.0, g, t::ON_PRIMARY);
                response
            })
            .inner
            .clicked()
        {
            self.pick_files(ui.ctx());
        }
    }
}
