use super::{
    app::CavalryKitApp,
    icons::{self, Icon},
    theme as t,
};
use crate::{i18n::Key as K, paths};
use egui::{Pos2, Rect, Stroke, Vec2};
impl CavalryKitApp {
    pub(crate) fn show_brand_header(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        let (header, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), egui::Sense::hover());
        ui.painter().image(
            self.logo.id(),
            Rect::from_min_size(header.min, Vec2::splat(32.0)),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        let title = ui
            .painter()
            .layout_no_wrap("CavalryKit".into(), t::medium(19.0), p.text);
        let title_rect = Rect::from_min_size(
            Pos2::new(
                header.left() + 40.0,
                header.center().y - title.size().y / 2.0,
            ),
            title.size(),
        );
        ui.painter().galley(title_rect.min, title, p.text);
        self.audit.title = Some(title_rect);
        let label = format!(
            "Cavalry {}",
            lang.tr(if self.cavalry_running {
                K::Running
            } else {
                K::Stopped
            })
        );
        let text = ui
            .painter()
            .layout_no_wrap(label.clone(), t::font(12.0), p.muted);
        let status = Rect::from_min_size(
            Pos2::new(
                header.right() - text.size().x - 16.0,
                header.center().y - 14.0,
            ),
            Vec2::new(text.size().x + 16.0, 28.0),
        );
        ui.painter().circle_filled(
            Pos2::new(status.left() + 3.0, status.center().y),
            3.0,
            if self.cavalry_running {
                p.success
            } else {
                p.muted
            },
        );
        ui.painter().galley(
            Pos2::new(
                status.left() + 16.0,
                status.center().y - text.size().y / 2.0,
            ),
            text,
            p.muted,
        );
        let r = ui
            .interact(
                status,
                egui::Id::new("cavalry-status"),
                egui::Sense::hover(),
            )
            .on_hover_text(lang.tr(K::StatusHint));
        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &label));
        self.audit.status = Some(status);
    }
    pub(crate) fn show_rail(&mut self, ui: &mut egui::Ui, height: f32) {
        let p = self.p();
        let lang = self.prefs().language;
        ui.spacing_mut().item_spacing.y = 0.0;
        let footer_height = 3.0 * t::CONTROL + 2.0 * t::SM;
        let drop_height = (height - footer_height - t::LG).max(216.0);
        self.show_install_stage(ui, drop_height);
        ui.add_space(t::LG);
        for (key, icon) in [
            (K::ScriptsFolder, Icon::Folder),
            (K::PluginsFolder, Icon::Plugin),
        ] {
            let r = ui
                .add_enabled_ui(self.paths.is_some(), |ui| {
                    t::nav_button(ui, lang.tr(key), icon, None, None, p)
                })
                .inner;
            self.audit.navigation.push(r.rect);
            if r.clicked() {
                if let Some(path) = &self.paths {
                    paths::open_dir(if key == K::ScriptsFolder {
                        &path.scripts
                    } else {
                        &path.plugins
                    });
                }
            }
            ui.add_space(t::SM);
        }
        let (footer, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), t::CONTROL),
            egui::Sense::hover(),
        );
        let width = (footer.width() - t::SM) / 2.0;
        for (n, key, icon) in [(0, K::About, Icon::Info), (1, K::Settings, Icon::Settings)] {
            let rect = Rect::from_min_size(
                footer.min + Vec2::new(n as f32 * (width + t::SM), 0.0),
                Vec2::new(width, t::CONTROL),
            );
            let mut child = ui.new_child(egui::UiBuilder::new().id_salt(key as u32).max_rect(rect));
            let r = t::nav_button(&mut child, lang.tr(key), icon, None, None, p);
            self.audit.navigation.push(r.rect);
            if r.clicked() {
                if key == K::About {
                    self.about_open = true
                } else {
                    self.open_settings();
                }
            }
        }
    }
    pub(crate) fn show_install_stage(&mut self, ui: &mut egui::Ui, height: f32) {
        let p = self.p();
        let lang = self.prefs().language;
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );
        self.audit.install = Some(rect);
        let dragging = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let drag = t::motion(ui, egui::Id::new("import-drag"), dragging);
        let fill = t::mix(p.panel, p.selected, drag);
        let border = t::mix(p.line, p.focus, drag);
        ui.painter().rect(
            rect,
            t::CARD_RADIUS,
            fill,
            Stroke::new(1.0 + drag, border),
            egui::StrokeKind::Inside,
        );
        let inner = rect.shrink(t::MD);
        let status = if let Some(key) = self.busy {
            lang.tr(key).to_owned()
        } else if !self.conflicts.is_empty() {
            lang.tr(K::ConflictsWaiting)
                .replace("{n}", &self.conflicts.len().to_string())
        } else if !self.queue.is_empty() {
            lang.tr(K::Queued)
                .replace("{n}", &self.queue.len().to_string())
        } else if self.paths.is_none() {
            lang.tr(K::DirectoriesUnavailable).to_owned()
        } else {
            lang.tr(K::DropHint).to_owned()
        };
        let hint_color = if self.busy.is_some() {
            p.accent_text
        } else {
            p.muted
        };
        let mut hint_job =
            egui::text::LayoutJob::simple(status.clone(), t::font(11.0), hint_color, inner.width());
        hint_job.halign = egui::Align::Center;
        let hint = ui.painter().layout_job(hint_job);
        let hint_rect = Rect::from_min_size(
            Pos2::new(inner.left(), inner.bottom() - hint.size().y),
            Vec2::new(inner.width(), hint.size().y),
        );
        ui.painter().galley(
            Pos2::new(hint_rect.center().x, hint_rect.top()),
            hint,
            hint_color,
        );
        ui.interact(
            hint_rect,
            egui::Id::new("import-status"),
            egui::Sense::hover(),
        )
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &status));
        self.audit.install_hint = Some(hint_rect);
        let button_rect = Rect::from_min_size(
            Pos2::new(inner.left(), hint_rect.top() - t::SM - t::CONTROL),
            Vec2::new(inner.width(), t::CONTROL),
        );
        let mut button_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("import-picker")
                .max_rect(button_rect)
                .layout(egui::Layout::top_down(egui::Align::Center)),
        );
        let r = button_ui
            .add_enabled_ui(self.paths.is_some(), |ui| {
                t::button(ui, lang.tr(K::ChooseFiles), t::Button::Primary, p)
            })
            .inner;
        self.audit.import_button = Some(r.rect);
        if r.clicked() {
            self.pick_files(ui.ctx());
        }
        // Center only the decorative content, never the click target or text slots.
        let content = Rect::from_min_max(
            inner.min,
            Pos2::new(inner.right(), button_rect.top() - t::MD),
        );
        let compact = height < 300.0;
        let tile_size = if compact { 32.0 } else { 56.0 };
        let mut title_job = egui::text::LayoutJob::simple(
            lang.tr(K::DropTitle).into(),
            t::medium(14.0),
            p.text,
            inner.width(),
        );
        title_job.halign = egui::Align::Center;
        let title = ui.painter().layout_job(title_job);
        let formats =
            ui.painter()
                .layout_no_wrap("JS · JSX · JSC · ZIP · 7z".into(), t::font(11.0), p.muted);
        let stack_h = tile_size + t::MD + title.size().y + t::SM + formats.size().y;
        let y = content.top() + ((content.height() - stack_h) / 2.0).max(0.0);
        let tile = Rect::from_center_size(
            Pos2::new(content.center().x, y + tile_size / 2.0 - 2.0 * drag),
            Vec2::splat(tile_size),
        );
        ui.painter().rect_filled(tile, t::CARD_RADIUS, p.surface);
        icons::paint(
            ui.painter(),
            Rect::from_center_size(
                tile.center(),
                Vec2::splat(if compact { 20.0 } else { 28.0 }),
            ),
            Icon::Import,
            p.accent_text,
        );
        let title_y = y + tile_size + t::MD;
        let title_h = title.size().y;
        self.audit.import_art = Some(Rect::from_min_max(
            Pos2::new(inner.left(), tile.top()),
            Pos2::new(inner.right(), title_y + title_h + t::SM + formats.size().y),
        ));
        ui.painter()
            .galley(Pos2::new(content.center().x, title_y), title, p.text);
        ui.painter().galley(
            Pos2::new(
                content.center().x - formats.size().x / 2.0,
                title_y + title_h + t::SM,
            ),
            formats,
            p.muted,
        );
    }
}
