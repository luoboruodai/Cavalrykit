//! Event-driven application state. Slow work stays in Worker; UI decisions are explicit.
use super::theme as t;
use crate::{
    i18n::Key as K,
    installer::{ConflictChoice, InstallPlan, Outcome, PlannedUnit},
    library::Library,
    paths::CavalryPaths,
    restart::{RestartError, RestartResult},
    settings::{self, Settings},
    worker::{Event, Job, Worker},
};
use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc,
    },
    time::{Duration, Instant},
};
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
#[derive(Default)]
pub(crate) struct Audit {
    pub rail: Option<egui::Rect>,
    pub library: Option<egui::Rect>,
    pub card: Option<egui::Rect>,
    pub title: Option<egui::Rect>,
    pub status: Option<egui::Rect>,
    pub dialog: Option<egui::Rect>,
    pub install: Option<egui::Rect>,
    pub install_hint: Option<egui::Rect>,
    pub search: Option<egui::Rect>,
    pub filters: Vec<egui::Rect>,
    pub import_button: Option<egui::Rect>,
    pub import_art: Option<egui::Rect>,
    pub navigation: Vec<egui::Rect>,
    pub settings_body: Option<egui::Rect>,
    pub buttons: Vec<(K, egui::Rect)>,
}
pub struct CavalryKitApp {
    pub(crate) paths: Option<CavalryPaths>,
    pub(crate) filter: Filter,
    pub(crate) queue: Vec<PathBuf>,
    pub(crate) pending_plan: Option<InstallPlan>,
    pub(crate) conflicts: VecDeque<PlannedUnit>,
    resolved: Vec<(PlannedUnit, ConflictChoice)>,
    pub(crate) outcome: Option<Outcome>,
    pub(crate) library: Library,
    pub(crate) confirm_uninstall: Option<PathBuf>,
    pub(crate) cavalry_running: bool,
    pub(crate) about_open: bool,
    toast: Option<(String, Instant)>,
    pub(crate) search: String,
    pub(crate) busy: Option<K>,
    worker: Option<Worker>,
    picker: Option<Receiver<Vec<PathBuf>>>,
    picking_app: bool,
    pub(crate) rows: Vec<Row>,
    cached_query: Option<(String, Filter)>,
    pub(crate) logo: egui::TextureHandle,
    pub(crate) settings: Settings,
    pub(crate) settings_draft: Option<Settings>,
    pub(crate) settings_section: super::settings_panel::Section,
    settings_path: Option<PathBuf>,
    pub(crate) settings_error: Option<String>,
    applied_style: Option<Settings>,
    pub(crate) notice: Option<(K, Option<String>)>,
    pub(crate) restart_prompt: bool,
    pub(crate) restart_cancel: Option<Arc<AtomicBool>>,
    reload_scripts: usize,
    reload_plugins: usize,
    pub(crate) result_page: usize,
    pub(crate) apply_all: bool,
    was_focused: bool,
    last_refresh: Instant,
    pub(crate) audit: Audit,
}
impl CavalryKitApp {
    pub fn new(ctx: &egui::Context, initial: Vec<PathBuf>) -> Self {
        super::fonts::setup(ctx);
        let mut app = Self::with_paths(ctx, initial, CavalryPaths::detect());
        app.settings_path = settings::config_path();
        if let Some(path) = &app.settings_path {
            match Settings::load(path) {
                Ok(s) => app.settings = s,
                Err(e) => app.notice = Some((K::SettingsLoadError, Some(e))),
            }
        }
        app.apply_style(ctx);
        app
    }
    pub fn with_paths(
        ctx: &egui::Context,
        initial: Vec<PathBuf>,
        paths: Option<CavalryPaths>,
    ) -> Self {
        let worker = paths.clone().map(|p| {
            let ctx = ctx.clone();
            Worker::spawn(p, move || ctx.request_repaint())
        });
        let icon = super::icon::brand_icon();
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
            queue: vec![],
            pending_plan: None,
            conflicts: VecDeque::new(),
            resolved: vec![],
            outcome: None,
            library: Library::default(),
            confirm_uninstall: None,
            cavalry_running: false,
            about_open: false,
            toast: None,
            search: String::new(),
            busy: None,
            worker,
            picker: None,
            picking_app: false,
            rows: vec![],
            cached_query: None,
            logo,
            settings: Settings::default(),
            settings_draft: None,
            settings_section: super::settings_panel::Section::General,
            settings_path: None,
            settings_error: None,
            applied_style: None,
            notice: None,
            restart_prompt: false,
            restart_cancel: None,
            reload_scripts: 0,
            reload_plugins: 0,
            result_page: 0,
            apply_all: false,
            was_focused: false,
            last_refresh: Instant::now(),
            audit: Audit::default(),
        };
        if app.paths.is_some() {
            app.submit(Job::Scan, K::ScanBusy);
        }
        app.enqueue(initial);
        app.apply_style(ctx);
        app
    }
    pub(crate) fn prefs(&self) -> &Settings {
        self.settings_draft.as_ref().unwrap_or(&self.settings)
    }
    pub(crate) fn tr(&self, key: K) -> &'static str {
        self.prefs().language.tr(key)
    }
    pub(crate) fn p(&self) -> t::Palette {
        t::palette(self.prefs())
    }
    fn apply_style(&mut self, ctx: &egui::Context) {
        let settings = self.prefs().clone();
        if self.applied_style.as_ref() != Some(&settings) {
            if self.applied_style.as_ref().map(|s| s.language) != Some(settings.language) {
                super::fonts::setup_for_language(ctx, settings.language);
            }
            t::apply_settings(ctx, &settings);
            self.applied_style = Some(settings);
            ctx.request_repaint();
        }
    }
    pub(crate) fn record(&mut self, key: K, response: &egui::Response) {
        self.audit.buttons.push((key, response.rect));
    }
    pub(crate) fn enqueue(&mut self, files: Vec<PathBuf>) {
        let mut seen: HashSet<PathBuf> = self.queue.iter().cloned().collect();
        for file in files {
            if self.queue.len() >= 256 {
                self.push_toast(self.tr(K::QueueLimit));
                break;
            }
            if seen.insert(file.clone()) {
                self.queue.push(file);
            }
        }
        // A pending reload offer must wait for newly dropped input rather than interrupting it.
        if !self.queue.is_empty() {
            self.restart_prompt = false;
        }
    }
    pub(crate) fn submit(&mut self, job: Job, status: K) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| self.tr(K::WorkerError).to_owned())
            .and_then(|w| w.submit(job));
        match result {
            Ok(()) => self.busy = Some(status),
            Err(e) => {
                self.busy = None;
                self.notice = Some((K::OperationError, Some(e)));
            }
        }
    }
    pub(crate) fn push_toast(&mut self, text: impl Into<String>) {
        self.toast = Some((text.into(), Instant::now()));
    }
    pub(crate) fn refresh_library(&mut self) {
        if self.busy.is_none() && self.pending_plan.is_none() {
            self.submit(Job::Scan, K::ScanBusy);
            self.last_refresh = Instant::now();
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
    pub(crate) fn pick_files(&mut self, ctx: &egui::Context) {
        if self.picker.is_some() {
            return;
        }
        self.picking_app = false;
        let future = rfd::AsyncFileDialog::new()
            .set_title(self.tr(K::ChooseFiles))
            .add_filter(
                self.tr(K::Formats),
                &[
                    "jsc", "js", "jsx", "zip", "7z", "tar", "tgz", "tbz2", "gz", "bz2",
                ],
            )
            .add_filter(self.tr(K::AllFiles), &["*"])
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
    pub(crate) fn pick_application(&mut self, ctx: &egui::Context) {
        if self.picker.is_some() {
            return;
        }
        #[cfg(target_os = "macos")]
        let extensions = &["app"];
        #[cfg(not(target_os = "macos"))]
        let extensions = &["exe"];
        let future = rfd::AsyncFileDialog::new()
            .set_title(self.tr(K::SelectApp))
            .add_filter("Cavalry", extensions)
            .pick_file();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let files = pollster::block_on(future)
                .map(|f| vec![f.path().to_owned()])
                .unwrap_or_default();
            let _ = tx.send(files);
            ctx.request_repaint();
        });
        self.picking_app = true;
        self.picker = Some(rx);
    }
    pub(crate) fn open_settings(&mut self) {
        if let Some(status) = self.busy {
            self.push_toast(self.tr(status));
            return;
        }
        if self.settings_draft.is_none() {
            self.settings_draft = Some(self.settings.clone());
            self.settings_section = super::settings_panel::Section::General;
            self.settings_error = None;
        }
    }
    pub(crate) fn cancel_settings(&mut self, ctx: &egui::Context) {
        if self.busy != Some(K::SaveBusy) {
            self.settings_draft = None;
            self.settings_error = None;
            self.apply_style(ctx);
        }
    }
    pub(crate) fn save_settings(&mut self) {
        if self.busy.is_some() {
            return;
        }
        if let Some(draft) = self.settings_draft.clone() {
            if let Some(path) = self.settings_path.clone() {
                self.submit(Job::SaveSettings(path, draft), K::SaveBusy);
            } else if self.paths.is_none() {
                self.settings = draft;
                self.settings_draft = None;
                self.push_toast(self.tr(K::SettingsSaved));
            } else {
                self.settings_error = Some(self.tr(K::DirectoriesUnavailable).into());
            }
        }
    }
    fn poll_events(&mut self) {
        if let Some(picker) = &self.picker {
            match picker.try_recv() {
                Ok(files) => {
                    self.picker = None;
                    if self.picking_app {
                        if let (Some(draft), Some(path)) =
                            (self.settings_draft.as_mut(), files.first())
                        {
                            draft.cavalry_path = path.to_string_lossy().into_owned();
                        }
                        self.picking_app = false;
                    } else {
                        self.enqueue(files);
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.picker = None;
                    self.picking_app = false;
                    self.notice = Some((K::OperationError, None));
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
                    self.apply_all = false;
                    if self.conflicts.is_empty() {
                        self.submit(Job::Install(plan, vec![]), K::InstallBusy);
                    } else {
                        self.pending_plan = Some(plan);
                    }
                }
                Ok(Event::Installed(outcome, lib)) => {
                    self.busy = None;
                    self.replace_library(lib);
                    self.reload_scripts += outcome.installed_scripts.len();
                    self.reload_plugins += outcome.installed_plugins.len();
                    self.outcome = Some(outcome);
                    self.result_page = 0;
                }
                Ok(Event::Uninstalled(result, lib)) => {
                    self.busy = None;
                    self.replace_library(lib);
                    match result {
                        Ok(()) => self.push_toast(self.tr(K::Removed)),
                        Err(e) => self.notice = Some((K::OperationError, Some(e))),
                    }
                }
                Ok(Event::SettingsSaved(result)) => {
                    self.busy = None;
                    match result {
                        Ok(()) => {
                            if let Some(s) = self.settings_draft.take() {
                                self.settings = s;
                            }
                            self.settings_error = None;
                            self.push_toast(self.tr(K::SettingsSaved));
                        }
                        Err(e) => self.settings_error = Some(e),
                    }
                }
                Ok(Event::Restarted(result)) => {
                    self.busy = None;
                    self.restart_cancel = None;
                    match result {
                        Ok(RestartResult::LaunchRequested) => {
                            self.push_toast(self.tr(K::RestartDone))
                        }
                        Ok(RestartResult::AlreadyRunning) => {
                            self.push_toast(self.tr(K::AlreadyRunning))
                        }
                        Err(e) => {
                            let key = match e {
                                RestartError::NotFound | RestartError::InvalidPath => {
                                    K::RestartNotFound
                                }
                                RestartError::Multiple => K::RestartMultiple,
                                RestartError::Timeout | RestartError::QuitDeclined => {
                                    K::RestartTimeout
                                }
                                RestartError::Cancelled => K::RestartCancelled,
                                RestartError::System(_) => K::RestartError,
                            };
                            let detail = if let RestartError::System(text) = e {
                                Some(text)
                            } else {
                                None
                            };
                            self.notice = Some((key, detail));
                        }
                    }
                }
                Ok(Event::Discarded) => self.busy = None,
                Ok(Event::Failed(e)) => {
                    self.busy = None;
                    self.restart_cancel = None;
                    self.notice = Some((K::OperationError, Some(e)));
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.worker = None;
                    self.busy = None;
                    self.restart_cancel = None;
                    self.notice = Some((K::WorkerError, None));
                    break;
                }
            }
        }
    }
    pub(crate) fn resolve_current(&mut self, choice: ConflictChoice) {
        if let Some(unit) = self.conflicts.pop_front() {
            self.resolved.push((unit, choice));
        }
        if self.apply_all {
            self.resolved
                .extend(self.conflicts.drain(..).map(|u| (u, choice)));
        }
        if self.conflicts.is_empty() {
            if let Some(plan) = self.pending_plan.take() {
                let resolved = std::mem::take(&mut self.resolved);
                self.submit(Job::Install(plan, resolved), K::InstallBusy);
            }
        }
    }
    pub(crate) fn cancel_batch(&mut self) {
        if let Some(mut plan) = self.pending_plan.take() {
            let names = plan
                .units
                .iter()
                .chain(self.conflicts.iter())
                .chain(self.resolved.iter().map(|(u, _)| u))
                .map(|u| u.display.clone())
                .collect();
            self.conflicts.clear();
            self.resolved.clear();
            self.outcome = Some(Outcome {
                skipped: names,
                failures: std::mem::take(&mut plan.failures),
                ..Default::default()
            });
            self.submit(Job::Discard(plan), K::PlanBusy);
        }
    }
    pub(crate) fn dismiss_outcome(&mut self) {
        self.outcome = None;
        self.result_page = 0;
        if self.queue.is_empty() {
            self.offer_restart();
        }
    }
    fn offer_restart(&mut self) {
        self.restart_prompt = self
            .settings
            .restart_prompt
            .wants(self.reload_scripts, self.reload_plugins);
        if !self.restart_prompt {
            self.reload_scripts = 0;
            self.reload_plugins = 0;
        }
    }
    pub(crate) fn decline_restart(&mut self) {
        self.restart_prompt = false;
        self.reload_scripts = 0;
        self.reload_plugins = 0;
    }
    pub(crate) fn begin_restart(&mut self, allow_quit: bool) {
        if self.busy.is_some() || !self.queue.is_empty() {
            return;
        }
        self.decline_restart();
        let cancel = Arc::new(AtomicBool::new(false));
        self.restart_cancel = Some(cancel.clone());
        self.submit(
            Job::Restart(self.settings.cavalry_path.clone(), allow_quit, cancel),
            K::RestartBusy,
        );
        if self.busy.is_none() {
            self.restart_cancel = None;
        }
    }
    fn modal_active(&self) -> bool {
        !self.conflicts.is_empty()
            || self.outcome.is_some()
            || self.confirm_uninstall.is_some()
            || self.about_open
            || self.settings_draft.is_some()
            || self.restart_prompt
            || self.restart_cancel.is_some()
            || self.notice.is_some()
    }
    fn pump_queue(&mut self) {
        if self.busy.is_none()
            && !self.modal_active()
            && !self.queue.is_empty()
            && self.paths.is_some()
        {
            let files = std::mem::take(&mut self.queue);
            self.submit(Job::Plan(files), K::PlanBusy);
        }
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        self.audit = Audit::default();
        self.poll_events();
        self.apply_style(ctx);
        let dropped = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        self.enqueue(dropped);
        let focused = ctx.input(|i| i.focused);
        if focused
            && !self.was_focused
            && self.settings.refresh_on_focus
            && self.last_refresh.elapsed() > Duration::from_secs(2)
            && !self.modal_active()
            && self.paths.is_some()
        {
            self.refresh_library();
        }
        self.was_focused = focused;
        if !self.modal_active() {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F)) {
                ctx.memory_mut(|m| m.request_focus(egui::Id::new("library-search")));
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
                self.pick_files(ctx);
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::R)) {
                self.refresh_library();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Comma)) {
                self.open_settings();
            }
        }
        self.pump_queue();
        if ctx.input(|i| i.viewport().close_requested())
            && (self.busy.is_some() || self.picker.is_some())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.push_toast(self.tr(K::BusyClose));
        }
        let p = self.p();
        let compact = ctx.screen_rect().width() < 850.0;
        let gap = if compact { t::MD } else { t::LG };
        egui::TopBottomPanel::top("brand-header")
            .exact_height(t::HEADER)
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(gap as i8, 12)),
            )
            .show(ctx, |ui| self.show_brand_header(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg).inner_margin(gap))
            .show(ctx, |ui| {
                let area = ui.max_rect();
                let left_width = (area.width() * 0.27).clamp(216.0, 296.0);
                let left =
                    egui::Rect::from_min_size(area.min, egui::vec2(left_width, area.height()));
                let right =
                    egui::Rect::from_min_max(egui::pos2(left.right() + gap, area.top()), area.max);
                self.audit.rail = Some(left);
                self.audit.library = Some(right);
                let mut rail = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("import-column")
                        .max_rect(left),
                );
                rail.set_clip_rect(left.intersect(ui.clip_rect()));
                self.show_rail(&mut rail, left.height());
                let mut library = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("library-column")
                        .max_rect(right),
                );
                library.set_clip_rect(right.intersect(ui.clip_rect()));
                self.show_library(&mut library);
            });
        if let Some((text, at)) = &self.toast {
            let duration = Duration::from_secs(3);
            if at.elapsed() >= duration {
                self.toast = None;
            } else {
                ctx.request_repaint_after(duration.saturating_sub(at.elapsed()));
                egui::Area::new(egui::Id::new("toast"))
                    .order(egui::Order::Foreground)
                    .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -16.0])
                    .show(ctx, |ui| {
                        egui::Frame::new()
                            .fill(p.raised)
                            .stroke(egui::Stroke::new(1.0_f32, p.border))
                            .corner_radius(8)
                            .inner_margin(t::MD)
                            .show(ui, |ui| {
                                ui.set_max_width((ctx.screen_rect().width() - 64.0).min(640.0));
                                ui.label(t::text(text, 14.0, p.text));
                            });
                    });
            }
        }
        if self.restart_cancel.is_some() {
            self.show_restart_progress(ctx);
        } else if !self.conflicts.is_empty() {
            self.show_conflict_dialog(ctx);
        } else if self.outcome.is_some() {
            self.show_result_dialog(ctx);
        } else if self.restart_prompt {
            self.show_restart_dialog(ctx);
        } else if self.confirm_uninstall.is_some() {
            self.show_uninstall_dialog(ctx);
        } else if self.settings_draft.is_some() {
            self.show_settings(ctx);
        } else if self.about_open {
            self.show_about_dialog(ctx);
        } else if self.notice.is_some() {
            self.show_notice(ctx);
        }
    }
}
impl eframe::App for CavalryKitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }
}
impl Drop for CavalryKitApp {
    fn drop(&mut self) {
        if let Some(cancel) = &self.restart_cancel {
            cancel.store(true, Ordering::SeqCst);
        }
    }
}

/// Synthetic UI rendering only. Compiled out of production builds; never touches user libraries.
#[cfg(feature = "ui-preview")]
impl CavalryKitApp {
    pub fn preview(ctx: &egui::Context, language: crate::settings::Language, screen: &str) -> Self {
        super::fonts::setup(ctx);
        let mut app = Self::with_paths(ctx, vec![], None);
        app.settings.language = language;
        app.settings.refresh_on_focus = false;
        app.paths = Some(CavalryPaths::from_base(std::path::Path::new(
            "preview-library",
        )));
        app.cavalry_running = true;
        for (index, name) in [
            "Create Render Item.js",
            "Camera Control.jsc",
            "Export Composition.jsx",
            "Text Time Roll.js",
            "Shape Utilities.js",
            "タイミング調整.js",
            "레이어 정리.js",
            "路径工具.js",
            "Color Palette.js",
            "Animation Helpers.js",
            "Arrange Layers.js",
            "Preview Tools.js",
        ]
        .into_iter()
        .enumerate()
        {
            app.library.scripts.push(crate::library::ScriptEntry {
                name: name.into(),
                group: if index < 4 {
                    None
                } else {
                    Some("Utilities".into())
                },
                path: format!("preview-library/Scripts/{name}").into(),
                search_key: name.to_lowercase(),
            });
        }
        app.library.plugins.push(crate::library::PluginEntry {
            name: "Shape Toolkit".into(),
            author: Some("Demo author".into()),
            path: "preview-library/Plugins/Shape Toolkit".into(),
            search_key: "shape toolkit demo author".into(),
        });
        let (screen, light) = screen
            .strip_prefix("light-")
            .map(|s| (s, true))
            .unwrap_or((screen, false));
        if light {
            app.settings.theme = crate::settings::Theme::Light;
        }
        match screen {
            "settings" => app.open_settings(),
            "custom" => {
                app.open_settings();
                app.settings_section = super::settings_panel::Section::Appearance;
                let s = app.settings_draft.as_mut().unwrap();
                s.theme = crate::settings::Theme::Custom;
                s.accent = [95, 172, 255];
            }
            "appearance" => {
                app.open_settings();
                app.settings_section = super::settings_panel::Section::Appearance;
            }
            "installation" => {
                app.open_settings();
                app.settings_section = super::settings_panel::Section::Installation;
            }
            "cavalry-settings" => {
                app.open_settings();
                app.settings_section = super::settings_panel::Section::Cavalry;
            }
            "light" => app.settings.theme = crate::settings::Theme::Light,
            "result" => {
                app.outcome = Some(Outcome {
                    installed_scripts: vec!["Text Time Roll.js".into()],
                    installed_plugins: vec!["Shape Toolkit".into()],
                    restart_hint: true,
                    ..Default::default()
                })
            }
            "error" => {
                app.outcome = Some(Outcome {
                    failures: vec![(
                        "Example.zip".into(),
                        "Synthetic diagnostic: invalid archive header".into(),
                    )],
                    ..Default::default()
                })
            }
            "restart" => app.restart_prompt = true,
            "conflict" => app.conflicts.push_back(PlannedUnit {
                kind: crate::installer::UnitKind::Script,
                display: "Text Time Roll.js".into(),
                source: "incoming/Text Time Roll.js".into(),
                dest: "preview-library/Scripts/Text Time Roll.js".into(),
            }),
            _ => {}
        }
        app
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        installer::{TempGuard, UnitKind},
        settings::{Language, RestartPrompt, Theme},
    };
    fn fixture() -> (egui::Context, CavalryKitApp) {
        let ctx = egui::Context::default();
        super::super::fonts::setup(&ctx);
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
    fn settle(ctx: &egui::Context, app: &mut CavalryKitApp, size: [f32; 2], tick: &mut f64) {
        for _ in 0..4 {
            *tick += 1.0;
            let _ = render(ctx, app, size, *tick);
        }
    }
    #[test]
    fn four_language_layouts_keep_header_and_dialog_actions_aligned() {
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        for language in Language::ALL {
            for theme in [Theme::Cavalry, Theme::Light] {
                for size in [[720.0, 496.0], [900.0, 620.0], [1120.0, 760.0]] {
                    app.settings.language = language;
                    app.settings.theme = theme;
                    app.cavalry_running = true;
                    for dialog in 0..6 {
                        app.outcome = None;
                        app.confirm_uninstall = None;
                        app.about_open = false;
                        app.settings_draft = None;
                        app.restart_prompt = false;
                        app.conflicts.clear();
                        match dialog {
                            0 => {
                                app.outcome = Some(Outcome {
                                    installed_scripts: vec!["Very long 名称".repeat(20)],
                                    ..Default::default()
                                })
                            }
                            1 => app.conflicts.push_back(PlannedUnit {
                                kind: UnitKind::Script,
                                display: "Long 名称".repeat(40),
                                source: "synthetic.js".into(),
                                dest: PathBuf::from("long/path/".repeat(500)),
                            }),
                            2 => app.confirm_uninstall = Some("synthetic.js".into()),
                            3 => app.about_open = true,
                            4 => app.open_settings(),
                            5 => app.restart_prompt = true,
                            _ => {}
                        }
                        settle(&ctx, &mut app, size, &mut tick);
                        let title = app.audit.title.unwrap();
                        let status = app.audit.status.unwrap();
                        assert!(
                            title.right() + 4.0 <= status.left(),
                            "header overlap {language:?} {size:?}: {title:?} {status:?}"
                        );
                        assert!(
                            (title.center().y - status.center().y).abs() <= 4.0,
                            "status not on title row {language:?} {size:?}: {title:?} {status:?}"
                        );
                        assert!(
                            app.audit.rail.unwrap().right()
                                + (if size[0] < 850.0 { t::MD } else { t::LG })
                                - 0.5
                                <= app.audit.library.unwrap().left()
                        );
                        let panel = app.audit.dialog.unwrap();
                        assert!(
                            panel.left() >= -1.0
                                && panel.top() >= -1.0
                                && panel.right() <= size[0] + 1.0
                                && panel.bottom() <= size[1] + 1.0,
                            "modal exceeds window {language:?} {dialog} {size:?}: {panel:?}"
                        );
                        let buttons = &app.audit.buttons;
                        assert!(!buttons.is_empty());
                        assert!((buttons[0].1.right()-(panel.right()-t::LG-1.0)).abs()<3.0,"primary is not right-aligned {language:?} {dialog}: {panel:?} {buttons:?}");
                        for pair in buttons.windows(2) {
                            assert!(
                                pair[1].1.right() + 7.0 <= pair[0].1.left(),
                                "buttons overlap {language:?} {dialog}: {buttons:?}"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn original_left_import_right_filters_layout_stays_inside_both_columns() {
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        for language in Language::ALL {
            for theme in [Theme::Cavalry, Theme::Light, Theme::Custom] {
                app.settings.language = language;
                app.settings.theme = theme;
                for size in [[720.0, 496.0], [900.0, 620.0], [1120.0, 760.0]] {
                    for busy in [None, Some(K::PlanBusy), Some(K::RestartBusy)] {
                        app.busy = busy;
                        settle(&ctx, &mut app, size, &mut tick);
                        let left = app.audit.rail.unwrap();
                        let right = app.audit.library.unwrap();
                        let import = app.audit.install.unwrap();
                        assert!(
                            left.expand(0.5).contains_rect(import),
                            "import must stay left: {language:?} {size:?}"
                        );
                        assert!(left.right() + t::MD <= right.left() + 0.5);
                        assert!((import.top() - app.audit.card.unwrap().top()).abs() < 1.0);
                        for rect in [app.audit.search.unwrap(), app.audit.card.unwrap()] {
                            assert!(
                                right.expand(0.5).contains_rect(rect),
                                "right panel overflow: {language:?} {size:?} {rect:?} {right:?}"
                            );
                        }
                        assert_eq!(app.audit.filters.len(), 3);
                        for rect in &app.audit.filters {
                            assert!(right.contains_rect(*rect));
                            assert!(rect.bottom() + t::SM <= app.audit.search.unwrap().top());
                        }
                        let button = app.audit.import_button.unwrap();
                        let hint = app.audit.install_hint.unwrap();
                        let art = app.audit.import_art.unwrap();
                        for rect in [button, hint, art] {
                            assert!(
                                import.contains_rect(rect),
                                "import content clipped: {language:?} {size:?} {rect:?} {import:?}"
                            );
                        }
                        assert!(
                            art.bottom() + t::SM <= button.top(),
                            "decorative content overlaps input: {language:?} {size:?}"
                        );
                        assert!(button.bottom() + t::SM <= hint.top() + 0.5);
                        for rect in &app.audit.navigation {
                            assert!(
                                left.expand(0.5).contains_rect(*rect),
                                "left actions clipped {language:?} {size:?}: {rect:?} {left:?}"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn restored_filters_work_without_duplicate_library_titles_or_hover_repaint_loops() {
        let (ctx, mut app) = fixture();
        app.library.scripts.push(crate::library::ScriptEntry {
            name: "Example.js".into(),
            group: None,
            path: "synthetic/Example.js".into(),
            search_key: "example.js".into(),
        });
        app.library.plugins.push(crate::library::PluginEntry {
            name: "Example plugin".into(),
            author: None,
            path: "synthetic/Plugin".into(),
            search_key: "example plugin".into(),
        });
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let target = app.audit.filters[2].center();
        for pressed in [true, false] {
            tick += 0.02;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
                    time: Some(tick),
                    events: vec![
                        egui::Event::PointerMoved(target),
                        egui::Event::PointerButton {
                            pos: target,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
        }
        assert!(app.filter == Filter::Plugins);
        assert_eq!(app.rows, vec![Row::Plugin(0)]);
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let output = render(&ctx, &mut app, [900.0, 620.0], tick + 1.0);
        for shape in &output.shapes {
            if let egui::Shape::Text(text) = &shape.shape {
                for redundant in [
                    K::Library,
                    K::LibraryHint,
                    K::LibraryNav,
                    K::Shortcuts,
                    K::IdleHint,
                ] {
                    assert_ne!(
                        text.galley.job.text,
                        app.tr(redundant),
                        "redundant label returned"
                    );
                }
            }
        }
        assert_eq!(
            output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
            std::time::Duration::MAX,
            "steady hover should not run an animation loop"
        );
    }
    #[test]
    fn all_settings_sections_keep_content_and_actions_inside_the_modal() {
        use super::super::settings_panel::Section;
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        for language in Language::ALL {
            app.settings.language = language;
            for size in [[720.0, 496.0], [900.0, 620.0], [1120.0, 760.0]] {
                app.open_settings();
                app.settings_draft.as_mut().unwrap().theme = Theme::Custom;
                for section in [
                    Section::General,
                    Section::Appearance,
                    Section::Installation,
                    Section::Cavalry,
                ] {
                    app.settings_section = section;
                    settle(&ctx, &mut app, size, &mut tick);
                    let dialog = app.audit.dialog.unwrap();
                    let body = app.audit.settings_body.unwrap();
                    assert!(dialog.contains_rect(body));
                    assert!(body.bottom() + t::SM <= app.audit.buttons[0].1.top());
                    assert!(egui::Rect::from_min_size(egui::Pos2::ZERO, size.into())
                        .expand(1.0)
                        .contains_rect(dialog));
                }
                app.cancel_settings(&ctx);
            }
        }
    }
    #[test]
    fn library_scrollbar_keeps_contrast_after_local_style_overrides() {
        let (ctx, mut app) = fixture();
        for i in 0..100 {
            app.library.scripts.push(crate::library::ScriptEntry {
                name: format!("Synthetic {i}.js"),
                search_key: format!("synthetic {i}.js"),
                group: None,
                path: format!("synthetic/{i}.js").into(),
            });
        }
        let mut tick = 0.0;
        for theme in [Theme::Cavalry, Theme::Light, Theme::Custom] {
            app.settings.theme = theme;
            app.settings.background = [128, 128, 128];
            settle(&ctx, &mut app, [1120.0, 760.0], &mut tick);
            tick += 1.0;
            let output = render(&ctx, &mut app, [1120.0, 760.0], tick);
            let card = app.audit.card.unwrap();
            let p = app.p();
            let mut handles = 0;
            // Inspect the actual painted scrollbar, not just the global palette:
            // a local widget foreground override also affects egui's scroll thumb.
            for clipped in output.shapes {
                let egui::Shape::Rect(shape) = clipped.shape else {
                    continue;
                };
                if shape.rect.width() <= 8.5
                    && shape.rect.height() >= 39.5
                    && shape.rect.right() > card.right() - t::MD - 8.0
                    && card.contains_rect(shape.rect)
                    && shape.fill.a() > 0
                {
                    handles += 1;
                    let alpha = shape.fill.a() as f32 / 255.0;
                    let blended = t::rgb(std::array::from_fn(|i| {
                        (shape.fill[i] as f32 + p.panel[i] as f32 * (1.0 - alpha))
                            .round()
                            .min(255.0) as u8
                    }));
                    assert!(
                        t::contrast(blended, p.panel) >= 3.0 - 0.01,
                        "painted scrollbar contrast {theme:?}: {}",
                        t::contrast(blended, p.panel)
                    );
                }
            }
            assert_eq!(handles, 1, "expected one visible vertical scroll thumb");
        }
    }
    #[test]
    fn large_library_and_results_only_render_bounded_visible_work() {
        let (ctx, mut app) = fixture();
        for i in 0..10_000 {
            let name = format!("{i:05} {}.js", "长い名前한국어".repeat(30));
            app.library.scripts.push(crate::library::ScriptEntry {
                search_key: name.to_lowercase(),
                name,
                group: None,
                path: format!("synthetic/{i}.js").into(),
            });
        }
        let mut tick = 0.0;
        settle(&ctx, &mut app, [1120.0, 760.0], &mut tick);
        let output = render(&ctx, &mut app, [1120.0, 760.0], 10.0);
        assert!(
            app.audit
                .library
                .unwrap()
                .expand(0.5)
                .contains_rect(app.audit.card.unwrap()),
            "library card overflow {:?} {:?}",
            app.audit.library,
            app.audit.card
        );
        assert_eq!(app.rows.len(), 10_000);
        assert!(
            output.shapes.len() < 1000,
            "not virtualized: {}",
            output.shapes.len()
        );
        let pointer = app.rows.as_ptr();
        app.update_rows();
        assert_eq!(pointer, app.rows.as_ptr());
        app.search = "09999".into();
        app.update_rows();
        assert_eq!(app.rows, vec![Row::Script(9999)]);
        app.outcome = Some(Outcome {
            installed_scripts: (0..10_000).map(|n| format!("synthetic-{n}.js")).collect(),
            ..Default::default()
        });
        settle(&ctx, &mut app, [1120.0, 760.0], &mut tick);
        let output = render(&ctx, &mut app, [1120.0, 760.0], 20.0);
        assert!(
            output.shapes.len() < 1200,
            "unbounded result UI: {}",
            output.shapes.len()
        );
    }
    #[test]
    fn settings_preview_can_cancel_without_mutating_saved_preferences() {
        let (ctx, mut app) = fixture();
        let original = app.settings.clone();
        app.open_settings();
        let draft = app.settings_draft.as_mut().unwrap();
        draft.language = Language::Ko;
        draft.theme = Theme::Light;
        draft.scale_percent = 125;
        app.apply_style(&ctx);
        assert_eq!(app.tr(K::Settings), Language::Ko.tr(K::Settings));
        assert_eq!(app.settings, original);
        app.cancel_settings(&ctx);
        assert_eq!(app.settings, original);
        assert!(app.settings_draft.is_none());
        assert_eq!(ctx.zoom_factor(), 1.0);
    }
    #[test]
    fn restart_offer_waits_for_results_success_and_end_of_queue() {
        let (_, mut app) = fixture();
        app.reload_scripts = 1;
        app.outcome = Some(Outcome::default());
        app.dismiss_outcome();
        assert!(app.restart_prompt);
        app.decline_restart();
        assert_eq!(app.reload_scripts, 0);
        app.outcome = Some(Outcome {
            failures: vec![("bad.zip".into(), "invalid".into())],
            ..Default::default()
        });
        app.dismiss_outcome();
        assert!(!app.restart_prompt);
        app.settings.restart_prompt = RestartPrompt::Plugins;
        app.reload_scripts = 1;
        app.dismiss_outcome();
        assert!(!app.restart_prompt);
        app.reload_plugins = 1;
        app.queue.push("next.zip".into());
        app.dismiss_outcome();
        assert!(!app.restart_prompt);
        app.queue.clear();
        app.dismiss_outcome();
        assert!(app.restart_prompt);
        assert!(
            app.restart_cancel.is_none(),
            "offering restart must not execute it"
        );
    }
    #[test]
    fn queue_still_deduplicates_and_limits_without_filesystem_calls() {
        let (_, mut app) = fixture();
        app.enqueue(vec!["missing.js".into(), "missing.js".into()]);
        assert_eq!(app.queue.len(), 1);
        app.enqueue((0..300).map(|n| format!("{n}.js").into()).collect());
        assert_eq!(app.queue.len(), 256);
        assert!(app.toast.is_some());
    }
    #[test]
    fn conflict_cancel_never_executes_or_overwrites() {
        let (_, mut app) = fixture();
        let temp = tempfile::tempdir().unwrap();
        let unit = PlannedUnit {
            kind: UnitKind::Script,
            display: "sample.js".into(),
            source: temp.path().join("new.js"),
            dest: temp.path().join("old.js"),
        };
        std::fs::write(&unit.dest, "original").unwrap();
        app.conflicts.push_back(unit.clone());
        app.pending_plan = Some(InstallPlan {
            units: vec![],
            conflicts: vec![],
            failures: vec![],
            _temp: Some(TempGuard(temp.path().join("unused"))),
        });
        app.cancel_batch();
        assert!(app.pending_plan.is_none());
        assert!(app.conflicts.is_empty());
        assert_eq!(std::fs::read_to_string(unit.dest).unwrap(), "original");
    }
    #[test]
    fn cancelling_conflicts_keeps_prior_input_failures_visible() {
        let (_, mut app) = fixture();
        app.pending_plan = Some(InstallPlan {
            units: vec![],
            conflicts: vec![],
            failures: vec![("broken.zip".into(), "invalid archive".into())],
            _temp: None,
        });
        app.cancel_batch();
        assert_eq!(app.outcome.as_ref().unwrap().failures.len(), 1);
    }
    #[test]
    fn physical_dpi_and_zoom_keep_settings_actions_reachable() {
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        for dpi in [1.0, 1.25, 1.5, 2.0] {
            for scale in [90, 100, 110, 125] {
                app.settings.scale_percent = scale;
                app.settings.language = Language::En;
                app.settings_draft = None;
                app.open_settings();
                for _ in 0..5 {
                    tick += 1.0;
                    let mut input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(900.0, 620.0),
                        )),
                        time: Some(tick),
                        ..Default::default()
                    };
                    input
                        .viewports
                        .get_mut(&egui::ViewportId::ROOT)
                        .unwrap()
                        .native_pixels_per_point = Some(dpi);
                    let _ = ctx.run(input, |ctx| app.draw(ctx));
                }
                let screen = ctx.screen_rect();
                let rect = app.audit.dialog.unwrap();
                assert!(
                    screen.expand(1.0).contains_rect(rect),
                    "dpi={dpi} scale={scale}: {screen:?} {rect:?}"
                );
                for (_, rect) in &app.audit.buttons {
                    assert!(screen.contains_rect(*rect));
                }
                app.cancel_settings(&ctx);
            }
        }
    }
    #[test]
    fn escape_cancels_preferences_and_defers_restart_without_side_effects() {
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        app.open_settings();
        app.settings_draft.as_mut().unwrap().language = Language::Ja;
        settle(&ctx, &mut app, [1120.0, 760.0], &mut tick);
        let escape = egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: Some(egui::Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        let _ = ctx.run(
            egui::RawInput {
                events: vec![escape.clone()],
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        assert!(app.settings_draft.is_none());
        assert_eq!(app.settings.language, Language::Zh);
        app.restart_prompt = true;
        settle(&ctx, &mut app, [1120.0, 760.0], &mut tick);
        let _ = ctx.run(
            egui::RawInput {
                events: vec![escape],
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        assert!(!app.restart_prompt);
        assert!(app.restart_cancel.is_none());
    }
}
