use super::{
    app::{CavalryKitApp, Filter},
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
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0_f32, 1.0_f32)),
            egui::Color32::WHITE,
        );
        let title = ui
            .painter()
            .layout_no_wrap("CavalryKit".into(), t::medium(19.0), p.text);
        let title_rect = Rect::from_min_size(
            Pos2::new(
                header.left() + 40.0,
                header.center().y - title.size().y / 2.0_f32,
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
        let status = Rect::from_center_size(
            Pos2::new(
                header.right() - (text.size().x + 32.0) / 2.0_f32,
                header.center().y,
            ),
            Vec2::new(text.size().x + 32.0, 28.0),
        );
        ui.painter().rect_filled(status, 14, p.surface);
        ui.painter().circle_filled(
            Pos2::new(status.left() + 12.0, status.center().y),
            3.0,
            if self.cavalry_running {
                p.success
            } else {
                p.muted
            },
        );
        ui.painter().galley(
            Pos2::new(
                status.left() + 24.0,
                status.center().y - text.size().y / 2.0_f32,
            ),
            text,
            p.muted,
        );
        let response = ui
            .interact(
                status,
                egui::Id::new("cavalry-status"),
                egui::Sense::hover(),
            )
            .on_hover_text(lang.tr(K::StatusHint));
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &label));
        self.audit.status = Some(status);
    }
    pub(crate) fn show_rail(&mut self, ui: &mut egui::Ui, height: f32) {
        let start = ui.cursor().top();
        let p = self.p();
        let lang = self.prefs().language;
        ui.spacing_mut().item_spacing.y = t::XS;
        ui.add_space(t::SM);
        ui.label(t::text(lang.tr(K::LibraryNav), 11.0, p.muted));
        ui.add_space(t::SM);
        for (filter, key, icon, count) in [
            (
                Filter::All,
                K::All,
                Icon::Library,
                self.library.scripts.len() + self.library.plugins.len(),
            ),
            (
                Filter::Scripts,
                K::Scripts,
                Icon::Script,
                self.library.scripts.len(),
            ),
            (
                Filter::Plugins,
                K::Plugins,
                Icon::Plugin,
                self.library.plugins.len(),
            ),
        ] {
            if t::nav_button(
                ui,
                lang.tr(key),
                icon,
                Some(self.filter == filter),
                Some(count),
                p,
            )
            .clicked()
            {
                self.filter = filter;
            }
        }
        // Keep secondary navigation near the bottom without assigning fixed-height text slots.
        let nav_height = if ui.ctx().screen_rect().height() < 600.0 {
            36.0
        } else {
            40.0
        };
        let footer_height = 4.0 * (nav_height + t::XS) + 20.0 + t::SM + 24.0;
        ui.add_space((height - (ui.cursor().top() - start) - footer_height).max(t::LG));
        ui.label(t::text(lang.tr(K::Shortcuts), 11.0, p.muted));
        ui.add_space(t::SM);
        for (key, icon) in [
            (K::ScriptsFolder, Icon::Folder),
            (K::PluginsFolder, Icon::Plugin),
            (K::About, Icon::Info),
            (K::Settings, Icon::Settings),
        ] {
            let r = t::nav_button(ui, lang.tr(key), icon, None, None, p);
            self.audit.navigation.push(r.rect);
            if r.clicked() {
                match key {
                    K::ScriptsFolder => {
                        if let Some(path) = &self.paths {
                            paths::open_dir(&path.scripts);
                        }
                    }
                    K::PluginsFolder => {
                        if let Some(path) = &self.paths {
                            paths::open_dir(&path.plugins);
                        }
                    }
                    K::About => self.about_open = true,
                    K::Settings => self.open_settings(),
                    _ => {}
                }
            }
        }
        ui.add_space(t::SM);
        ui.label(t::text(
            format!("CavalryKit  {}", env!("CARGO_PKG_VERSION")),
            11.0,
            p.muted,
        ));
    }
    pub(crate) fn show_install_stage(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        let dragging = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let compact = ui.ctx().screen_rect().height() < 600.0;
        let frame = egui::Frame::new()
            .fill(if dragging { p.selected } else { p.panel })
            .stroke(Stroke::new(
                1.0_f32,
                if dragging { p.focus } else { p.line },
            ))
            .corner_radius(8)
            .inner_margin(t::MD)
            .show(ui, |ui| {
                let button_width = ui
                    .painter()
                    .layout_no_wrap(lang.tr(K::ChooseFiles).into(), t::medium(14.0), p.on_accent)
                    .size()
                    .x
                    + 2.0_f32 * t::MD;
                let icon_width = 40.0;
                let text_width =
                    (ui.available_width() - icon_width - 2.0_f32 * t::MD - button_width).max(80.0);
                let title = ui.painter().layout(
                    lang.tr(K::DropTitle).into(),
                    t::medium(14.0),
                    p.text,
                    text_width,
                );
                let formats = ui.painter().layout_no_wrap(
                    "JS · JSX · JSC · ZIP · 7z".into(),
                    t::font(11.0),
                    p.muted,
                );
                let text_height = title.size().y
                    + if compact {
                        0.0
                    } else {
                        t::XS + formats.size().y
                    };
                let height = text_height.max(40.0);
                let (rect, r) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), height),
                    egui::Sense::hover(),
                );
                r.on_hover_text(lang.tr(K::EmptyHint));
                let tile = Rect::from_center_size(
                    Pos2::new(rect.left() + 20.0, rect.center().y),
                    Vec2::splat(40.0),
                );
                ui.painter().rect_filled(tile, 8, p.selected);
                icons::paint(ui.painter(), tile.shrink(10.0), Icon::Import, p.accent_text);
                let start = Pos2::new(
                    tile.right() + t::MD,
                    rect.center().y - text_height / 2.0_f32,
                );
                let title_height = title.size().y;
                ui.painter().galley(start, title, p.text);
                if !compact {
                    ui.painter().galley(
                        start + Vec2::new(0.0, title_height + t::XS),
                        formats,
                        p.muted,
                    );
                }
                let button_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.right() - button_width,
                        rect.center().y - t::CONTROL / 2.0_f32,
                    ),
                    Vec2::new(button_width, t::CONTROL),
                );
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(button_rect));
                if t::button(&mut child, lang.tr(K::ChooseFiles), t::Button::Primary, p).clicked() {
                    self.pick_files(ui.ctx());
                }
                ui.add_space(t::SM);
                let status = if let Some(key) = self.busy {
                    lang.tr(key).to_owned()
                } else if !self.conflicts.is_empty() {
                    lang.tr(K::ConflictsWaiting)
                        .replace("{n}", &self.conflicts.len().to_string())
                } else if !self.queue.is_empty() {
                    lang.tr(K::Queued)
                        .replace("{n}", &self.queue.len().to_string())
                } else {
                    lang.tr(K::DropHint).to_owned()
                };
                let label = ui.add(
                    egui::Label::new(t::text(
                        status,
                        12.0,
                        if self.busy.is_some() {
                            p.accent_text
                        } else {
                            p.muted
                        },
                    ))
                    .wrap(),
                );
                self.audit.install_hint = Some(label.rect);
            });
        self.audit.install = Some(frame.response.rect);
    }
}
