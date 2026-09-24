//! 主应用状态与界面框架(MD3:左栏动作轨 + 右区装备库 + 对话框 + snackbar)。

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{Margin, Vec2};

use super::theme as t;
use crate::installer::{self, ConflictChoice, InstallPlan, Outcome, PlannedUnit};
use crate::library::{self, Library};
use crate::paths::{self, CavalryPaths};

/// 装备库筛选
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Filter {
    All,
    Scripts,
    Plugins,
}

pub struct CavalryKitApp {
    pub(crate) paths: Option<CavalryPaths>,
    pub(crate) filter: Filter,
    /// 待安装的输入(拖入 / 文件对话框 / 命令行参数)
    pub(crate) queue: Vec<PathBuf>,
    pub(crate) pending_plan: Option<InstallPlan>,
    pub(crate) conflicts: Vec<PlannedUnit>,
    pub(crate) resolved: Vec<(PlannedUnit, ConflictChoice)>,
    pub(crate) outcome: Option<Outcome>,
    pub(crate) library: Library,
    pub(crate) confirm_uninstall: Option<PathBuf>,
    pub(crate) cavalry_running: bool,
    pub(crate) about_open: bool,
    /// snackbar(底部暗条),2.4s 自动消退
    pub(crate) toast: Option<(String, Instant)>,
    pub(crate) search: String,
    last_check: Instant,
    pub(crate) status_line: String,
}

impl CavalryKitApp {
    pub fn new(initial_files: Vec<PathBuf>) -> Self {
        let paths = CavalryPaths::detect();
        let mut status_line = String::new();
        if paths.is_none() {
            status_line = "未能定位 Cavalry 目录 — 请先安装 Cavalry".to_string();
        }
        let library = paths.as_ref().map(library::scan).unwrap_or_default();
        Self {
            paths,
            filter: Filter::All,
            queue: initial_files,
            pending_plan: None,
            conflicts: Vec::new(),
            resolved: Vec::new(),
            outcome: None,
            library,
            confirm_uninstall: None,
            cavalry_running: paths::is_cavalry_running(),
            about_open: false,
            toast: None,
            search: String::new(),
            last_check: Instant::now(),
            status_line,
        }
    }

    pub(crate) fn enqueue(&mut self, files: Vec<PathBuf>) {
        let files: Vec<PathBuf> = files.into_iter().filter(|p| p.exists()).collect();
        if files.is_empty() {
            return;
        }
        self.outcome = None;
        self.queue.extend(files);
    }

    pub(crate) fn finish(&mut self, outcome: Outcome) {
        self.outcome = Some(outcome);
        self.pending_plan = None;
        self.refresh_library();
    }

    pub(crate) fn refresh_library(&mut self) {
        if let Some(paths) = &self.paths {
            self.library = library::scan(paths);
        }
    }

    pub(crate) fn push_toast(&mut self, text: impl Into<String>) {
        self.toast = Some((text.into(), Instant::now()));
    }

    fn resolve_current(&mut self, choice: ConflictChoice) {
        if let Some(unit) = self.conflicts.first().cloned() {
            self.resolved.push((unit, choice));
            self.conflicts.remove(0);
        }
        if self.conflicts.is_empty() {
            if let Some(mut plan) = self.pending_plan.take() {
                let resolved = std::mem::take(&mut self.resolved);
                let mut outcome = installer::execute(plan.units, resolved);
                outcome.failures.extend(std::mem::take(&mut plan.failures));
                self.finish(outcome);
            }
        }
    }

    fn pump_install_queue(&mut self) {
        if self.pending_plan.is_some() || !self.conflicts.is_empty() {
            return;
        }
        if self.queue.is_empty() || self.paths.is_none() {
            return;
        }
        let paths = self.paths.clone().unwrap();
        let queue = std::mem::take(&mut self.queue);
        let mut plan = installer::plan(&queue, &paths);
        if plan.conflicts.is_empty() {
            let mut outcome = installer::execute(std::mem::take(&mut plan.units), vec![]);
            outcome.failures.extend(std::mem::take(&mut plan.failures));
            self.finish(outcome);
        } else {
            self.conflicts = std::mem::take(&mut plan.conflicts);
            self.pending_plan = Some(plan);
        }
    }
}

impl eframe::App for CavalryKitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. 拖放
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if !dropped.is_empty() {
            self.enqueue(dropped);
        }

        // 2. Cavalry 运行状态
        if self.last_check.elapsed() > Duration::from_secs(2) {
            self.last_check = Instant::now();
            self.cavalry_running = paths::is_cavalry_running();
        }

        // 3. 安装流水线
        self.pump_install_queue();

        // 4. 主界面:左栏动作轨(约 26%)+ 右区装备库(comp-b 构图)
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(t::SURFACE)
                    .inner_margin(Margin {
                        left: 16,
                        right: 16,
                        top: 14,
                        bottom: 16,
                    }),
            )
            .show(ctx, |ui| {
                let full = ui.max_rect();
                let rail_w = (full.width() * 0.26).clamp(220.0, 260.0);
                let gap = 16.0_f32;
                // 左栏
                let rail_rect = egui::Rect::from_min_size(full.min, Vec2::new(rail_w, full.height()));
                let mut rail_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rail_rect)
                        .layout(egui::Layout::top_down(egui::Align::LEFT)),
                );
                egui::Frame::new()
                    .fill(t::SURFACE_CONTAINER)
                    .corner_radius(egui::CornerRadius::same(t::RADIUS_MD))
                    .inner_margin(14.0)
                    .show(&mut rail_ui, |ui| {
                        ui.set_width(rail_w - 28.0);
                        self.show_rail(ui);
                    });
                // 右区
                let lib_rect = egui::Rect::from_min_max(
                    egui::Pos2::new(full.min.x + rail_w + gap, full.min.y),
                    full.max,
                );
                let mut lib_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(lib_rect)
                        .layout(egui::Layout::top_down(egui::Align::LEFT)),
                );
                self.show_library(&mut lib_ui);
            });

        // 5. Snackbar(底部暗条)
        if let Some((text, at)) = &self.toast {
            if at.elapsed() > Duration::from_millis(2400) {
                self.toast = None;
            } else {
                let t_in = ctx.animate_bool_with_time(egui::Id::new("ck-toast"), true, 0.2);
                let galley = ctx.fonts(|f| {
                    f.layout_no_wrap(text.clone(), t::font(t::BODY_MD), t::INVERSE_ON_SURFACE)
                });
                let size = egui::vec2(galley.size().x + 48.0, 48.0);
                egui::Area::new(egui::Id::new("ck-toast-area"))
                    .order(egui::Order::Middle)
                    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0 - (1.0 - t_in) * 8.0))
                    .show(ctx, |ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(size * t_in.max(0.85), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(
                            rect,
                            egui::CornerRadius::same(t::RADIUS_XS),
                            t::fade(t::INVERSE_SURFACE, t_in),
                        );
                        painter.galley(
                            rect.center() - galley.size() / 2.0,
                            galley,
                            t::fade(t::INVERSE_ON_SURFACE, t_in),
                        );
                    });
            }
        }

        // 6. 对话框
        if !self.conflicts.is_empty() {
            self.show_conflict_dialog(ctx);
        }
        if self.confirm_uninstall.is_some() {
            self.show_uninstall_dialog(ctx);
        }
        if self.about_open {
            self.show_about_dialog(ctx);
        }
    }
}

impl CavalryKitApp {
    /// 左栏动作轨:品牌行(字标 + 状态 chip 同行)+ 安装区 + 目录入口(固定间距)
    fn show_rail(&mut self, ui: &mut egui::Ui) {
        // 品牌行:紫块 >_ + 字标 + 紧凑状态 chip(同行右对齐;chip 缩小避免遮挡)
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 34.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let (mark, _) = ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
                let painter = ui.painter_at(mark);
                painter.rect_filled(mark, egui::CornerRadius::same(t::RADIUS_SM), t::PRIMARY);
                let g = ui
                    .painter()
                    .layout_no_wrap(">_".to_owned(), t::font_bold(13.0), t::ON_PRIMARY);
                ui.painter()
                    .galley(mark.center() - g.size() / 2.0 + Vec2::new(1.0, 0.0), g, t::ON_PRIMARY);
                ui.label(t::medium("CavalryKit", 16.0).color(t::ON_SURFACE));
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 24.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        t::status_chip_compact(
                            ui,
                            self.cavalry_running,
                            if self.cavalry_running { "运行中" } else { "未运行" },
                        );
                    },
                );
            },
        );
        ui.add_space(12.0);

        // 安装区(拖放卡或结果卡;占左栏余量)
        self.show_install_stage(ui);

        // 底端:目录入口与 CTA 固定间距(不随窗口高度漂移)
        ui.add_space(12.0);
        rail_link(ui, "脚本目录", Icon::Folder, || {
            if let Some(p) = self.paths.clone() {
                paths::open_dir(&p.scripts);
            }
        });
        ui.add_space(4.0);
        rail_link(ui, "插件目录", Icon::Puzzle, || {
            if let Some(p) = self.paths.clone() {
                paths::open_dir(&p.plugins);
            }
        });
        ui.add_space(4.0);
        let about = self.about_open;
        rail_link(ui, "关于", Icon::Info, || {
            if !about {
                self.about_open = true;
            }
        });
    }

    fn show_conflict_dialog(&mut self, ctx: &egui::Context) {
        let t_in = ctx.animate_bool_with_time(egui::Id::new("ck-dialog-conflict"), true, 0.2);
        t::dim_screen(ctx, "dlg-dim", 0.32 * t_in);

        let (kind_label, dest_text, remaining) = {
            let c = &self.conflicts[0];
            (
                c.kind.label(),
                c.dest.to_string_lossy().replace('\\', "/"),
                self.conflicts.len(),
            )
        };

        let mut choice: Option<ConflictChoice> = None;
        t::modal_area(ctx, "dlg-conflict", t_in, |ui| {
            t::dialog_frame()
                .fill(t::fade(t::SURFACE_CONTAINER_HIGH, t_in))
                .show(ui, |ui| {
                    ui.set_width(392.0);
                    ui.label(
                        t::medium("安装冲突", t::TITLE_LG).color(t::fade(t::ON_SURFACE, t_in)),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!("目标位置已存在同名{kind_label}:"))
                            .font(t::font(t::BODY_MD))
                            .color(t::fade(t::ON_SURFACE_VARIANT, t_in)),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(dest_text)
                            .font(t::font(t::BODY_SM))
                            .color(t::fade(t::ON_SURFACE_VARIANT, t_in)),
                    );
                    ui.add_space(20.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), 40.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if t::button(ui, "覆盖", t::Btn::Filled, 40.0).clicked() {
                                choice = Some(ConflictChoice::Overwrite);
                            }
                            ui.add_space(4.0);
                            if t::button(ui, "保留两者", t::Btn::Text(t::PRIMARY), 40.0).clicked() {
                                choice = Some(ConflictChoice::KeepBoth);
                            }
                            ui.add_space(4.0);
                            if t::button(ui, "跳过", t::Btn::Text(t::PRIMARY), 40.0).clicked() {
                                choice = Some(ConflictChoice::Skip);
                            }
                        },
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("还有 {remaining} 个冲突待处理"))
                            .font(t::font(t::LABEL_MD))
                            .color(t::fade(t::ON_SURFACE_VARIANT, t_in)),
                    );
                });
        });
        if let Some(c) = choice {
            self.resolve_current(c);
        }
    }

    fn show_uninstall_dialog(&mut self, ctx: &egui::Context) {
        let t_in = ctx.animate_bool_with_time(egui::Id::new("ck-dialog-uninstall"), true, 0.2);
        t::dim_screen(ctx, "dlg-dim2", 0.32 * t_in);

        let target = self.confirm_uninstall.clone().unwrap();
        let name = target
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| target.display().to_string());

        let mut decided: Option<bool> = None;
        t::modal_area(ctx, "dlg-uninstall", t_in, |ui| {
            t::dialog_frame()
                .fill(t::fade(t::SURFACE_CONTAINER_HIGH, t_in))
                .show(ui, |ui| {
                    ui.set_width(360.0);
                    ui.label(
                        t::medium("确认卸载", t::TITLE_LG).color(t::fade(t::ON_SURFACE, t_in)),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("将以下条目移入废纸篓(可从废纸篓恢复):")
                            .font(t::font(t::BODY_MD))
                            .color(t::fade(t::ON_SURFACE_VARIANT, t_in)),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(name.clone())
                            .font(t::font(t::BODY_SM))
                            .color(t::fade(t::ON_SURFACE_VARIANT, t_in)),
                    );
                    ui.add_space(20.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), 40.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if t::button(ui, "移入废纸篓", t::Btn::Filled, 40.0).clicked() {
                                decided = Some(true);
                            }
                            ui.add_space(4.0);
                            if t::button(ui, "取消", t::Btn::Text(t::PRIMARY), 40.0).clicked() {
                                decided = Some(false);
                            }
                        },
                    );
                });
        });

        match decided {
            Some(true) => {
                let result = self
                    .paths
                    .as_ref()
                    .ok_or_else(|| "Cavalry 目录不可用".to_string())
                    .and_then(|p| library::uninstall(p, &target));
                self.status_line = match result {
                    Ok(()) => format!("已卸载: {name}"),
                    Err(e) => format!("卸载失败: {e}"),
                };
                self.push_toast(self.status_line.clone());
                self.confirm_uninstall = None;
                self.refresh_library();
            }
            Some(false) => self.confirm_uninstall = None,
            None => {}
        }
    }

    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        let t_in = ctx.animate_bool_with_time(egui::Id::new("ck-dialog-about"), true, 0.2);
        t::dim_screen(ctx, "dlg-dim3", 0.32 * t_in);
        let mut close = false;
        t::modal_area(ctx, "dlg-about", t_in, |ui| {
            t::dialog_frame()
                .fill(t::fade(t::SURFACE_CONTAINER_HIGH, t_in))
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    ui.horizontal(|ui| {
                        let (mark, _) =
                            ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
                        ui.painter_at(mark).rect_filled(
                            mark,
                            egui::CornerRadius::same(t::RADIUS_SM),
                            t::PRIMARY,
                        );
                        let g = ui.painter().layout_no_wrap(
                            ">_".to_owned(),
                            t::font_bold(13.0),
                            t::ON_PRIMARY,
                        );
                        ui.painter().galley(
                            mark.center() - g.size() / 2.0 + Vec2::new(1.0, 0.0),
                            g,
                            t::ON_PRIMARY,
                        );
                        ui.add_space(4.0);
                        ui.label(
                            t::medium("CavalryKit", t::TITLE_LG).color(t::fade(t::ON_SURFACE, t_in)),
                        );
                    });
                    ui.add_space(8.0);
                    ui.label(
                        t::body("Cavalry 脚本与插件的快捷安装器。拖入即装,冲突逐个询问,随时还原。", t::BODY_MD)
                            .color(t::fade(t::ON_SURFACE_VARIANT, t_in)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        t::body("v0.1.0 · 仅供学习用途 · 与 Scene Group 无关", t::BODY_SM)
                            .color(t::fade(t::ON_SURFACE_VARIANT, t_in)),
                    );
                    ui.add_space(16.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), 40.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if t::button(ui, "知道了", t::Btn::Filled, 40.0).clicked() {
                                close = true;
                            }
                        },
                    );
                });
        });
        if close {
            self.about_open = false;
        }
    }
}

// ==================== 轨内链接 ====================

enum Icon {
    Folder,
    Puzzle,
    Info,
}

/// 轨底链接行:小图标 + label-medium 文字,悬停 8% 中性罩
fn rail_link(ui: &mut egui::Ui, text: &str, icon: Icon, on_click: impl FnOnce()) {
    let h = 26.0_f32;
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());
    let k = t::interact_t(ui, resp.id, resp.hovered());
    let painter = ui.painter_at(rect);
    if k > 0.0 {
        painter.rect_filled(rect, egui::CornerRadius::same(t::RADIUS_SM), t::with_alpha(t::STATE_HOVER_NEUTRAL, k));
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let c = egui::Pos2::new(rect.min.x + 12.0, rect.center().y);
    let col = t::ON_SURFACE_VARIANT;
    match icon {
        Icon::Folder => {
            // 文件夹图标(体 + 左上 tab)
            let body = egui::Rect::from_min_size(
                egui::Pos2::new(c.x - 7.0, c.y - 4.0),
                Vec2::new(14.0, 9.0),
            );
            painter.rect(
                body,
                egui::CornerRadius::same(2),
                egui::Color32::TRANSPARENT,
                egui::Stroke::new(1.5_f32, col),
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [egui::Pos2::new(c.x - 7.0, c.y - 4.0), egui::Pos2::new(c.x - 2.0, c.y - 4.0)],
                egui::Stroke::new(2.0_f32, col),
            );
            painter.line_segment(
                [egui::Pos2::new(c.x - 2.0, c.y - 4.0), egui::Pos2::new(c.x - 0.5, c.y - 2.5)],
                egui::Stroke::new(1.5_f32, col),
            );
        }
        Icon::Puzzle => {
            painter.circle_stroke(c, 6.0, egui::Stroke::new(1.5_f32, col));
            painter.circle_filled(c + Vec2::new(3.0, -3.0), 2.0, col);
        }
        Icon::Info => {
            painter.circle_stroke(c, 6.0, egui::Stroke::new(1.5_f32, col));
            let g = ui.painter().layout_no_wrap("i".to_owned(), t::font_bold(9.0), col);
            ui.painter().galley(c - g.size() / 2.0, g, col);
        }
    }
    let g = ui
        .painter()
        .layout_no_wrap(text.to_owned(), t::font_medium(t::LABEL_MD), col);
    ui.painter().galley(
        egui::Pos2::new(rect.min.x + 26.0, rect.center().y - g.size().y / 2.0),
        g,
        col,
    );
    if resp.clicked() {
        on_click();
    }
}
