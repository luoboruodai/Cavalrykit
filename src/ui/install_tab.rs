use super::{
    app::CavalryKitApp,
    icons::{self, Icon},
    theme as t,
};
use crate::{i18n::Key as K, paths};
use egui::{Pos2, Rect, Stroke, Vec2};
impl CavalryKitApp {
    /// Identity belongs to the left rail, not a second full-width title bar.
    pub(crate) fn show_brand_header(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), t::HEADER),
            egui::Sense::hover(),
        );
        let logo = Rect::from_center_size(
            Pos2::new(rect.left() + 14.0, rect.center().y),
            Vec2::splat(28.0),
        );
        ui.painter().image(
            self.logo.id(),
            logo,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        let status_label = lang.tr(if self.cavalry_running {
            K::Running
        } else {
            K::Stopped
        });
        let text = ui
            .painter()
            .layout_no_wrap(status_label.into(), t::font(11.0), p.muted);
        let status = Rect::from_center_size(
            Pos2::new(
                rect.right() - (t::ink_bounds(&text).width() + 30.0) / 2.0,
                rect.center().y,
            ),
            Vec2::new(t::ink_bounds(&text).width() + 30.0, 28.0),
        );
        ui.painter().rect(
            status,
            14,
            p.surface,
            Stroke::new(1.0_f32, p.line),
            egui::StrokeKind::Inside,
        );
        let pulse = super::motion::status_breath(ui.ctx(), self.cavalry_running, ui.is_enabled());
        self.audit.status_pulse = pulse;
        let dot = Pos2::new(status.left() + 11.0, status.center().y);
        let color = if self.cavalry_running {
            t::category(p, false).color
        } else {
            p.muted
        };
        if self.cavalry_running {
            ui.painter().circle_filled(
                dot,
                4.0 + 0.8 * pulse,
                color.gamma_multiply(0.05 + 0.11 * pulse),
            );
        }
        ui.painter().circle_filled(
            dot,
            2.5,
            color.gamma_multiply(if self.cavalry_running {
                0.65 + 0.35 * pulse
            } else {
                1.0
            }),
        );
        let text_pos = t::left_ink(status.left() + 20.0, status.center().y, &text);
        self.audit.status_text = Some(t::ink_bounds(&text).translate(text_pos.to_vec2()));
        ui.painter().galley(text_pos, text, p.muted);
        let r = ui
            .interact(
                status,
                egui::Id::new("cavalry-status"),
                egui::Sense::hover(),
            )
            .on_hover_text(lang.tr(K::StatusHint));
        r.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Label,
                true,
                format!("Cavalry {status_label}"),
            )
        });
        self.audit.status = Some(status);
        let title = t::label_truncated(
            ui,
            "CavalryKit",
            t::medium(16.0),
            p.text,
            (status.left() - logo.right() - 2.0 * t::SM).max(40.0),
        );
        let title_rect = Rect::from_min_size(
            Pos2::new(logo.right() + t::SM, rect.center().y - title.size().y / 2.0),
            title.size(),
        );
        ui.painter().galley(title_rect.min, title, p.text);
        self.audit.title = Some(title_rect);
    }
    pub(crate) fn show_rail(&mut self, ui: &mut egui::Ui, _height: f32) {
        let p = self.p();
        let lang = self.prefs().language;
        let tone = t::category(p, false);
        let compact = ui.ctx().screen_rect().height() < 600.0;
        ui.spacing_mut().item_spacing.y = 0.0;
        self.show_brand_header(ui);
        ui.add_space(if compact { t::SM } else { t::MD });
        let hint = self.import_status();
        let hint_color = if self.busy.is_some() {
            tone.color
        } else {
            t::helper_ink(p)
        };
        let mut job = egui::text::LayoutJob::simple(
            hint.clone(),
            t::font(11.0),
            hint_color,
            ui.available_width(),
        );
        job.halign = egui::Align::Center;
        let helper = ui.painter().layout_job(job);
        let nav_gap = if compact { t::XS } else { t::SM };
        let section_gap = if compact { t::MD } else { t::LG };
        let tail = t::SM
            + t::SEARCH_HEIGHT
            + t::SM
            + helper.size().y
            + section_gap
            + 4.0 * t::CONTROL
            + 3.0 * nav_gap;
        let drop_height = (ui.available_height() - tail).max(112.0);
        self.show_install_stage(ui, drop_height);
        ui.add_space(t::SM);
        let action = ui
            .add_enabled_ui(self.paths.is_some(), |ui| {
                t::import_action(ui, lang.tr(K::ChooseFiles), tone, p)
            })
            .inner;
        self.audit.import_button = Some(action.rect);
        if action.clicked() {
            self.audit.import_activated = true;
            self.pick_files(ui.ctx());
        }
        ui.add_space(t::SM);
        let (hint_rect, r) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), helper.size().y),
            egui::Sense::hover(),
        );
        ui.painter().galley(
            Pos2::new(hint_rect.center().x, hint_rect.top()),
            helper,
            hint_color,
        );
        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &hint));
        self.audit.install_hint = Some(hint_rect);
        ui.add_space(section_gap);
        for (index, key, icon, plugin) in [
            (0, K::ScriptsFolder, Icon::Folder, false),
            (1, K::PluginsFolder, Icon::Plugin, false),
            (2, K::About, Icon::Info, false),
            (3, K::Settings, Icon::Settings, false),
        ] {
            let available = index >= 2 || self.paths.is_some();
            let r = ui
                .add_enabled_ui(available, |ui| {
                    t::shortcut(ui, lang.tr(key), icon, t::category(p, plugin), p)
                })
                .inner;
            self.audit.navigation.push(r.rect);
            self.audit
                .navigation_colors
                .push(t::category(p, false).color);
            if r.clicked() {
                match key {
                    K::ScriptsFolder => {
                        if let Some(path) = &self.paths {
                            paths::open_dir(&path.scripts)
                        }
                    }
                    K::PluginsFolder => {
                        if let Some(path) = &self.paths {
                            paths::open_dir(&path.plugins)
                        }
                    }
                    K::About => self.about_open = true,
                    K::Settings => self.open_settings(),
                    _ => {}
                }
            }
            if index < 3 {
                ui.add_space(nav_gap);
            }
        }
    }
    fn import_status(&self) -> String {
        let lang = self.prefs().language;
        if let Some(key) = self.busy {
            lang.tr(key).into()
        } else if !self.conflicts.is_empty() {
            lang.tr(K::ConflictsWaiting)
                .replace("{n}", &self.conflicts.len().to_string())
        } else if !self.queue.is_empty() {
            lang.tr(K::Queued)
                .replace("{n}", &self.queue.len().to_string())
        } else if self.paths.is_none() {
            lang.tr(K::DirectoriesUnavailable).into()
        } else {
            lang.tr(K::DropHint).into()
        }
    }
    pub(crate) fn show_install_stage(&mut self, ui: &mut egui::Ui, height: f32) {
        let p = self.p();
        let lang = self.prefs().language;
        let tone = t::category(p, false);
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );
        let available = self.paths.is_some();
        let r = ui.interact(
            rect,
            egui::Id::new("import-card"),
            if available {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );
        r.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                ui.is_enabled() && available,
                lang.tr(K::DropTitle),
            )
        });
        let hover = t::motion(
            ui,
            r.id.with("hover"),
            ui.is_enabled() && available && (r.hovered() || r.has_focus()),
        );
        let dragging = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let drag = super::motion::travel(
            ui.ctx(),
            r.id.with("file-drag"),
            if dragging { 1.0 } else { 0.0 },
            0.32,
        );
        let active = hover.max(drag);
        self.audit.install = Some(rect);
        self.audit.import_hit = Some(r.rect);
        self.audit.import_hover = hover;
        super::diagnostics::sample(
            ui.ctx(),
            "import-hover",
            r.id,
            if r.hovered() || r.has_focus() {
                1.0
            } else {
                0.0
            },
            hover,
        );
        let resting = t::drop_surface(p);
        ui.painter().rect(
            rect,
            12,
            t::mix(resting, p.surface, active * 0.55),
            Stroke::new(active, tone.color),
            egui::StrokeKind::Inside,
        );
        // Four edges are the same stroke width. A separate keyboard-only inset ring
        // preserves focus visibility without reintroducing a thick left accent rail.
        if r.has_focus() {
            ui.painter().rect_stroke(
                rect.shrink(3.0),
                10,
                Stroke::new(1.0_f32, tone.color),
                egui::StrokeKind::Inside,
            );
        }
        let inner = rect.shrink(t::MD);
        let compact = height < 180.0;
        let tile_size = if compact { 36.0 } else { 56.0 };
        let text_gap = if compact { t::SM } else { t::MD };
        let mut job = egui::text::LayoutJob::simple(
            lang.tr(K::DropTitle).into(),
            t::medium(14.0),
            p.text,
            inner.width(),
        );
        job.halign = egui::Align::Center;
        let title = ui.painter().layout_job(job);
        let formats =
            ui.painter()
                .layout_no_wrap("JS · JSX · JSC · ZIP · 7z".into(), t::font(11.0), p.muted);
        let stack = tile_size + text_gap + title.size().y + t::XS + formats.size().y;
        let start = inner.top() + ((inner.height() - stack) / 2.0).max(0.0);
        let spring = super::motion::settle(active);
        let tile = Rect::from_center_size(
            Pos2::new(inner.center().x, start + tile_size / 2.0 - 3.0 * spring),
            Vec2::splat(tile_size),
        );
        ui.painter()
            .rect_filled(tile, 10, t::mix(p.surface, tone.tint, active));
        let icon_rect = tile.shrink(if compact { 6.0 } else { 12.0 });
        self.audit.import_icon = Some(icon_rect);
        icons::paint(ui.painter(), icon_rect, Icon::Import, tone.color);
        let title_y = start + tile_size + text_gap - 2.0 * active;
        let format_y = title_y + title.size().y + t::XS;
        let title_rect = Rect::from_min_size(
            Pos2::new(inner.left(), title_y),
            Vec2::new(inner.width(), title.size().y),
        );
        let format_rect = Rect::from_min_size(
            Pos2::new(inner.center().x - formats.size().x / 2.0, format_y),
            formats.size(),
        );
        ui.painter().galley_with_override_text_color(
            Pos2::new(inner.center().x, title_y),
            title,
            t::mix(p.text, tone.color, active * 0.16),
        );
        ui.painter().galley_with_override_text_color(
            format_rect.min,
            formats,
            t::mix(p.muted, p.text, active * 0.55),
        );
        self.audit.import_title = Some(title_rect);
        self.audit.import_formats = Some(format_rect);
        self.audit.import_art = Some(tile.union(title_rect).union(format_rect));
        if r.clicked() && available {
            self.audit.import_activated = true;
            self.pick_files(ui.ctx());
        }
        if available {
            r.on_hover_cursor(egui::CursorIcon::PointingHand);
        }
    }
}
