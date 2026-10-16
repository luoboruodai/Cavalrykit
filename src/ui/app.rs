//! Event-driven application state. Slow work stays in Worker; UI decisions are explicit.
use super::theme as t;
use crate::{
    i18n::Key as K,
    installer::{ConflictChoice, InstallPlan, Outcome, PlannedUnit},
    library::Library,
    paths::CavalryPaths,
    restart::{RestartError, RestartResult},
    settings::{self, Language, Settings, Theme},
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
/// Only visual preferences rebuild egui styles/fonts and notify the native window.
/// Installation, refresh and Cavalry path edits must not reset an in-flight animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StyleKey {
    language: Language,
    theme: Theme,
    accent: [u8; 3],
    background: [u8; 3],
    scale_percent: u16,
    reduce_motion: bool,
}
impl From<&Settings> for StyleKey {
    fn from(s: &Settings) -> Self {
        Self {
            language: s.language,
            theme: s.theme,
            accent: s.accent,
            background: s.background,
            scale_percent: s.scale_percent,
            reduce_motion: s.reduce_motion,
        }
    }
}
#[derive(Default)]
pub(crate) struct Audit {
    pub rail: Option<egui::Rect>,
    pub rail_surface: Option<egui::Rect>,
    pub status_text: Option<egui::Rect>,
    pub status_pulse: f32,
    pub row_decorations: Vec<egui::Rect>,
    pub row_accent: Vec<egui::Rect>,
    pub row_text_ink: Vec<(egui::Rect, egui::Rect)>,
    pub import_icon: Option<egui::Rect>,
    pub library: Option<egui::Rect>,
    pub card: Option<egui::Rect>,
    pub empty_card: Option<egui::Rect>,
    pub empty_card_hit: Option<egui::Rect>,
    pub empty_clear: Option<egui::Rect>,
    pub empty_clear_visual: Option<egui::Rect>,
    pub empty_clear_icon: Option<egui::Rect>,
    pub empty_cluster: Option<egui::Rect>,
    pub empty_title_slot: Option<egui::Rect>,
    pub empty_title_visual: Option<egui::Rect>,
    pub empty_hint_slot: Option<egui::Rect>,
    pub empty_hint_visual: Option<egui::Rect>,
    pub empty_hint: Option<String>,
    pub empty_decorations: Vec<egui::Rect>,
    pub empty_dots: Vec<egui::Rect>,
    pub empty_tile_angles: Vec<f32>,
    pub title: Option<egui::Rect>,
    pub status: Option<egui::Rect>,
    pub dialog: Option<egui::Rect>,
    pub install: Option<egui::Rect>,
    pub install_hint: Option<egui::Rect>,
    pub search: Option<egui::Rect>,
    pub search_hover: f32,
    pub search_text: Option<egui::Rect>,
    pub search_icon: Option<egui::Rect>,
    pub search_hint: String,
    pub row_visuals: Vec<(egui::Rect, f32)>,
    pub row_hits: Vec<egui::Rect>,
    pub navigation_colors: Vec<egui::Color32>,
    pub refresh: Option<egui::Rect>,
    pub count: Option<egui::Rect>,
    pub count_color: Option<egui::Color32>,
    pub count_label: String,
    pub filters: Vec<egui::Rect>,
    pub import_button: Option<egui::Rect>,
    pub import_hit: Option<egui::Rect>,
    pub import_activated: bool,
    pub import_hover: f32,
    pub row_actions: Vec<(egui::Rect, egui::Rect)>,
    pub row_action_art: Vec<(egui::Rect, egui::Rect)>,
    pub row_action_ink: Vec<egui::Color32>,
    pub import_art: Option<egui::Rect>,
    pub import_title: Option<egui::Rect>,
    pub import_formats: Option<egui::Rect>,
    pub row_colors: Vec<egui::Color32>,
    pub navigation: Vec<egui::Rect>,
    pub settings_body: Option<egui::Rect>,
    pub path_input: Option<egui::Rect>,
    pub path_browse: Option<egui::Rect>,
    pub buttons: Vec<(K, egui::Rect)>,
}
pub struct CavalryKitApp {
    pub(crate) paths: Option<CavalryPaths>,
    pub(crate) filter: Filter,
    pub(crate) selected_row: Option<PathBuf>,
    pub(crate) list_view_key: Option<(Filter, String)>,
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
    // (trimmed raw query, normalized query, filter); the raw copy avoids lowercasing
    // and allocating on every frame while the search field is idle.
    cached_query: Option<(String, String, Filter)>,
    pub(crate) logo: egui::TextureHandle,
    pub(crate) settings: Settings,
    pub(crate) settings_draft: Option<Settings>,
    pub(crate) settings_section: super::settings_panel::Section,
    pub(crate) settings_closing: bool,
    settings_path: Option<PathBuf>,
    pub(crate) settings_error: Option<String>,
    applied_style: Option<StyleKey>,
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
            selected_row: None,
            list_view_key: None,
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
            settings_closing: false,
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
    /// Palette to paint this frame: eases toward `p()` during a theme switch.
    pub(crate) fn paint_palette(&self, ctx: &egui::Context) -> t::Palette {
        super::motion::themed_palette(ctx, self.p(), 0.30).0
    }
    pub(crate) fn apply_style(&mut self, ctx: &egui::Context) {
        let key = StyleKey::from(self.prefs());
        if self.applied_style != Some(key) {
            if self.applied_style.map(|s| s.language) != Some(key.language) {
                super::fonts::setup_for_language(ctx, key.language);
            }
            t::apply_settings(ctx, self.prefs());
            self.applied_style = Some(key);
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
        let raw = self.search.trim();
        if let Some((cached_raw, _, cached_filter)) = self.cached_query.as_ref() {
            if cached_raw == raw && *cached_filter == self.filter {
                return;
            }
        }
        let normalized = raw.to_lowercase();
        self.rows.clear();
        if self.filter != Filter::Plugins {
            self.rows.extend(
                self.library
                    .scripts
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.search_key.contains(&normalized))
                    .map(|(i, _)| Row::Script(i)),
            );
        }
        if self.filter != Filter::Scripts {
            self.rows.extend(
                self.library
                    .plugins
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.search_key.contains(&normalized))
                    .map(|(i, _)| Row::Plugin(i)),
            );
        }
        self.cached_query = Some((raw.to_owned(), normalized, self.filter));
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
            self.settings_closing = false;
            self.settings_section = super::settings_panel::Section::General;
            self.settings_error = None;
        }
    }
    pub(crate) fn cancel_settings(&mut self, ctx: &egui::Context) {
        if self.busy != Some(K::SaveBusy) && self.settings_draft.is_some() {
            self.settings_draft = Some(self.settings.clone());
            self.settings_error = None;
            self.settings_closing = true;
            self.apply_style(ctx);
            ctx.set_zoom_factor(self.settings.scale_percent as f32 / 100.0);
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
                self.settings = draft.clone();
                self.settings_draft = Some(draft);
                self.settings_closing = true;
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
                                self.settings = s.clone();
                                self.settings_draft = Some(s);
                                self.settings_closing = true;
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
                duplicates: std::mem::take(&mut plan.duplicates),
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
        let (p, theme_animating) = super::motion::themed_palette(ctx, self.p(), 0.30);
        if theme_animating {
            // Stock widgets (text edits, checkboxes) ease through the same colors.
            t::apply_palette(ctx, self.prefs(), p);
        }
        let modal_open = self.modal_active();
        let compact = ctx.screen_rect().width() < 850.0;
        let gap = if compact { t::MD } else { t::LG };
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg).inner_margin(0))
            .show(ctx, |ui| {
                let area = ui.max_rect();
                let sidebar_width = (area.width() * 0.28).clamp(256.0, 304.0);
                let sidebar =
                    egui::Rect::from_min_size(area.min, egui::vec2(sidebar_width, area.height()));
                let surface = sidebar.shrink(t::SM);
                self.audit.rail_surface = Some(surface);
                ui.painter().rect(
                    surface,
                    16,
                    p.panel,
                    egui::Stroke::new(1.0_f32, p.line),
                    egui::StrokeKind::Inside,
                );
                let left = sidebar.shrink(gap);
                let right = egui::Rect::from_min_max(
                    egui::pos2(sidebar.right() + gap, area.top() + gap),
                    area.max - egui::vec2(gap, gap),
                );
                self.audit.rail = Some(left);
                self.audit.library = Some(right);
                let mut rail = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("import-column")
                        .max_rect(left),
                );
                rail.set_clip_rect(left.intersect(ui.clip_rect()));
                if modal_open {
                    rail.disable();
                }
                self.show_rail(&mut rail, left.height());
                let mut library = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("library-column")
                        .max_rect(right),
                );
                library.set_clip_rect(right.intersect(ui.clip_rect()));
                if modal_open {
                    library.disable();
                }
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
        super::diagnostics::frame(ctx);
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
    /// Deterministic synthetic demo input. This method is absent from release builds.
    /// It never drops a real path, clicks the picker, saves preferences, or starts Worker.
    pub fn preview_motion_input(&mut self, ctx: &egui::Context, frame: u32) -> egui::RawInput {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(960.0, 660.0),
            )),
            time: Some(frame as f64 / 30.0),
            ..Default::default()
        };
        let mut point = egui::pos2(950.0, 650.0);
        if (24..52).contains(&frame) {
            if let Some(rect) = self.audit.import_button {
                point = rect.center();
            }
        }
        if (55..90).contains(&frame) {
            input.hovered_files.push(egui::HoveredFile {
                path: None,
                mime: "application/zip".into(),
            });
            if let Some(rect) = self.audit.install {
                point = rect.center();
            }
        }
        for (at, index) in [(101, 2), (132, 0)] {
            if (at - 5..at + 10).contains(&frame) && self.audit.filters.len() == 3 {
                point = self.audit.filters[index].center();
            }
            if (frame == at || frame == at + 1) && self.audit.filters.len() == 3 {
                input.events.push(egui::Event::PointerButton {
                    pos: self.audit.filters[index].center(),
                    button: egui::PointerButton::Primary,
                    pressed: frame == at,
                    modifiers: egui::Modifiers::NONE,
                });
            }
        }
        if (112..128).contains(&frame) {
            if let Some(rect) = self.audit.navigation.first() {
                point = rect.center();
            }
        }
        if frame == 150 {
            self.open_settings();
        }
        if frame == 177 {
            self.settings_section = super::settings_panel::Section::Appearance;
        }
        if frame == 202 {
            if let Some(s) = self.settings_draft.as_mut() {
                s.theme = crate::settings::Theme::Light;
            }
        }
        if frame == 222 {
            self.cancel_settings(ctx);
        }
        input.events.insert(0, egui::Event::PointerMoved(point));
        input
    }
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
        if screen == "mixed-names" {
            app.library.scripts.clear();
            for name in [
                "Affine仿射工具.js",
                "关键帧插值Keyframe.Tools.jsx",
                "Replace素材路径.jsc",
                "清理未使用的资产UnusedAssets.js",
                "镜头与渲染Camera.Render.js",
                "Shape曲线_路径.jsx",
                "日本語タイミング.js",
                "한글레이어정리.js",
            ] {
                app.library.scripts.push(crate::library::ScriptEntry {
                    name: name.into(),
                    group: None,
                    path: format!("preview-library/{name}").into(),
                    search_key: name.to_lowercase(),
                });
            }
        }
        match screen {
            "stopped" => app.cavalry_running = false,
            "plugins" => app.filter = Filter::Plugins,
            "scripts" => app.filter = Filter::Scripts,
            "no-results" | "hover-empty" | "hover-clear" => {
                app.search = "missing-script-or-plugin".into()
            }
            "duplicate-result" => {
                app.outcome = Some(Outcome {
                    duplicates: vec!["Text Time Roll.js".into(), "Shape Toolkit".into()],
                    ..Default::default()
                })
            }
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
    #[test]
    fn nonvisual_preferences_do_not_reset_style_or_native_window_theme() {
        let (ctx, mut app) = fixture();
        let before = app.applied_style;
        app.settings.refresh_on_focus = !app.settings.refresh_on_focus;
        app.settings.restart_prompt = RestartPrompt::Never;
        app.settings.cavalry_path = "synthetic/nonexistent/Cavalry.app".into();
        app.apply_style(&ctx);
        assert_eq!(app.applied_style, before);
        let mut visual = app.settings.clone();
        visual.reduce_motion = !visual.reduce_motion;
        assert_ne!(StyleKey::from(&visual), before.unwrap());
        visual = app.settings.clone();
        visual.theme = Theme::Light;
        assert_ne!(StyleKey::from(&visual), before.unwrap());
    }
    #[test]
    fn search_text_icon_refresh_and_hint_align_in_all_languages_and_themes() {
        for language in Language::ALL {
            for theme in [Theme::Cavalry, Theme::Light, Theme::Custom] {
                let (ctx, mut app) = fixture();
                app.settings.language = language;
                app.settings.theme = theme;
                app.search = "Aa名称".into();
                let mut tick = 0.0;
                settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
                let search = app.audit.search.unwrap();
                let icon = app.audit.search_icon.unwrap();
                let text = app.audit.search_text.unwrap();
                let refresh = app.audit.refresh.unwrap();
                assert!(
                    (text.center().y - search.center().y).abs() < 0.5,
                    "{language:?} {text:?} {search:?}"
                );
                assert!((icon.center().y - text.center().y).abs() < 0.5);
                assert!((refresh.center().y - search.center().y).abs() < 0.5);
                assert_eq!(refresh.size(), egui::Vec2::splat(t::SEARCH_HEIGHT));
                assert!(search.right() + t::SM <= refresh.left() + 0.1);
                assert!(
                    app.audit.search_hint.is_empty(),
                    "never paint a rotating placeholder over user text"
                );
                assert!(t::contrast(t::placeholder_ink(app.p()), app.p().panel) >= 4.5);
                for c in &app.audit.navigation_colors {
                    assert_eq!(*c, t::category(app.p(), false).color);
                }
                let title = app.audit.title.unwrap();
                let status = app.audit.status.unwrap();
                assert!(title.right() + t::SM <= status.left() + 0.5);
                assert!((title.center().y - status.center().y).abs() < 0.5);
            }
        }
    }
    #[test]
    fn row_cards_rise_and_fade_in_order_with_fixed_hit_targets_then_stop() {
        let (ctx, mut app) = fixture();
        for i in 0..12 {
            app.library.scripts.push(crate::library::ScriptEntry {
                name: format!("Row {i}.js"),
                path: format!("synthetic/{i}.js").into(),
                group: None,
                search_key: format!("row {i}.js"),
            });
        }
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        app.filter = Filter::Scripts;
        tick += 1.0;
        let start = tick;
        let _ = render(&ctx, &mut app, [900.0, 620.0], tick);
        let targets = app.audit.row_hits.clone();
        assert!(app.audit.row_visuals.iter().all(|(_, v)| *v == 0.0));
        let _ = render(&ctx, &mut app, [900.0, 620.0], start + 0.16);
        assert_eq!(targets, app.audit.row_hits);
        let mid = app.audit.row_visuals.clone();
        assert!(mid[0].1 > mid[1].1 && mid[1].1 > mid[2].1);
        assert!(mid[0].0.top() > targets[0].top());
        assert!(mid[0].1 < 1.0);
        for (rect, alpha) in &mid {
            if *alpha > 0.0 && *alpha < 1.0 {
                assert!(rect.is_finite());
            }
        }
        let _ = render(&ctx, &mut app, [900.0, 620.0], start + 0.8);
        assert_eq!(targets, app.audit.row_hits);
        assert!(app.audit.row_visuals.iter().all(|(_, a)| *a == 1.0));
        let output = render(&ctx, &mut app, [900.0, 620.0], start + 1.0);
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT].repaint_delay > Duration::from_secs(1)
        );
        app.settings.reduce_motion = true;
        app.filter = Filter::All;
        let _ = render(&ctx, &mut app, [900.0, 620.0], start + 2.0);
        assert!(app.audit.row_visuals.iter().all(|(_, a)| *a == 1.0));
    }
    #[test]
    fn decorative_dots_have_no_action_or_popup_and_remove_requires_confirmation() {
        let (ctx, mut app) = fixture();
        app.library.scripts.push(crate::library::ScriptEntry {
            name: "Synthetic.js".into(),
            group: None,
            path: "synthetic/a.js".into(),
            search_key: "synthetic.js".into(),
        });
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let dot = app.audit.row_decorations[0];
        mouse_click(&ctx, &mut app, dot.center(), &mut tick);
        assert!(app.selected_row.is_none());
        assert!(app.confirm_uninstall.is_none());
        assert!(!ctx.memory(|m| m.any_popup_open()));
        let (_, remove) = app.audit.row_actions[0];
        mouse_click(&ctx, &mut app, remove.center(), &mut tick);
        assert_eq!(
            app.confirm_uninstall.as_deref(),
            Some(std::path::Path::new("synthetic/a.js"))
        );
        assert!(app.busy.is_none());
    }
    #[test]
    fn settings_modal_blocks_the_restored_direct_row_actions() {
        let (ctx, mut app) = fixture();
        app.library.scripts.push(crate::library::ScriptEntry {
            name: "Synthetic.js".into(),
            group: None,
            path: "synthetic/a.js".into(),
            search_key: "synthetic.js".into(),
        });
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let (_, remove) = app.audit.row_actions[0];
        app.open_settings();
        tick += 0.01;
        let _ = render(&ctx, &mut app, [900.0, 620.0], tick);
        mouse_click(&ctx, &mut app, remove.center(), &mut tick);
        assert!(app.settings_draft.is_some());
        assert!(app.confirm_uninstall.is_none());
    }
    #[test]
    fn settled_row_hover_expands_the_surface_but_never_the_hit_or_menu_target() {
        let (ctx, mut app) = fixture();
        app.library.scripts.push(crate::library::ScriptEntry {
            name: "Synthetic.js".into(),
            group: None,
            path: "synthetic/a.js".into(),
            search_key: "synthetic.js".into(),
        });
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let hit = app.audit.row_hits[0];
        let surface = app.audit.row_visuals[0].0;
        let more = app.audit.row_actions[0];
        let point = hit.center();
        for i in 0..20 {
            tick += 1.0 / 60.0;
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(tick),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
                    events: vec![egui::Event::PointerMoved(point)],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            assert_eq!(hit, app.audit.row_hits[0]);
            assert_eq!(more, app.audit.row_actions[0]);
            if i == 19 {
                assert_eq!(app.audit.row_visuals[0].0, hit);
                assert!(surface.left() > hit.left());
            }
        }
        assert!(app.confirm_uninstall.is_none());
    }
    #[test]
    fn toolbar_aligns_to_card_and_sidebar_is_an_inset_rounded_surface() {
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        for count in [1, 100] {
            app.replace_library(Library::default());
            for i in 0..count {
                app.library.scripts.push(crate::library::ScriptEntry {
                    name: format!("Letter字形{i}.js"),
                    group: None,
                    path: format!("synthetic/{i}.js").into(),
                    search_key: format!("letter{i}"),
                });
            }
            for size in [[720.0, 496.0], [1120.0, 760.0]] {
                settle(&ctx, &mut app, size, &mut tick);
                let row = app.audit.row_visuals[0].0;
                let search = app.audit.search.unwrap();
                let refresh = app.audit.refresh.unwrap();
                assert!((search.left() - row.left()).abs() < 0.1);
                assert!(
                    (refresh.right() - row.right()).abs() < 0.1,
                    "toolbar right {refresh:?} row {row:?}"
                );
                let surface = app.audit.rail_surface.unwrap();
                assert!(surface.left() > 0.0 && surface.top() > 0.0 && surface.bottom() < size[1]);
                let text = app.audit.status_text.unwrap();
                let badge = app.audit.status.unwrap();
                assert!((text.center().y - badge.center().y).abs() < 0.1);
                assert!((badge.right() - text.right() - 10.0).abs() < 0.1);
                let (title, detail) = app.audit.row_text_ink[0];
                assert!((detail.top() - title.bottom() - t::XS).abs() < 0.1);
                assert!((title.union(detail).center().y - row.center().y).abs() < 0.1);
                for rect in &app.audit.row_accent {
                    assert!(rect.top() >= row.top() + 7.9 || rect.top() > row.bottom());
                }
            }
        }
    }
    #[test]
    fn row_reveal_only_changes_opacity_never_disabled_gray_tint() {
        for theme in [Theme::Cavalry, Theme::Light] {
            let (ctx, mut app) = fixture();
            app.settings.theme = theme;
            app.library.scripts.push(crate::library::ScriptEntry {
                name: "Mixed 字形.js".into(),
                group: None,
                path: "synthetic/a.js".into(),
                search_key: "mixed".into(),
            });
            let mut tick = 0.0;
            settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
            app.filter = Filter::Scripts;
            tick += 1.0;
            let _ = render(&ctx, &mut app, [900.0, 620.0], tick);
            for dt in [0.03, 0.07, 0.12, 0.18, 0.24, 0.32, 0.40, 0.80] {
                let output = render(&ctx, &mut app, [900.0, 620.0], tick + dt);
                let (rect, alpha) = app.audit.row_visuals[0];
                let shape = output
                    .shapes
                    .iter()
                    .find_map(|s| {
                        if let egui::Shape::Rect(r) = &s.shape {
                            if (r.rect.min - rect.min).length() < 0.1
                                && (r.rect.max - rect.max).length() < 0.1
                            {
                                Some(r)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .expect("actual row background shape");
                assert_eq!(
                    shape.fill,
                    app.p().panel.gamma_multiply(alpha),
                    "gray disabled tint must not contaminate the reveal: {theme:?} {dt}"
                );
            }
        }
    }
    #[test]
    fn program_location_field_and_browse_keep_their_row_inside_small_modals() {
        for language in Language::ALL {
            for theme in [Theme::Cavalry, Theme::Light] {
                let (ctx, mut app) = fixture();
                app.settings.language = language;
                app.settings.theme = theme;
                app.open_settings();
                app.settings_section = super::super::settings_panel::Section::Cavalry;
                app.settings_draft.as_mut().unwrap().cavalry_path =
                    "synthetic/长名称/Application/".repeat(20);
                let mut tick = 0.0;
                settle(&ctx, &mut app, [720.0, 496.0], &mut tick);
                let field = app.audit.path_input.unwrap();
                let browse = app.audit.path_browse.unwrap();
                let body = app.audit.settings_body.unwrap();
                assert!(
                    body.contains_rect(field) && body.contains_rect(browse),
                    "{language:?} {body:?} {field:?} {browse:?}"
                );
                assert!((field.center().y - browse.center().y).abs() < 0.5);
                assert!(field.right() + t::SM <= browse.left() + 0.5);
                assert!(browse.right() <= body.right() - t::MD + 0.5);
            }
        }
    }
    #[test]
    fn row_hover_animates_both_actions_and_decoration_without_moving_their_targets() {
        for theme in [Theme::Cavalry, Theme::Light] {
            for reduce in [false, true] {
                let (ctx, mut app) = fixture();
                app.settings.theme = theme;
                app.settings.reduce_motion = reduce;
                app.library.scripts.push(crate::library::ScriptEntry {
                    name: "Shared hover.js".into(),
                    group: None,
                    path: "synthetic/shared.js".into(),
                    search_key: "shared hover.js".into(),
                });
                let mut tick = 0.0;
                settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
                let targets = app.audit.row_actions[0];
                let art = app.audit.row_action_art[0];
                let dots = app.audit.row_decorations[0];
                let ink = app.audit.row_action_ink[0];
                let row = app.audit.row_hits[0];
                let point = row.left_center() + egui::vec2(70.0, 0.0);
                let start = tick;
                let mut middle = false;
                let mut x = art.0.center().x;
                for frame in 0..=24 {
                    tick = start + frame as f64 / 60.0;
                    let _ = ctx.run(
                        egui::RawInput {
                            time: Some(tick),
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(900.0, 620.0),
                            )),
                            events: vec![egui::Event::PointerMoved(point)],
                            ..Default::default()
                        },
                        |ctx| app.draw(ctx),
                    );
                    assert_eq!(targets, app.audit.row_actions[0]);
                    let now = app.audit.row_action_art[0];
                    for (hit, visual) in [(targets.0, now.0), (targets.1, now.1)] {
                        assert!(hit.contains_rect(visual));
                    }
                    assert!(now.0.center().x >= x - 0.01);
                    x = now.0.center().x;
                    assert!(
                        (now.0.center().x
                            - art.0.center().x
                            - (now.1.center().x - art.1.center().x))
                            .abs()
                            < 0.01
                    );
                    let dx = now.0.center().x - art.0.center().x;
                    assert!(
                        (app.audit.row_decorations[0].center().x - dots.center().x - dx).abs()
                            < 0.01
                    );
                    middle |= dx > 0.01 && dx < t::ROW_INSET - 0.01;
                    if reduce {
                        assert!((dx - t::ROW_INSET).abs() < 0.01);
                    }
                }
                assert_eq!(middle, !reduce);
                assert_ne!(ink, app.audit.row_action_ink[0]);
                assert!(app.audit.row_action_art[0].0.width() > art.0.width());
                assert!(app.confirm_uninstall.is_none() && app.selected_row.is_none());
                let _ = render(&ctx, &mut app, [900.0, 620.0], start + 1.0);
                let _ = render(&ctx, &mut app, [900.0, 620.0], start + 1.2);
                let output = render(&ctx, &mut app, [900.0, 620.0], start + 2.0);
                assert!(
                    output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
                        > Duration::from_secs(1),
                    "settled group must not add idle animation"
                );
            }
        }
    }
    #[test]
    fn final_badges_use_the_tuned_material_and_legible_ink_in_both_themes() {
        for theme in [Theme::Cavalry, Theme::Light] {
            let (ctx, mut app) = fixture();
            app.settings.theme = theme;
            let mut tick = 0.0;
            settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
            let out = render(&ctx, &mut app, [900.0, 620.0], tick + 1.0);
            let p = app.p();
            let painted = |rect: egui::Rect| {
                out.shapes
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
                    .unwrap()
            };
            let status = painted(app.audit.status.unwrap());
            assert_eq!(status.fill, t::neutral_surface(p));
            assert_eq!(status.stroke, t::neutral_stroke(p));
            let count = painted(app.audit.count.unwrap());
            let ink = app.audit.count_color.unwrap();
            assert!(t::contrast(ink, count.fill) >= 4.5);
            assert!(t::contrast(p.muted, status.fill) >= 4.5);
            if theme == Theme::Cavalry {
                assert_eq!(status.fill, p.surface);
                assert_eq!(count.fill, t::category(p, false).tint);
            } else {
                assert_eq!(status.stroke.width, 0.0);
                assert!(t::luminance(count.fill) < t::luminance(p.bg) - 0.08);
            }
        }
    }
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
                        let left = app.audit.rail.unwrap();
                        assert!(left.contains_rect(title) && left.contains_rect(status));
                        assert!(
                            title.right() + t::SM - 0.5 <= status.left()
                                && (title.center().y - status.center().y).abs() < 0.5,
                            "sidebar identity/status overlap {language:?}"
                        );
                        assert!(left.right() + t::MD <= app.audit.library.unwrap().left());
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
                        assert!(app.audit.status.unwrap().bottom() + t::SM <= import.top());
                        assert!(app.audit.title.unwrap().top() < app.audit.filters[0].bottom());
                        for rect in [app.audit.search.unwrap(), app.audit.card.unwrap()] {
                            assert!(
                                right.expand(0.5).contains_rect(rect),
                                "right panel overflow: {language:?} {size:?} {rect:?} {right:?}"
                            );
                        }
                        assert_eq!(app.audit.filters.len(), 3);
                        for rect in &app.audit.filters {
                            assert!(
                                right.expand(0.01).contains_rect(*rect),
                                "filter outside column {rect:?} {right:?}"
                            );
                            assert!(rect.bottom() + t::SM <= app.audit.search.unwrap().top());
                        }
                        let button = app.audit.import_button.unwrap();
                        let hint = app.audit.install_hint.unwrap();
                        let art = app.audit.import_art.unwrap();
                        assert!(
                            import.contains_rect(art),
                            "drop content clipped {language:?} {size:?}: {art:?} {import:?}"
                        );
                        assert!(left.contains_rect(button) && left.contains_rect(hint));
                        assert!(
                            import.bottom() + t::SM <= button.top() + 0.5,
                            "CTA must be BELOW drop card"
                        );
                        assert!(button.bottom() + t::SM <= hint.top() + 0.5);
                        let count = app.audit.count.unwrap();
                        assert!(count.left() >= app.audit.filters[2].right() + t::SM);
                        assert!(count.left() - app.audit.filters[2].right() <= t::LG);
                        assert!(right.contains_rect(count));
                        let refresh = app.audit.refresh.unwrap();
                        assert!(
                            (refresh.center().y - app.audit.search.unwrap().center().y).abs() < 1.0
                        );
                        assert!(app.audit.search.unwrap().right() + t::SM <= refresh.left() + 0.5);
                        for pair in app.audit.navigation.windows(2) {
                            assert!((pair[0].left() - pair[1].left()).abs() < 0.5);
                            assert!((pair[0].width() - pair[1].width()).abs() < 0.5);
                            assert!(pair[0].bottom() + t::XS <= pair[1].top() + 0.5);
                        }
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
    fn idle_search_reuses_normalized_query_cache_until_query_or_filter_changes() {
        let (ctx, mut app) = fixture();
        app.library.scripts.push(crate::library::ScriptEntry {
            name: "Example.js".into(),
            group: None,
            path: "synthetic/Example.js".into(),
            search_key: "example.js".into(),
        });
        app.search = "  EXAMPLE  ".into();
        app.update_rows();
        let first = app.cached_query.clone();
        assert_eq!(app.rows, vec![Row::Script(0)]);
        app.search = " EXAMPLE ".into();
        app.update_rows();
        assert_eq!(app.cached_query, first);
        app.filter = Filter::Plugins;
        app.update_rows();
        assert_ne!(app.cached_query, first);
        assert!(app.rows.is_empty());
        drop(ctx);
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
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT].repaint_delay > Duration::from_secs(10),
            "steady hover must sleep until the next occasional hint, not poll frames"
        );
    }
    #[test]
    fn opening_modal_blocks_an_in_flight_background_click_from_its_first_frame() {
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let target = app.audit.filters[2].center();
        for pressed in [true, false] {
            if !pressed {
                app.open_settings();
            }
            tick += 0.016;
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(tick),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
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
        assert!(
            app.filter == Filter::All,
            "fading modal may not pass input through to the library"
        );
        assert!(app.confirm_uninstall.is_none());
        assert!(app.busy.is_none());
    }
    #[test]
    fn drop_choreography_keeps_targets_still_and_settles_without_an_idle_timer() {
        for reduce in [false, true] {
            let (ctx, mut app) = fixture();
            app.settings.reduce_motion = reduce;
            let mut tick = 0.0;
            settle(&ctx, &mut app, [720.0, 496.0], &mut tick);
            let button = app.audit.import_button.unwrap();
            let search = app.audit.search.unwrap();
            let filters = app.audit.filters.clone();
            let mut art_top = Vec::new();
            for frame in 0..90 {
                let hovering = (4..48).contains(&frame);
                let mut input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(720.0, 496.0),
                    )),
                    time: Some(tick + frame as f64 / 60.0),
                    ..Default::default()
                };
                if hovering {
                    input.hovered_files.push(egui::HoveredFile {
                        path: None,
                        mime: "application/zip".into(),
                    });
                }
                let output = ctx.run(input, |ctx| app.draw(ctx));
                assert_eq!(button, app.audit.import_button.unwrap());
                assert_eq!(search, app.audit.search.unwrap());
                assert_eq!(filters, app.audit.filters);
                assert!(app
                    .audit
                    .install
                    .unwrap()
                    .contains_rect(app.audit.import_art.unwrap()));
                art_top.push(app.audit.import_art.unwrap().top());
                if frame == 89 {
                    assert!(output.viewport_output[&egui::ViewportId::ROOT].repaint_delay > Duration::from_secs(10), "drop motion is settled; only the optional long-delay hint timer may remain");
                }
            }
            assert_eq!(art_top[0], art_top[89]);
            if reduce {
                assert_eq!(art_top[4], art_top[47]);
            } else {
                assert!(art_top[8] != art_top[47]);
            }
        }
    }
    fn mouse_click(
        ctx: &egui::Context,
        app: &mut CavalryKitApp,
        point: egui::Pos2,
        tick: &mut f64,
    ) {
        for down in [None, Some(true), Some(false)] {
            *tick += 0.04;
            let mut events = vec![egui::Event::PointerMoved(point)];
            if let Some(pressed) = down {
                events.push(egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(*tick),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
        }
    }
    #[test]
    fn separate_drop_surface_and_below_card_action_are_safe_nonoverlapping_targets() {
        let (ctx, mut app) = fixture();
        app.paths = Some(CavalryPaths::from_base(std::path::Path::new("synthetic")));
        let (_tx, rx) = std::sync::mpsc::channel();
        app.picker = Some(rx); // never open a native picker in unit tests
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let hit = app.audit.import_hit.unwrap();
        let visual = app.audit.import_button.unwrap();
        assert!(!hit.intersects(visual));
        assert!(hit.bottom() + t::SM <= visual.top() + 0.5);
        for point in [
            hit.left_top() + egui::vec2(8.0, 8.0),
            hit.center(),
            visual.center(),
            hit.right_bottom() - egui::vec2(8.0, 8.0),
        ] {
            mouse_click(&ctx, &mut app, point, &mut tick);
            assert!(app.audit.import_activated, "card misses point {point:?}");
        }
        mouse_click(
            &ctx,
            &mut app,
            egui::pos2(hit.right() + 6.0, hit.center().y),
            &mut tick,
        );
        assert!(
            !app.audit.import_activated,
            "expanded hit zone may not cover column gap"
        );
    }
    #[test]
    fn search_icon_and_blank_padding_focus_the_input() {
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let search = app.audit.search.unwrap();
        let id = egui::Id::new("library-search");
        for point in [
            search.left_center() + egui::vec2(8.0, 0.0),
            search.left_top() + egui::vec2(2.0, 2.0),
            search.right_center() - egui::vec2(8.0, 0.0),
        ] {
            ctx.memory_mut(|m| m.surrender_focus(id));
            mouse_click(&ctx, &mut app, point, &mut tick);
            assert!(
                ctx.memory(|m| m.has_focus(id)),
                "visible field missed {point:?}"
            );
        }
        app.search = "test".into();
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        // 没有 × 按钮：点击右端空白仍然只是聚焦，清空由空状态按钮或键盘完成。
        ctx.memory_mut(|m| m.surrender_focus(id));
        mouse_click(
            &ctx,
            &mut app,
            search.right_center() - egui::vec2(6.0, 0.0),
            &mut tick,
        );
        assert_eq!(app.search, "test");
        assert!(ctx.memory(|m| m.has_focus(id)));
    }
    #[test]
    fn empty_card_hover_animates_all_content_without_moving_targets() {
        let (ctx, mut app) = fixture();
        app.search = "missing".into();
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let card = app.audit.empty_card.unwrap();
        let hit = app.audit.empty_card_hit.unwrap();
        let clear = app.audit.empty_clear.unwrap();
        let clear_visual = app.audit.empty_clear_visual.unwrap();
        let clear_icon = app.audit.empty_clear_icon.unwrap();
        let center = card.center();
        assert!(
            hit.width() > 480.0 && hit.height() >= 360.0,
            "empty hero should remain spacious: {hit:?}"
        );
        let output = render(&ctx, &mut app, [900.0, 620.0], tick);
        let card_shape = output
            .shapes
            .iter()
            .find_map(|clipped| {
                if let egui::Shape::Rect(shape) = &clipped.shape {
                    (shape.rect == card).then_some(shape)
                } else {
                    None
                }
            })
            .expect("empty card shape");
        let tone = t::category(app.p(), false);
        assert_eq!(card_shape.stroke.width, 0.5);
        assert_eq!(
            card_shape.stroke.color,
            t::mix(app.p().line, tone.color, 0.02)
        );
        assert!(hit.contains_rect(card));
        let resting = app.audit.empty_decorations.clone();
        let resting_dots = app.audit.empty_dots.clone();
        assert_eq!(resting.len(), 3);
        assert_eq!(resting_dots.len(), 4);
        assert!(resting.iter().all(|rect| hit.contains_rect(*rect)));
        assert!(resting[2].left() - resting[0].right() >= 5.9);
        assert!(resting[1].left() - resting[2].right() >= 5.9);
        assert!(resting[0].center().y > resting[1].center().y);
        let angles = &app.audit.empty_tile_angles;
        assert_eq!(angles.len(), 3);
        assert!(angles[0] < 0.0 && angles[1] > 0.0 && angles[2] == 0.0);
        let cluster = app.audit.empty_cluster.unwrap();
        let title_slot = app.audit.empty_title_slot.unwrap();
        let hint_slot = app.audit.empty_hint_slot.unwrap();
        assert!((title_slot.top() - cluster.bottom() - 32.0).abs() < 0.01);
        assert!((hint_slot.top() - title_slot.bottom() - 16.0).abs() < 0.01);
        assert!((clear.top() - hint_slot.bottom() - 24.0).abs() < 0.01);
        assert_eq!(
            app.audit.empty_hint.as_deref(),
            Some(app.settings.language.tr(K::NoResultsHint))
        );
        assert!(!app.audit.empty_hint.as_deref().unwrap().contains("missing"));
        let title_visual = app.audit.empty_title_visual.unwrap();
        let hint_visual = app.audit.empty_hint_visual.unwrap();
        let hero_center = resting[2].center();
        let mut intermediate = None;
        for (index, time) in [tick + 0.2, tick + 0.32, tick + 0.5]
            .into_iter()
            .enumerate()
        {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
                    time: Some(time),
                    events: vec![egui::Event::PointerMoved(center)],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            if index == 1 {
                intermediate = Some(app.audit.empty_decorations.clone());
            }
        }
        let hovered = app.audit.empty_card.unwrap();
        let hovered_tiles = app.audit.empty_decorations.clone();
        let hovered_dots = app.audit.empty_dots.clone();
        let intermediate = intermediate.expect("intermediate illustration");
        assert!(hovered.width() > card.width());
        assert!(hovered.height() > card.height());
        assert_eq!(hovered.center(), center);
        assert!(hit.contains_rect(hovered));
        assert!(hovered_tiles[0].center().x < resting[0].center().x);
        assert!(hovered_tiles[1].center().x > resting[1].center().x);
        assert!(hovered_tiles[2].center().y < resting[2].center().y);
        assert!(hovered_tiles[2].width() > resting[2].width());
        assert!(intermediate[2].width() > resting[2].width());
        assert!(intermediate[2].width() < hovered_tiles[2].width());
        assert!(hovered_tiles.iter().all(|rect| hit.contains_rect(*rect)));
        for (before, after) in resting_dots.iter().zip(&hovered_dots) {
            assert!(
                (after.center() - hero_center).length() > (before.center() - hero_center).length()
            );
            assert!(after.width() > before.width());
            assert!(hit.contains_rect(*after));
        }
        assert!(app.audit.empty_title_visual.unwrap().top() < title_visual.top());
        assert!(app.audit.empty_hint_visual.unwrap().top() < hint_visual.top());
        assert_eq!(app.audit.empty_title_slot, Some(title_slot));
        assert_eq!(app.audit.empty_hint_slot, Some(hint_slot));
        let group_clear_visual = app.audit.empty_clear_visual.unwrap();
        assert!(group_clear_visual.width() > clear_visual.width());
        for time in [tick + 0.7, tick + 0.82, tick + 1.0] {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
                    time: Some(time),
                    events: vec![egui::Event::PointerMoved(clear.center())],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
        }
        let direct_clear_visual = app.audit.empty_clear_visual.unwrap();
        let direct_clear_icon = app.audit.empty_clear_icon.unwrap();
        assert!(direct_clear_visual.width() > group_clear_visual.width());
        assert!(direct_clear_icon.width() > clear_icon.width());
        assert!(hit.contains_rect(direct_clear_visual));
        assert_eq!(app.audit.empty_card_hit, Some(hit));
        assert_eq!(app.audit.empty_clear, Some(clear));
        assert_eq!(app.search, "missing");
        tick += 1.0;
        mouse_click(&ctx, &mut app, clear.center(), &mut tick);
        assert!(app.search.is_empty());
    }

    #[test]
    fn ordinary_mouse_hover_animates_the_large_card_after_reactive_idle() {
        let (ctx, mut app) = fixture();
        app.paths = Some(CavalryPaths::from_base(std::path::Path::new("synthetic")));
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let point = app.audit.import_hit.unwrap().center();
        let mut values = Vec::new();
        for time in [tick + 10.0, tick + 10.06, tick + 10.12, tick + 10.30] {
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    events: vec![egui::Event::PointerMoved(point)],
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            values.push(app.audit.import_hover);
        }
        assert!(values[0] < 0.1);
        assert!(values[1] > 0.0 && values[1] < 1.0);
        assert_eq!(values[3], 1.0);
    }
    #[test]
    fn whole_import_card_is_one_keyboard_target_and_unavailable_card_cannot_activate() {
        for available in [true, false] {
            let (ctx, mut app) = fixture();
            if available {
                app.paths = Some(CavalryPaths::from_base(std::path::Path::new("synthetic")));
            }
            let (_tx, rx) = std::sync::mpsc::channel();
            app.picker = Some(rx);
            let mut tick = 0.0;
            settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
            let id = egui::Id::new("import-card");
            ctx.memory_mut(|m| m.request_focus(id));
            let _ = render(&ctx, &mut app, [900.0, 620.0], tick + 1.0);
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(tick + 1.02),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
                    events: vec![egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: Some(egui::Key::Enter),
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            assert_eq!(app.audit.import_activated, available);
        }
    }
    #[test]
    fn direct_actions_and_decoration_stay_separate_in_all_languages() {
        for language in Language::ALL {
            let (ctx, mut app) = fixture();
            app.settings.language = language;
            app.library.scripts.push(crate::library::ScriptEntry {
                name: "Example.js".into(),
                search_key: "example.js".into(),
                path: "synthetic/Example.js".into(),
                group: None,
            });
            let mut tick = 0.0;
            settle(&ctx, &mut app, [720.0, 496.0], &mut tick);
            let (show, remove) = app.audit.row_actions[0];
            let dots = app.audit.row_decorations[0];
            for rect in [show, remove] {
                assert_eq!(rect.size(), egui::Vec2::splat(t::CONTROL));
                assert!(app.audit.library.unwrap().contains_rect(rect));
            }
            assert!(show.right() + t::MD <= remove.left());
            assert!(remove.right() + t::XS <= dots.left());
            assert_eq!(show.center().y, dots.center().y);
            assert_eq!(remove.center().y, dots.center().y);
        }
    }
    #[test]
    fn semantic_row_and_count_colors_follow_the_selected_kind_and_search_without_a_stale_frame() {
        let (ctx, mut app) = fixture();
        let mut tick = 0.0;
        app.library.scripts.push(crate::library::ScriptEntry {
            name: "Orbit.js".into(),
            group: None,
            path: "synthetic/Orbit.js".into(),
            search_key: "orbit.js".into(),
        });
        app.library.plugins.push(crate::library::PluginEntry {
            name: "Toolkit".into(),
            author: None,
            path: "synthetic/Toolkit".into(),
            search_key: "toolkit".into(),
        });
        for theme in [Theme::Cavalry, Theme::Light] {
            app.settings.theme = theme;
            for language in Language::ALL {
                app.settings.language = language;
                for (filter, n) in [(Filter::All, 2), (Filter::Scripts, 1), (Filter::Plugins, 1)] {
                    app.filter = filter;
                    app.search.clear();
                    settle(&ctx, &mut app, [720.0, 496.0], &mut tick);
                    let p = app.p();
                    let color = t::count_style(p, t::category(p, filter == Filter::Plugins)).1;
                    assert_eq!(app.audit.count_color, Some(color));
                    assert_eq!(
                        app.audit.count_label,
                        language.tr(K::Count).replace("{n}", &n.to_string())
                    );
                    let expected: Vec<_> = app
                        .rows
                        .iter()
                        .map(|row| t::category(p, matches!(row, Row::Plugin(_))).color)
                        .collect();
                    assert_eq!(app.audit.row_colors, expected);
                    assert_eq!(app.audit.row_actions.len(), n);
                    for (show, remove) in &app.audit.row_actions {
                        assert_eq!(show.size(), egui::Vec2::splat(t::CONTROL));
                        assert_eq!(remove.size(), show.size());
                    }
                    app.search = "no matching object".into();
                    tick += 1.0;
                    let _ = render(&ctx, &mut app, [720.0, 496.0], tick);
                    assert!(app.rows.is_empty());
                    assert_eq!(
                        app.audit.count_label,
                        language.tr(K::Count).replace("{n}", "0")
                    );
                }
            }
        }
    }
    #[test]
    fn import_hover_uses_a_single_even_outline_and_moves_both_text_lines_without_moving_targets() {
        let (ctx, mut app) = fixture();
        app.paths = Some(CavalryPaths::from_base(std::path::Path::new("synthetic")));
        let mut tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut tick);
        let area = app.audit.install.unwrap();
        let button = app.audit.import_button.unwrap();
        let title = app.audit.import_title.unwrap();
        let formats = app.audit.import_formats.unwrap();
        let idle = render(&ctx, &mut app, [900.0, 620.0], tick + 1.0);
        for shape in idle.shapes {
            if let egui::Shape::Rect(r) = shape.shape {
                if r.rect == area {
                    assert_eq!(r.stroke.width, 0.0);
                }
            }
        }
        let mut last = None;
        for frame in 0..32 {
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 620.0),
                    )),
                    time: Some(tick + 2.0 + frame as f64 / 60.0),
                    events: vec![egui::Event::PointerMoved(area.center())],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            last = Some(out);
        }
        assert_eq!(app.audit.install.unwrap(), area);
        assert_eq!(app.audit.import_button.unwrap(), button);
        assert!(app.audit.import_title.unwrap().top() < title.top());
        assert!(app.audit.import_formats.unwrap().top() < formats.top());
        let p = app.p();
        let tone = t::category(p, false);
        let mut outlines = 0;
        for shape in last.unwrap().shapes {
            if let egui::Shape::Rect(r) = shape.shape {
                if r.rect == area {
                    assert_eq!(r.stroke.width, 1.0);
                    assert_eq!(r.stroke.color, tone.color);
                    outlines += 1;
                }
                assert!(
                    !(area.contains_rect(r.rect)
                        && r.rect.width() <= 4.0
                        && r.rect.height() > area.height() / 2.0
                        && r.fill == tone.color),
                    "drop surface must not regain a thick left stripe"
                );
            }
        }
        assert_eq!(outlines, 1);
        assert!(!app.tr(K::ChooseFiles).contains('…'));
        assert!(!app.tr(K::ChooseFiles).contains("..."));
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
                settle(&ctx, &mut app, size, &mut tick);
                assert!(app.settings_draft.is_none());
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
        let mut close_tick = 0.0;
        settle(&ctx, &mut app, [900.0, 620.0], &mut close_tick);
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
            duplicates: vec![],
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
            duplicates: vec![],
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
        settle(&ctx, &mut app, [1120.0, 760.0], &mut tick);
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
