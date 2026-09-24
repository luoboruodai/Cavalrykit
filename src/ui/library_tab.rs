//! 装备库(右区):标题 + 搜索 + segmented 筛选 + 白卡行列表(MD3 list)。

use egui::{CursorIcon, Pos2, RichText, Sense, Stroke, StrokeKind, Vec2};

use super::app::{CavalryKitApp, Filter};
use super::theme as t;
use crate::paths;

impl CavalryKitApp {
    pub(crate) fn show_library(&mut self, ui: &mut egui::Ui) {
        // 标题行:「装备库」大字 + 计数徽章(淡紫底 pill,与标题同中线)
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 36.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(t::medium("Cavalry 装备库", 24.0).color(t::ON_SURFACE));
                ui.add_space(8.0);
                let count = self.library.scripts.len() + self.library.plugins.len();
                count_badge(ui, count);
            },
        );
        ui.add_space(12.0);

        // 工具行:筛选药丸(左) | 搜索框(右)——搜索属于过滤系统,同行同线
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 40.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let labels = ["全部", "脚本", "插件"];
                let idx = match self.filter {
                    Filter::All => 0,
                    Filter::Scripts => 1,
                    Filter::Plugins => 2,
                };
                let picked = t::segmented(ui, &labels, idx);
                self.filter = [Filter::All, Filter::Scripts, Filter::Plugins][picked];
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 40.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        t::search_input(ui, &mut self.search, 240.0);
                    },
                );
            },
        );
        ui.add_space(16.0);

        // 过滤
        let q = self.search.trim().to_lowercase();
        let want_scripts = self.filter != Filter::Plugins;
        let want_plugins = self.filter != Filter::Scripts;
        let matched_scripts: Vec<(String, Option<String>, std::path::PathBuf)> = if want_scripts {
            self.library
                .scripts
                .iter()
                .filter(|e| {
                    q.is_empty()
                        || e.name.to_lowercase().contains(&q)
                        || e.group.as_deref().unwrap_or("").to_lowercase().contains(&q)
                })
                .map(|e| (e.name.clone(), e.group.clone(), e.path.clone()))
                .collect()
        } else {
            Vec::new()
        };
        let matched_plugins: Vec<(String, Option<String>, std::path::PathBuf)> = if want_plugins {
            self.library
                .plugins
                .iter()
                .filter(|e| {
                    q.is_empty()
                        || e.name.to_lowercase().contains(&q)
                        || e.author.as_deref().unwrap_or("").to_lowercase().contains(&q)
                })
                .map(|e| (e.name.clone(), e.author.clone(), e.path.clone()))
                .collect()
        } else {
            Vec::new()
        };

        // 白卡列表(level-1 柔影);确定性分配:卡片吃掉「可用 - 页脚预留」,窗口拉高不重叠
        let footer_reserve = 30.0_f32;
        let card_h = (ui.available_height() - footer_reserve - 8.0).max(160.0);
        egui::Frame::new()
            .fill(t::SURFACE)
            .stroke(Stroke::new(1.0_f32, t::OUTLINE_VARIANT))
            .corner_radius(egui::CornerRadius::same(t::RADIUS_MD))
            .inner_margin(egui::Margin::symmetric(8, 6))
            .shadow(t::elevation_1())
            .show(ui, |ui| {
                ui.set_min_height(card_h);
                ui.set_max_height(card_h);
                egui::ScrollArea::vertical()
                    .id_salt("ck-md3-list")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let searching = !q.is_empty();
                        let any_shown = (want_scripts && !matched_scripts.is_empty())
                            || (want_plugins && !matched_plugins.is_empty());
                        // 全空 → 居中空状态(搜索无结果 / 库为空),不再堆裸文本
                        if !any_shown {
                            if searching {
                                empty_state(
                                    ui,
                                    card_h,
                                    true,
                                    "无匹配结果",
                                    &format!("没有找到与「{q}」相关的脚本或插件"),
                                );
                            } else {
                                empty_state(
                                    ui,
                                    card_h,
                                    false,
                                    "装备库还是空的",
                                    "把 .jsc / .zip 拖进来,或点击「选择文件」开始安装",
                                );
                            }
                            return;
                        }
                        if want_scripts {
                            if matched_scripts.is_empty() {
                                // 搜索中某组无匹配:静默略过;未搜索才提示未安装
                                if !searching {
                                    empty_hint(ui, true, "尚未安装任何脚本", self.paths.as_ref().map(|p| p.scripts.clone()));
                                }
                            } else {
                                let total = matched_scripts.len();
                                for (i, (name, group, path)) in matched_scripts.iter().enumerate() {
                                    if list_row(ui, name, Some("脚本"), group.as_deref(), path) {
                                        self.confirm_uninstall = Some(path.clone());
                                    }
                                    if i + 1 < total {
                                        row_divider(ui);
                                    }
                                }
                            }
                        }
                        if want_plugins {
                            if want_scripts && !matched_scripts.is_empty() && !matched_plugins.is_empty() {
                                group_separator(ui);
                            }
                            if matched_plugins.is_empty() {
                                if !searching {
                                    empty_hint(ui, true, "尚未安装任何插件", self.paths.as_ref().map(|p| p.plugins.clone()));
                                }
                            } else {
                                let total = matched_plugins.len();
                                for (i, (name, author, path)) in matched_plugins.iter().enumerate() {
                                    if list_row(ui, name, Some("插件"), author.as_deref(), path) {
                                        self.confirm_uninstall = Some(path.clone());
                                    }
                                    if i + 1 < total {
                                        row_divider(ui);
                                    }
                                }
                            }
                        }
                    });
            });

        // 底部版本/提示行(卡片之下,离窗底保留外边距)
        ui.add_space(8.0);
        let hint = if !self.status_line.is_empty() {
            self.status_line.clone()
        } else if self.cavalry_running {
            "新装脚本即时生效;插件需重启 Cavalry".to_string()
        } else {
            "启动 Cavalry 后加载新安装的内容".to_string()
        };
        ui.label(
            RichText::new(format!("v0.1.0 · {hint}"))
                .font(t::font(t::BODY_MD))
                .color(egui::Color32::from_rgb(0xb5, 0xb6, 0xb9)),
        );
    }
}

fn row_divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    let line = egui::Rect::from_min_max(
        Pos2::new(rect.min.x + 56.0, rect.min.y),
        Pos2::new(rect.max.x - 12.0, rect.max.y),
    );
    ui.painter().line_segment(
        [line.min, line.max],
        Stroke::new(1.0_f32, t::SURFACE_VARIANT),
    );
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
    t::icon_tile(&painter, Pos2::new(rect.min.x + 28.0, rect.center().y), if kind == Some("插件") { "PL" } else { "JS" });

    // 名称 + meta
    let text_x = rect.min.x + 56.0;
    let actions_slot = 112.0_f32;
    let text_max = (rect.max.x - actions_slot - text_x).max(120.0);
    let name_font = t::font(t::BODY_MD);
    let mut shown = name.to_owned();
    if painter
        .layout_no_wrap(shown.clone(), name_font.clone(), t::ON_SURFACE)
        .size()
        .x
        > text_max
    {
        while shown.chars().count() > 4 {
            shown.pop();
            let cand = format!("{shown}…");
            if painter
                .layout_no_wrap(cand.clone(), name_font.clone(), t::ON_SURFACE)
                .size()
                .x
                <= text_max
            {
                shown = cand;
                break;
            }
        }
    }
    let name_g = painter.layout_no_wrap(shown, name_font, t::ON_SURFACE);
    let meta_text = [kind, meta]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let name_h = name_g.size().y;
    let has_meta = !meta_text.is_empty();
    let block_h = if has_meta { name_h + 2.0 + 15.0 } else { name_h };
    let y0 = rect.center().y - block_h / 2.0;
    painter.galley(Pos2::new(text_x, y0), name_g, t::ON_SURFACE);
    if has_meta {
        let mg = painter.layout_no_wrap(meta_text, t::font(t::BODY_SM), t::ON_SURFACE_VARIANT);
        painter.galley(Pos2::new(text_x, y0 + name_h + 2.0), mg, t::ON_SURFACE_VARIANT);
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
        .on_hover_text("在访达中显示");
    let sh_press = sh_resp.hovered() && ui.ctx().input(|i| i.pointer.primary_down());
    let un_press = un_resp.hovered() && ui.ctx().input(|i| i.pointer.primary_down());
    paint_text_btn(&painter, sh_rect, "显示", t::PRIMARY, t::interact_t(ui, sh_resp.id, sh_resp.hovered()), sh_press);
    paint_text_btn(&painter, un_rect, "卸载", t::ERROR, t::interact_t(ui, un_resp.id, un_resp.hovered()), un_press);
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
    painter.galley(Pos2::new(cx - g1.size().x / 2.0, top + 56.0 + 16.0), g1, t::ON_SURFACE);
    let g2 = painter.layout_no_wrap(sub.to_owned(), t::font(t::BODY_SM), t::ON_SURFACE_VARIANT);
    painter.galley(
        Pos2::new(cx - g2.size().x / 2.0, top + 56.0 + 16.0 + g1_h + 6.0),
        g2,
        t::ON_SURFACE_VARIANT,
    );
}

/// 空状态
fn empty_hint(
    ui: &mut egui::Ui,
    is_empty_library: bool,
    label: &str,
    dir: Option<std::path::PathBuf>,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(label).font(t::font(t::BODY_MD)).color(t::ON_SURFACE_VARIANT),
        );
        if is_empty_library {
            if let Some(d) = dir {
                if t::button(ui, "打开目录", t::Btn::Text(t::PRIMARY), 30.0).clicked() {
                    paths::open_dir(&d);
                }
            }
        }
    });
    let _ = StrokeKind::Inside;
}

/// 计数徽章:淡紫底 pill + 主紫小字(标题旁)
fn count_badge(ui: &mut egui::Ui, count: usize) {
    use egui::Pos2;
    let g = ui
        .painter()
        .layout_no_wrap(format!("{count} 件"), t::font_medium(t::LABEL_MD), t::ON_SURFACE_VARIANT);
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

/// 组间分隔线(脚本 → 插件):左右缩进 1px outline-variant,与行线同位(无额外间距),比行线深一级但不突兀
fn group_separator(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::Vec2::new(ui.available_width(), 1.0),
        egui::Sense::hover(),
    );
    // 组间分隔:左右各缩进,明显但不横贯;颜色用 outline-variant(与卡片描边同调,不刺眼)
    let line = egui::Rect::from_min_max(
        Pos2::new(rect.min.x + 12.0, rect.min.y),
        Pos2::new(rect.max.x - 12.0, rect.max.y),
    );
    ui.painter().line_segment(
        [line.min, line.max],
        Stroke::new(1.0_f32, t::OUTLINE_VARIANT),
    );
}
