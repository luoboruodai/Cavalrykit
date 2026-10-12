//! UI state: bounded layout + event-driven background I/O + true modal dialogs.
use super::theme as t;
use crate::installer::{ConflictChoice, InstallPlan, Outcome, PlannedUnit};
use crate::library::Library;
use crate::paths::{self, CavalryPaths};
use crate::worker::{Event, Job, Worker};
use egui::{Margin, Vec2};
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Filter {
    All,
    Scripts,
    Plugins,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Row {
    Script(usize),
    Plugin(usize),
}

pub struct CavalryKitApp {
    pub(crate) paths: Option<CavalryPaths>,
    pub(crate) filter: Filter,
    pub(crate) queue: Vec<PathBuf>,
    pending_plan: Option<InstallPlan>,
    pub(crate) conflicts: VecDeque<PlannedUnit>,
    resolved: Vec<(PlannedUnit, ConflictChoice)>,
    pub(crate) outcome: Option<Outcome>,
    pub(crate) library: Library,
    pub(crate) confirm_uninstall: Option<PathBuf>,
    pub(crate) cavalry_running: bool,
    about_open: bool,
    toast: Option<(String, Instant)>,
    pub(crate) search: String,
    pub(crate) status_line: String,
    pub(crate) busy: Option<&'static str>,
    worker: Option<Worker>,
    picker: Option<Receiver<Vec<PathBuf>>>,
    pub(crate) rows: Vec<Row>,
    cached_query: Option<(String, Filter)>,
    logo: egui::TextureHandle,
}

impl CavalryKitApp {
    pub fn new(ctx: &egui::Context, initial_files: Vec<PathBuf>) -> Self {
        Self::with_paths(ctx, initial_files, CavalryPaths::detect())
    }

    /// Dependency injection for headless/layout tests; production always uses detect().
    pub fn with_paths(
        ctx: &egui::Context,
        initial_files: Vec<PathBuf>,
        paths: Option<CavalryPaths>,
    ) -> Self {
        let worker = paths.clone().map(|p| {
            let ctx = ctx.clone();
            Worker::spawn(p, move || ctx.request_repaint())
        });
        let icon = super::icon::window_icon();
        let logo = ctx.load_texture(
            "cavalrykit-logo",
            egui::ColorImage::from_rgba_unmultiplied(
                [icon.width as usize, icon.height as usize],
                &icon.rgba,
            ),
            egui::TextureOptions::LINEAR,
        );
        let mut app = Self {
            paths,
            filter: Filter::All,
            queue: Vec::new(),
            pending_plan: None,
            conflicts: VecDeque::new(),
            resolved: Vec::new(),
            outcome: None,
            library: Library::default(),
            confirm_uninstall: None,
            cavalry_running: false,
            about_open: false,
            toast: None,
            search: String::new(),
            status_line: String::new(),
            busy: None,
            worker,
            picker: None,
            rows: Vec::new(),
            cached_query: None,
            logo,
        };
        if app.paths.is_some() {
            app.submit(Job::Scan, "正在读取装备库…");
        } else {
            app.status_line = "无法定位用户配置目录".into();
        }
        app.enqueue(initial_files);
        app
    }

    pub(crate) fn enqueue(&mut self, files: Vec<PathBuf>) {
        // No metadata calls on the UI thread: network paths can block even exists().
        let mut seen: HashSet<PathBuf> = self.queue.iter().cloned().collect();
        for file in files {
            if self.queue.len() >= 256 {
                self.push_toast("单次队列最多 256 个输入，请分批导入。");
                break;
            }
            if seen.insert(file.clone()) {
                self.queue.push(file);
            }
        }
    }

    fn submit(&mut self, job: Job, status: &'static str) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "后台任务不可用".into())
            .and_then(|w| w.submit(job));
        match result {
            Ok(()) => self.busy = Some(status),
            Err(error) => {
                self.busy = None;
                self.report_error(error);
            }
        }
    }

    fn report_error(&mut self, error: String) {
        self.outcome = Some(Outcome {
            failures: vec![("操作未完成".into(), error)],
            ..Default::default()
        });
    }

    pub(crate) fn refresh_library(&mut self) {
        if self.busy.is_none() && self.pending_plan.is_none() {
            self.submit(Job::Scan, "正在刷新装备库…");
        }
    }

    fn replace_library(&mut self, library: Library) {
        self.library = library;
        self.cached_query = None;
    }

    pub(crate) fn update_rows(&mut self) {
        let key = (self.search.trim().to_lowercase(), self.filter);
        if self.cached_query.as_ref() == Some(&key) {
            return;
        }
        self.rows.clear();
        if self.filter != Filter::Plugins {
            self.rows.extend(
                self.library
                    .scripts
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.search_key.contains(&key.0))
                    .map(|(i, _)| Row::Script(i)),
            );
        }
        if self.filter != Filter::Scripts {
            self.rows.extend(
                self.library
                    .plugins
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.search_key.contains(&key.0))
                    .map(|(i, _)| Row::Plugin(i)),
            );
        }
        self.cached_query = Some(key);
    }

    pub(crate) fn push_toast(&mut self, text: impl Into<String>) {
        self.toast = Some((text.into(), Instant::now()));
    }

    pub(crate) fn pick_files(&mut self, ctx: &egui::Context) {
        if self.picker.is_some() {
            return;
        }
        let future = rfd::AsyncFileDialog::new()
            .add_filter(
                "Cavalry 文件",
                &[
                    "jsc", "js", "jsx", "zip", "7z", "tar", "tgz", "tbz2", "gz", "bz2",
                ],
            )
            .add_filter("所有文件", &["*"])
            .pick_files();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let files = pollster::block_on(future)
                .unwrap_or_default()
                .into_iter()
                .map(|f| f.path().to_owned())
                .collect();
            let _ = tx.send(files);
            ctx.request_repaint();
        });
        self.picker = Some(rx);
    }

    fn poll_events(&mut self) {
        if let Some(picker) = &self.picker {
            match picker.try_recv() {
                Ok(files) => {
                    self.picker = None;
                    self.enqueue(files);
                }
                Err(TryRecvError::Disconnected) => {
                    self.picker = None;
                    self.report_error("文件选择窗口异常关闭".into());
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        while let Some(worker) = &self.worker {
            match worker.try_recv() {
                Ok(Event::Running(value)) => self.cavalry_running = value,
                Ok(Event::Library(lib)) => {
                    self.replace_library(lib);
                    self.busy = None;
                }
                Ok(Event::Planned(mut plan)) => {
                    self.busy = None;
                    self.conflicts = std::mem::take(&mut plan.conflicts).into();
                    if self.conflicts.is_empty() {
                        self.submit(Job::Install(plan, vec![]), "正在安装…");
                    } else {
                        self.pending_plan = Some(plan);
                    }
                }
                Ok(Event::Installed(outcome, lib)) => {
                    self.busy = None;
                    self.replace_library(lib);
                    self.outcome = Some(outcome);
                }
                Ok(Event::Uninstalled(result, lib)) => {
                    self.busy = None;
                    self.replace_library(lib);
                    match result {
                        Ok(()) => self.push_toast("已移入废纸篓 / 回收站，可在系统中恢复"),
                        Err(error) => self.report_error(error),
                    }
                }
                Ok(Event::Failed(error)) => {
                    self.busy = None;
                    self.report_error(error);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.worker = None;
                    self.busy = None;
                    self.report_error("后台线程已停止，请关闭后重新打开 CavalryKit。".into());
                    break;
                }
            }
        }
    }

    fn pump_queue(&mut self) {
        if self.busy.is_none()
            && self.pending_plan.is_none()
            && self.outcome.is_none()
            && self.confirm_uninstall.is_none()
            && !self.about_open
            && !self.queue.is_empty()
            && self.paths.is_some()
        {
            let files = std::mem::take(&mut self.queue);
            self.submit(Job::Plan(files), "正在识别与解压…");
        }
    }

    fn resolve_current(&mut self, choice: ConflictChoice) {
        if let Some(unit) = self.conflicts.pop_front() {
            self.resolved.push((unit, choice));
        }
        if self.conflicts.is_empty() {
            if let Some(plan) = self.pending_plan.take() {
                let resolved = std::mem::take(&mut self.resolved);
                self.submit(Job::Install(plan, resolved), "正在安装…");
            }
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        self.poll_events();
        let dropped = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        self.enqueue(dropped);
        self.pump_queue();
        if ctx.input(|i| i.viewport().close_requested())
            && (self.busy.is_some() || self.picker.is_some())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.push_toast("操作尚未完成，请稍后关闭窗口。");
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t::SURFACE).inner_margin(Margin {
                left: 16,
                right: 16,
                top: 14,
                bottom: 16,
            }))
            .show(ctx, |ui| {
                let full = ui.max_rect();
                let rail_w = 230.0;
                let rail_rect =
                    egui::Rect::from_min_size(full.min, Vec2::new(rail_w, full.height()));
                let mut rail =
                    ui.new_child(egui::UiBuilder::new().id_salt("rail").max_rect(rail_rect));
                rail.set_clip_rect(rail_rect.intersect(ui.clip_rect()));
                egui::Frame::new()
                    .fill(t::SURFACE_CONTAINER)
                    .corner_radius(t::RADIUS_MD)
                    .inner_margin(14.0)
                    .show(&mut rail, |ui| {
                        ui.set_width(rail_w - 28.0);
                        egui::ScrollArea::vertical()
                            .id_salt("rail-scroll")
                            .max_height(full.height() - 28.0)
                            .show(ui, |ui| self.show_rail(ui));
                    });
                let lib_rect = egui::Rect::from_min_max(
                    egui::pos2(full.min.x + rail_w + 16.0, full.min.y),
                    full.max,
                );
                let mut lib =
                    ui.new_child(egui::UiBuilder::new().id_salt("library").max_rect(lib_rect));
                lib.set_clip_rect(lib_rect.intersect(ui.clip_rect()));
                self.show_library(&mut lib);
            });
        if let Some((text, at)) = &self.toast {
            let lifetime = Duration::from_millis(2600);
            if at.elapsed() >= lifetime {
                self.toast = None;
            } else {
                ctx.request_repaint_after(lifetime.saturating_sub(at.elapsed()));
                egui::Area::new(egui::Id::new("toast"))
                    .order(egui::Order::Foreground)
                    .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -18.0])
                    .show(ctx, |ui| {
                        egui::Frame::new()
                            .fill(t::INVERSE_SURFACE)
                            .corner_radius(8)
                            .inner_margin(16.0)
                            .show(ui, |ui| {
                                ui.set_max_width(500.0);
                                ui.label(t::body(text, t::BODY_MD).color(t::INVERSE_ON_SURFACE));
                            });
                    });
            }
        }
        // Only one modal at a time; all underlying controls are blocked by egui::Modal.
        if !self.conflicts.is_empty() {
            self.show_conflict_dialog(ctx);
        } else if self.outcome.is_some() {
            self.show_result_dialog(ctx);
        } else if self.confirm_uninstall.is_some() {
            self.show_uninstall_dialog(ctx);
        } else if self.about_open {
            self.show_about_dialog(ctx);
        }
    }

    fn show_rail(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.image((self.logo.id(), Vec2::splat(34.0)));
            ui.label(t::medium("CavalryKit", 18.0).color(t::ON_SURFACE));
        });
        ui.add_space(4.0);
        t::status_chip_compact(
            ui,
            self.cavalry_running,
            if self.cavalry_running {
                "Cavalry 运行中"
            } else {
                "Cavalry 未运行"
            },
        );
        ui.add_space(12.0);
        self.show_install_stage(ui);
        ui.add_space(12.0);
        rail_link(ui, "脚本目录", Icon::Folder, || {
            if let Some(p) = &self.paths {
                paths::open_dir(&p.scripts);
            }
        });
        rail_link(ui, "插件目录", Icon::Puzzle, || {
            if let Some(p) = &self.paths {
                paths::open_dir(&p.plugins);
            }
        });
        rail_link(ui, "关于", Icon::Info, || self.about_open = true);
    }

    fn show_conflict_dialog(&mut self, ctx: &egui::Context) {
        let unit = self.conflicts.front().unwrap();
        let mut choice = None;
        modal(ctx, "conflict", |ui| {
            ui.label(t::medium("安装冲突", t::TITLE_LG));
            ui.label(format!("已存在同名{}：", unit.kind.label()));
            ui.add(egui::Label::new(unit.dest.to_string_lossy()).wrap());
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                for (label, value, style) in [
                    ("覆盖", ConflictChoice::Overwrite, t::Btn::Filled),
                    (
                        "保留两者",
                        ConflictChoice::KeepBoth,
                        t::Btn::Text(t::PRIMARY),
                    ),
                    ("跳过", ConflictChoice::Skip, t::Btn::Text(t::PRIMARY)),
                ] {
                    if t::button(ui, label, style, 40.0).clicked() {
                        choice = Some(value);
                    }
                }
            });
            ui.label(t::body(
                format!("还有 {} 个冲突待处理", self.conflicts.len()),
                t::BODY_SM,
            ));
        });
        if let Some(choice) = choice {
            self.resolve_current(choice);
        }
    }

    fn show_result_dialog(&mut self, ctx: &egui::Context) {
        let outcome = self.outcome.as_ref().unwrap();
        let mut close = false;
        let response = modal(ctx, "result", |ui| {
            ui.label(t::medium(
                if outcome.failures.is_empty() {
                    "安装完成"
                } else {
                    "安装结果 · 请检查"
                },
                t::TITLE_LG,
            ));
            ui.label(format!(
                "成功 {} · 跳过 {} · 失败 {}",
                outcome.installed_scripts.len() + outcome.installed_plugins.len(),
                outcome.skipped.len(),
                outcome.failures.len()
            ));
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .id_salt("result-rows")
                .max_height((ctx.screen_rect().height() - 260.0).clamp(100.0, 340.0))
                .show(ui, |ui| {
                    for name in outcome
                        .installed_scripts
                        .iter()
                        .chain(&outcome.installed_plugins)
                    {
                        ui.add(
                            egui::Label::new(
                                t::body(format!("已安装：{name}"), t::BODY_MD)
                                    .color(t::ON_SUCCESS_CONTAINER),
                            )
                            .wrap(),
                        );
                    }
                    for name in &outcome.skipped {
                        ui.add(egui::Label::new(format!("已跳过：{name}")).wrap());
                    }
                    for (name, reason) in &outcome.failures {
                        ui.add(
                            egui::Label::new(
                                t::body(format!("{name}\n{reason}"), t::BODY_MD).color(t::ERROR),
                            )
                            .wrap(),
                        );
                    }
                });
            if outcome.restart_hint {
                ui.label(
                    t::body("插件已安装，重启 Cavalry 后生效。", t::BODY_MD).color(t::PRIMARY),
                );
            }
            if !self.queue.is_empty() {
                ui.label(format!(
                    "确认后继续处理 {} 个待安装输入。",
                    self.queue.len()
                ));
            }
            ui.add_space(10.0);
            if t::button(ui, "知道了", t::Btn::Filled, 40.0).clicked() {
                close = true;
            }
        });
        if close || response.should_close() {
            self.outcome = None;
            ctx.request_repaint();
        }
    }

    fn show_uninstall_dialog(&mut self, ctx: &egui::Context) {
        let target = self.confirm_uninstall.as_ref().unwrap();
        let mut decided = None;
        let response = modal(ctx, "uninstall", |ui| {
            ui.label(t::medium("确认卸载", t::TITLE_LG));
            ui.label("将以下条目移入废纸篓 / 回收站（可恢复）：");
            ui.add(
                egui::Label::new(target.file_name().unwrap_or_default().to_string_lossy()).wrap(),
            );
            ui.horizontal(|ui| {
                if t::button(ui, "移入废纸篓", t::Btn::Filled, 40.0).clicked() {
                    decided = Some(true);
                }
                if t::button(ui, "取消", t::Btn::Text(t::PRIMARY), 40.0).clicked() {
                    decided = Some(false);
                }
            });
        });
        if decided == Some(true) {
            let target = self.confirm_uninstall.take().unwrap();
            self.submit(Job::Uninstall(target), "正在卸载…");
        } else if decided == Some(false) || response.should_close() {
            self.confirm_uninstall = None;
        }
    }

    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        let mut close = false;
        let response = modal(ctx, "about", |ui| {
            ui.horizontal(|ui| {
                ui.image((self.logo.id(), Vec2::splat(40.0)));
                ui.label(t::medium("CavalryKit", t::TITLE_LG));
            });
            ui.label("Cavalry 脚本与插件的快捷安装器。拖入即装，冲突逐个询问，卸载可恢复。");
            ui.label(format!(
                "v{} · 仅供学习用途 · 与 Scene Group 无关",
                env!("CARGO_PKG_VERSION")
            ));
            if t::button(ui, "知道了", t::Btn::Filled, 40.0).clicked() {
                close = true;
            }
        });
        if close || response.should_close() {
            self.about_open = false;
        }
    }
}

impl eframe::App for CavalryKitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }
}

fn modal<R>(
    ctx: &egui::Context,
    id: &str,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::ModalResponse<R> {
    egui::Modal::new(egui::Id::new(id))
        .frame(t::dialog_frame())
        .show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 100.0).clamp(260.0, 480.0));
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            contents(ui)
        })
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
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), text));
    let k = t::interact_t(ui, resp.id, resp.hovered());
    let painter = ui.painter_at(rect);
    if k > 0.0 {
        painter.rect_filled(
            rect,
            egui::CornerRadius::same(t::RADIUS_SM),
            t::with_alpha(t::STATE_HOVER_NEUTRAL, k),
        );
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
                [
                    egui::Pos2::new(c.x - 7.0, c.y - 4.0),
                    egui::Pos2::new(c.x - 2.0, c.y - 4.0),
                ],
                egui::Stroke::new(2.0_f32, col),
            );
            painter.line_segment(
                [
                    egui::Pos2::new(c.x - 2.0, c.y - 4.0),
                    egui::Pos2::new(c.x - 0.5, c.y - 2.5),
                ],
                egui::Stroke::new(1.5_f32, col),
            );
        }
        Icon::Puzzle => {
            painter.circle_stroke(c, 6.0, egui::Stroke::new(1.5_f32, col));
            painter.circle_filled(c + Vec2::new(3.0, -3.0), 2.0, col);
        }
        Icon::Info => {
            painter.circle_stroke(c, 6.0, egui::Stroke::new(1.5_f32, col));
            let g = ui
                .painter()
                .layout_no_wrap("i".to_owned(), t::font_bold(9.0), col);
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

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (egui::Context, CavalryKitApp) {
        let ctx = egui::Context::default();
        super::super::fonts::setup(&ctx);
        t::apply(&ctx);
        let app = CavalryKitApp::with_paths(&ctx, vec![], None);
        (ctx, app)
    }
    fn render(
        ctx: &egui::Context,
        app: &mut CavalryKitApp,
        size: [f32; 2],
        time: f64,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size.into())),
                time: Some(time),
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        )
    }
    #[test]
    fn large_library_renders_only_visible_rows_and_caches_filter() {
        let (ctx, mut app) = fixture();
        for i in 0..10_000 {
            let name = format!("Very long script name {i:05} {}.js", "中文".repeat(60));
            app.library.scripts.push(crate::library::ScriptEntry {
                search_key: name.to_lowercase(),
                name,
                group: None,
                path: format!("synthetic/{i}.js").into(),
            });
        }
        let _ = render(&ctx, &mut app, [960.0, 680.0], 0.0);
        let output = render(&ctx, &mut app, [960.0, 680.0], 2.0);
        assert_eq!(app.rows.len(), 10_000);
        assert!(
            output.shapes.len() < 700,
            "rows were not virtualized: {} shapes",
            output.shapes.len()
        );
        let mut samples = Vec::new();
        for frame in 0..300 {
            let start = Instant::now();
            let out = render(&ctx, &mut app, [960.0, 680.0], 3.0 + frame as f64 / 60.0);
            let _ = ctx.tessellate(out.shapes, out.pixels_per_point);
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "PERF_10K_UI {{\"p50_ms\":{:.3},\"p95_ms\":{:.3},\"samples\":300}}",
            samples[150], samples[285]
        );
        let allocation = app.rows.as_ptr();
        app.update_rows();
        assert_eq!(allocation, app.rows.as_ptr());
        app.search = "09999".into();
        app.update_rows();
        assert_eq!(app.rows, vec![Row::Script(9999)]);
        app.filter = Filter::Plugins;
        app.update_rows();
        assert!(app.rows.is_empty());
    }
    #[test]
    fn long_failure_is_modal_and_does_not_change_import_layout() {
        let (ctx, mut app) = fixture();
        for (index, size) in [[800.0, 560.0], [960.0, 680.0], [1440.0, 900.0]]
            .into_iter()
            .enumerate()
        {
            app.outcome = None;
            for i in 0..3 {
                let _ = render(&ctx, &mut app, size, (index * 10 + i) as f64);
            }
            let baseline = render(&ctx, &mut app, size, 40.0);
            app.outcome = Some(Outcome {
                failures: vec![("script.js".into(), "very-long-input-path/".repeat(100))],
                ..Default::default()
            });
            for i in 0..3 {
                let _ = render(&ctx, &mut app, size, 42.0 + i as f64);
            }
            let modal = render(&ctx, &mut app, size, 46.0);
            fn rail_rects(out: &egui::FullOutput) -> Vec<egui::Rect> {
                out.shapes
                    .iter()
                    .filter_map(|s| match &s.shape {
                        egui::Shape::Rect(r)
                            if s.clip_rect.max.x <= 246.1 && r.rect.width() > 180.0 =>
                        {
                            Some(r.rect)
                        }
                        _ => None,
                    })
                    .collect()
            }
            assert!(!rail_rects(&baseline).is_empty());
            assert_eq!(
                rail_rects(&baseline),
                rail_rects(&modal),
                "result resized the import rail"
            );
            assert!(ctx.memory(|m| m.top_modal_layer().is_some()));
            fn modal_fits(shape: &egui::Shape, size: [f32; 2]) -> bool {
                match shape {
                    egui::Shape::Rect(r) => {
                        r.fill == t::SURFACE_CONTAINER_HIGH
                            && r.rect.width() <= size[0] - 40.0
                            && r.rect.height() <= size[1]
                            && r.rect.min.y >= 0.0
                            && r.rect.max.y <= size[1]
                    }
                    egui::Shape::Vec(shapes) => shapes.iter().any(|s| modal_fits(s, size)),
                    _ => false,
                }
            }
            assert!(
                modal.shapes.iter().any(|s| modal_fits(&s.shape, size)),
                "modal exceeded viewport"
            );
        }
    }
    #[test]
    fn queue_deduplicates_and_is_bounded_without_stat_calls() {
        let (_, mut app) = fixture();
        app.enqueue(vec!["missing.js".into(), "missing.js".into()]);
        assert_eq!(app.queue.len(), 1);
        app.enqueue(
            (0..300)
                .map(|n| PathBuf::from(format!("missing{n}.js")))
                .collect(),
        );
        assert_eq!(app.queue.len(), 256);
        assert!(app.toast.is_some());
    }
}
