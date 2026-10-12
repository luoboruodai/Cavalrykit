//! Cached filtering and virtualized rows: frame work is proportional to visible entries.
use super::app::{CavalryKitApp, Filter, Row};
use super::theme as t;
use crate::paths;
use egui::{CursorIcon, Pos2, RichText, Sense, Stroke, StrokeKind, Vec2};

impl CavalryKitApp {
    pub(crate) fn show_library(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(t::medium("Cavalry 装备库", 24.0).color(t::ON_SURFACE));
            count_badge(ui, self.library.scripts.len() + self.library.plugins.len());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(self.busy.is_none() && self.conflicts.is_empty(), |ui| {
                    if t::button(ui, "刷新", t::Btn::Text(t::PRIMARY), 34.0).clicked() {
                        self.refresh_library();
                    }
                });
            });
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let index = match self.filter {
                Filter::All => 0,
                Filter::Scripts => 1,
                Filter::Plugins => 2,
            };
            let picked = t::segmented(ui, &["全部", "脚本", "插件"], index);
            self.filter = [Filter::All, Filter::Scripts, Filter::Plugins][picked];
            t::search_input(ui, &mut self.search, ui.available_width().max(100.0));
        });
        ui.add_space(12.0);
        self.update_rows();
        let card_h = (ui.available_height() - 56.0).max(120.0);
        egui::Frame::new()
            .fill(t::SURFACE)
            .stroke(Stroke::new(1.0_f32, t::OUTLINE_VARIANT))
            .corner_radius(t::RADIUS_MD)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.set_min_height(card_h);
                ui.set_max_height(card_h);
                ui.set_min_width(ui.available_width());
                if self.rows.is_empty() {
                    let searching = !self.search.trim().is_empty();
                    empty_state(
                        ui,
                        card_h,
                        searching,
                        if searching {
                            "无匹配结果"
                        } else if self.busy.is_some() {
                            "正在读取装备库…"
                        } else {
                            "装备库还是空的"
                        },
                        if searching {
                            "尝试其他名称、分组或作者"
                        } else {
                            "拖入脚本或压缩包，或点击「选择文件」"
                        },
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("library-list")
                        .auto_shrink([false, false])
                        .max_height(card_h)
                        .show_rows(ui, 56.0, self.rows.len(), |ui, visible| {
                            for index in visible {
                                let (name, kind, meta, path) = match self.rows[index] {
                                    Row::Script(i) => {
                                        let e = &self.library.scripts[i];
                                        (&e.name, "脚本", e.group.as_deref(), &e.path)
                                    }
                                    Row::Plugin(i) => {
                                        let e = &self.library.plugins[i];
                                        (&e.name, "插件", e.author.as_deref(), &e.path)
                                    }
                                };
                                ui.push_id((kind, path), |ui| {
                                    ui.add_enabled_ui(self.busy.is_none(), |ui| {
                                        if list_row(ui, name, Some(kind), meta, path) {
                                            self.confirm_uninstall = Some(path.clone());
                                        }
                                    });
                                });
                            }
                        });
                }
            });
        ui.add_space(8.0);
        let hint = self.busy.unwrap_or_else(|| {
            if !self.status_line.is_empty() {
                &self.status_line
            } else if self.cavalry_running {
                "新装脚本即时生效；插件需重启 Cavalry"
            } else {
                "启动 Cavalry 后加载新安装的内容"
            }
        });
        ui.add(
            egui::Label::new(
                RichText::new(format!("v{} · {hint}", env!("CARGO_PKG_VERSION")))
                    .font(t::font(t::BODY_SM))
                    .color(t::ON_SURFACE_VARIANT),
            )
            .wrap(),
        );
    }
}

/// MD3 list 行:56px;图标块 + 名称 + meta + 右侧「显示/卸载」文字钮;悬停中性罩
fn list_row(
    ui: &mut egui::Ui,
    name: &str,
    kind: Option<&str>,
    meta: Option<&str>,
    path: &std::path::Path,
) -> bool {
    let height = 56.0_f32;
    let full_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(full_w, height), Sense::hover());
    let resp = resp.on_hover_text(name);
    let k = ui.ctx().animate_bool_with_time(
        egui::Id::new(("ck-md3row", path)),
        resp.hovered(),
        t::ANIM_HOVER,
    );
    let painter = ui.painter_at(rect);
    if k > 0.0 {
        painter.rect_filled(
            rect.shrink2(Vec2::new(2.0, 1.0)),
            egui::CornerRadius::same(t::RADIUS_SM),
            t::with_alpha(t::STATE_HOVER_NEUTRAL, k),
        );
    }

    // 图标块
    t::icon_tile(
        &painter,
        Pos2::new(rect.min.x + 28.0, rect.center().y),
        if kind == Some("插件") { "PL" } else { "JS" },
    );

    // 名称 + meta
    let text_x = rect.min.x + 56.0;
    let actions_slot = 112.0_f32;
    let text_max = (rect.max.x - actions_slot - 20.0 - text_x).max(24.0);
    // One eliding layout instead of O(n²) per-character trial shaping.
    let name_g = elided(&painter, name, t::font(t::BODY_MD), t::ON_SURFACE, text_max);
    let meta_text = [kind, meta]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let name_h = name_g.size().y;
    let has_meta = !meta_text.is_empty();
    let block_h = if has_meta {
        name_h + 2.0 + 15.0
    } else {
        name_h
    };
    let y0 = rect.center().y - block_h / 2.0;
    painter.galley(Pos2::new(text_x, y0), name_g, t::ON_SURFACE);
    if has_meta {
        let mg = elided(
            &painter,
            &meta_text,
            t::font(t::BODY_SM),
            t::ON_SURFACE_VARIANT,
            text_max,
        );
        painter.galley(
            Pos2::new(text_x, y0 + name_h + 2.0),
            mg,
            t::ON_SURFACE_VARIANT,
        );
    }

    // 右侧文字钮:显示 / 卸载(卸载 error 红)
    let un_rect = egui::Rect::from_min_size(
        Pos2::new(rect.max.x - 8.0 - 56.0, rect.center().y - 16.0),
        Vec2::new(56.0, 32.0),
    );
    let sh_rect = egui::Rect::from_min_size(
        Pos2::new(un_rect.min.x - 4.0 - 56.0, rect.center().y - 16.0),
        Vec2::new(56.0, 32.0),
    );
    let un_resp = ui
        .interact(un_rect, egui::Id::new(("ck-un", path)), Sense::click())
        .on_hover_text("移入废纸篓");
    let sh_resp = ui
        .interact(sh_rect, egui::Id::new(("ck-sh", path)), Sense::click())
        .on_hover_text("在文件管理器中显示");
    sh_resp.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled(),
            format!("显示 {name}"),
        )
    });
    un_resp.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled(),
            format!("卸载 {name}"),
        )
    });
    let sh_press = sh_resp.hovered() && ui.ctx().input(|i| i.pointer.primary_down());
    let un_press = un_resp.hovered() && ui.ctx().input(|i| i.pointer.primary_down());
    paint_text_btn(
        &painter,
        sh_rect,
        "显示",
        t::PRIMARY,
        t::interact_t(ui, sh_resp.id, sh_resp.hovered()),
        sh_press,
    );
    paint_text_btn(
        &painter,
        un_rect,
        "卸载",
        t::ERROR,
        t::interact_t(ui, un_resp.id, un_resp.hovered()),
        un_press,
    );
    if un_resp.hovered() || sh_resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if sh_resp.clicked() {
        paths::reveal_in_file_manager(path);
    }
    un_resp.clicked()
}

/// 行内文字钮:悬停罩 + 文字颜色过渡(微交互)
fn paint_text_btn(
    painter: &egui::Painter,
    rect: egui::Rect,
    text: &str,
    color: egui::Color32,
    k: f32,
    pressed: bool,
) {
    // 无底色罩(不泛白):交互只走文字色 + 下划线;按下文字直接最深
    let _ = rect;
    let _ = pressed;
    // 悬停/按下文字加深(显示→更深紫,卸载→更深红;按下再深一档)
    let deep = if color == t::ERROR {
        egui::Color32::from_rgb(0x8a, 0x1d, 0x17)
    } else {
        t::ON_PRIMARY_CONTAINER
    };
    let c = if pressed {
        deep
    } else {
        t::lerp_color(color, deep, k)
    };
    let g = painter.layout_no_wrap(text.to_owned(), t::font_medium(t::LABEL_LG), c);
    let (gw, gh) = (g.size().x, g.size().y);
    painter.galley(rect.center() - g.size() / 2.0, g, c);
    // 悬停下划线扫入(从中点向两侧展开,过渡感)
    if k > 0.02 && !pressed {
        let uw = gw * k;
        let y = rect.center().y + gh / 2.0 + 2.0;
        painter.line_segment(
            [
                Pos2::new(rect.center().x - uw / 2.0, y),
                Pos2::new(rect.center().x + uw / 2.0, y),
            ],
            Stroke::new(2.0_f32, t::with_alpha(c, k)),
        );
    }
}

/// 居中空状态(搜索无结果 / 库为空):tonal 图标块 + 主文案 + 次文案,垂直居中于卡片
fn empty_state(ui: &mut egui::Ui, card_h: f32, search: bool, title: &str, sub: &str) {
    let w = ui.available_width();
    let h = (card_h - 16.0).max(140.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
    let painter = ui.painter_at(rect);
    let cx = rect.center().x;
    let total_h = 56.0 + 16.0 + 20.0 + 6.0 + 17.0;
    let top = rect.center().y - total_h / 2.0;
    // 图标:56px 圆角 tonal 块(primary-container 底 + 主紫字形)
    let tile = egui::Rect::from_center_size(Pos2::new(cx, top + 28.0), Vec2::splat(56.0));
    painter.rect_filled(
        tile,
        egui::CornerRadius::same(t::RADIUS_LG),
        t::PRIMARY_CONTAINER,
    );
    let ic = tile.center();
    if search {
        // 放大镜(与搜索框图标同语言)
        painter.circle_stroke(
            ic - Vec2::new(2.4, 2.4),
            8.5,
            Stroke::new(2.4_f32, t::PRIMARY),
        );
        painter.line_segment(
            [ic + Vec2::new(4.6, 4.6), ic + Vec2::new(10.5, 10.5)],
            Stroke::new(2.4_f32, t::PRIMARY),
        );
    } else {
        // 托盘 + 下落箭头(呼应左轨拖放卡)
        let tray = egui::Rect::from_center_size(ic + Vec2::new(0.0, 6.0), Vec2::new(24.0, 13.0));
        painter.rect_stroke(
            tray,
            egui::CornerRadius::same(4),
            Stroke::new(2.0_f32, t::PRIMARY),
            StrokeKind::Inside,
        );
        painter.line_segment(
            [ic + Vec2::new(0.0, -13.0), ic + Vec2::new(0.0, -1.0)],
            Stroke::new(2.2_f32, t::PRIMARY),
        );
        painter.line_segment(
            [ic + Vec2::new(-4.5, -5.5), ic + Vec2::new(0.0, -1.0)],
            Stroke::new(2.2_f32, t::PRIMARY),
        );
        painter.line_segment(
            [ic + Vec2::new(4.5, -5.5), ic + Vec2::new(0.0, -1.0)],
            Stroke::new(2.2_f32, t::PRIMARY),
        );
    }
    // 主文案 + 次文案,水平居中
    let g1 = painter.layout_no_wrap(title.to_owned(), t::font_medium(t::TITLE_SM), t::ON_SURFACE);
    let g1_h = g1.size().y;
    painter.galley(
        Pos2::new(cx - g1.size().x / 2.0, top + 56.0 + 16.0),
        g1,
        t::ON_SURFACE,
    );
    let g2 = painter.layout_no_wrap(sub.to_owned(), t::font(t::BODY_SM), t::ON_SURFACE_VARIANT);
    painter.galley(
        Pos2::new(cx - g2.size().x / 2.0, top + 56.0 + 16.0 + g1_h + 6.0),
        g2,
        t::ON_SURFACE_VARIANT,
    );
}

/// 计数徽章:淡紫底 pill + 主紫小字(标题旁)
fn count_badge(ui: &mut egui::Ui, count: usize) {
    use egui::Pos2;
    let g = ui.painter().layout_no_wrap(
        format!("{count} 件"),
        t::font_medium(t::LABEL_MD),
        t::ON_SURFACE_VARIANT,
    );
    let (pad_x, pad_y) = (8.0_f32, 3.0_f32);
    let size = Vec2::new(g.size().x + pad_x * 2.0, g.size().y + pad_y * 2.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        egui::CornerRadius::same((size.y / 2.0).round() as u8),
        egui::Color32::TRANSPARENT,
        Stroke::new(1.0_f32, t::OUTLINE),
        egui::StrokeKind::Inside,
    );
    painter.galley(
        Pos2::new(rect.min.x + pad_x, rect.min.y + pad_y),
        g,
        t::ON_SURFACE_VARIANT,
    );
}

fn elided(
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap.max_width = width;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    painter.layout_job(job)
}
