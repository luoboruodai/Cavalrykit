//! 安装区(左栏):拖放卡(淡紫底 + 虚线内框 + 下载图标)/ 结果卡(白底 + 结果行)。

use egui::{CursorIcon, Pos2, RichText, Stroke, Vec2};

use super::app::CavalryKitApp;
use super::theme as t;

impl CavalryKitApp {
    pub(crate) fn show_install_stage(&mut self, ui: &mut egui::Ui) {
        // 有结果:结果卡(白底 + 结果行 + 清空)
        if let Some(outcome) = &self.outcome {
            let t_in = ui
                .ctx()
                .animate_bool_with_time(egui::Id::new("ck-result-in"), true, 0.2);
            let mut clear = false;
            egui::Frame::new()
                .fill(t::fade(t::SURFACE, t_in))
                .stroke(Stroke::new(1.0_f32, t::OUTLINE_VARIANT))
                .corner_radius(egui::CornerRadius::same(t::RADIUS_MD))
                .inner_margin(12.0)
                .shadow(t::elevation_1())
                .show(ui, |ui| {
                    let ok = outcome.installed_scripts.len() + outcome.installed_plugins.len();
                    ui.horizontal(|ui| {
                        ui.label(
                            t::medium("安装结果", t::TITLE_SM).color(t::fade(t::ON_SURFACE, t_in)),
                        );
                        ui.allocate_ui_with_layout(
                            Vec2::new(ui.available_width(), 22.0),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if t::button(ui, "清空", t::Btn::Text(t::PRIMARY), 26.0).clicked() {
                                    clear = true;
                                }
                            },
                        );
                    });
                    ui.add_space(6.0);
                    egui::ScrollArea::vertical()
                        .id_salt("ck-install-results")
                        .auto_shrink([false, true])
                        .max_height(120.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for name in outcome
                                .installed_scripts
                                .iter()
                                .chain(outcome.installed_plugins.iter())
                            {
                                result_row(ui, true, name, t_in);
                            }
                            for name in &outcome.skipped {
                                result_row(ui, false, &format!("{name}(跳过)"), t_in);
                            }
                            for (name, reason) in &outcome.failures {
                                result_row(ui, false, &format!("{name} — {reason}"), t_in);
                            }
                        });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "成功 {ok} · 跳过 {} · 失败 {}",
                            outcome.skipped.len(),
                            outcome.failures.len()
                        ))
                        .font(t::font(t::LABEL_MD))
                        .color(t::fade(t::ON_SURFACE_VARIANT, t_in)),
                    );
                    if outcome.restart_hint {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("已安装插件:需重启 Cavalry 后生效")
                                .font(t::font(t::LABEL_MD))
                                .color(t::fade(t::ERROR, t_in)),
                        );
                    }
                });
            ui.add_space(10.0);
            if clear {
                self.outcome = None;
            }
        }

        // 拖放卡(comp-b:淡紫底 + 虚线内框 + 居中下载图标/标题/副行)
        let dragging = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let t_anim = ui
            .ctx()
            .animate_bool_with_time(egui::Id::new("ck-dropzone"), dragging, 0.2);

        let available = ui.available_size();
        // 拖放卡高度:无结果时占轨内余量(留 CTA),有结果时收敛
        let zone_h = if self.outcome.is_none() {
            (available.y * 0.56).clamp(180.0, 460.0)
        } else {
            120.0
        };
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(available.x, zone_h), egui::Sense::hover());
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
        if ui.ctx().input(|i| i.pointer.latest_pos().map_or(false, |p| rect.contains(p))) || dragging {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        // 居中:下载图标 + 标题 + 副行(内容块整体垂直居中;悬停时整体上浮 2px)
        let cx = rect.center().x;
        let content_h = 96.0_f32; // 图标 38 + 16 + 标题 20 + 6 + 副行 16
        let icon_top = rect.center().y - content_h / 2.0 - hov * 2.0;
        // 下载箭头(Material Symbols 式:竖杆 + 箭头 + 底座)
        let col = if dragging { t::PRIMARY } else { t::ON_PRIMARY_CONTAINER };
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
            [Pos2::new(cx - 11.0, stem_bot.y + 6.0), Pos2::new(cx + 11.0, stem_bot.y + 6.0)],
            Stroke::new(2.5_f32, col),
        );
        painter.line_segment(
            [Pos2::new(cx - 11.0, stem_bot.y - 1.0), Pos2::new(cx - 11.0, stem_bot.y + 6.0)],
            Stroke::new(2.5_f32, col),
        );
        painter.line_segment(
            [Pos2::new(cx + 11.0, stem_bot.y - 1.0), Pos2::new(cx + 11.0, stem_bot.y + 6.0)],
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
            Some((format!("等待处理 {} 个冲突…", self.conflicts.len()), t::ERROR))
        } else if !self.queue.is_empty() {
            Some((format!("正在准备 {} 个输入…", self.queue.len()), t::ON_SURFACE_VARIANT))
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
                if response.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }
                let k = t::interact_t(ui, response.id, response.hovered());
                let painter = ui.painter_at(rect);
                painter.rect_filled(
                    rect,
                    egui::CornerRadius::same(20),
                    t::PRIMARY,
                );
                if k > 0.0 {
                    painter.rect_filled(
                        rect,
                        egui::CornerRadius::same(20),
                        t::with_alpha(t::ON_PRIMARY, k * 0.08),
                    );
                }
                let g = ui
                    .painter()
                    .layout_no_wrap("选择文件…".to_owned(), t::font_medium(t::LABEL_LG), t::ON_PRIMARY);
                painter.galley(rect.center() - g.size() / 2.0, g, t::ON_PRIMARY);
                response
            })
            .inner
            .clicked()
        {
            if let Some(files) = rfd::FileDialog::new()
                .add_filter(
                    "Cavalry 文件",
                    &["jsc", "js", "jsx", "zip", "7z", "tar", "tgz", "tbz2", "gz", "bz2"],
                )
                .add_filter("所有文件", &["*"])
                .pick_files()
            {
                self.enqueue(files);
            }
        }
    }
}

/// 结果行:状态图标(对勾/叉)+ 名称
fn result_row(ui: &mut egui::Ui, ok: bool, text: &str, t_in: f32) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 22.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let c = rect.center();
        let (bg, fg) = if ok {
            (t::SUCCESS_CONTAINER, t::ON_SUCCESS_CONTAINER)
        } else {
            (t::ERROR_CONTAINER, t::ERROR)
        };
        painter.circle_filled(c, 7.0, t::fade(bg, t_in));
        // 对勾/叉
        if ok {
            painter.line_segment(
                [c + Vec2::new(-3.5, 0.0), c + Vec2::new(-1.0, 2.5)],
                Stroke::new(1.8_f32, t::fade(fg, t_in)),
            );
            painter.line_segment(
                [c + Vec2::new(-1.0, 2.5), c + Vec2::new(4.0, -3.0)],
                Stroke::new(1.8_f32, t::fade(fg, t_in)),
            );
        } else {
            painter.line_segment(
                [c + Vec2::new(-2.5, -2.5), c + Vec2::new(2.5, 2.5)],
                Stroke::new(1.8_f32, t::fade(fg, t_in)),
            );
            painter.line_segment(
                [c + Vec2::new(2.5, -2.5), c + Vec2::new(-2.5, 2.5)],
                Stroke::new(1.8_f32, t::fade(fg, t_in)),
            );
        }
        ui.label(
            RichText::new(text)
                .font(t::font(t::BODY_SM))
                .color(t::fade(t::ON_SURFACE, t_in)),
        );
    });
}
