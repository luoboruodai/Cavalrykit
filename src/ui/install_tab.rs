use super::{app::CavalryKitApp, theme as t};
use crate::{i18n::Key as K, paths};
use egui::{Pos2, Stroke, Vec2};
impl CavalryKitApp {
    pub(crate) fn show_rail(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        let (header, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), egui::Sense::hover());
        let logo_rect = egui::Rect::from_center_size(
            Pos2::new(header.left() + 16.0, header.center().y),
            Vec2::splat(32.0),
        );
        ui.painter().image(
            self.logo.id(),
            logo_rect,
            egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        let title = ui
            .painter()
            .layout_no_wrap("CavalryKit".into(), t::medium(18.0), p.text);
        let title_rect = egui::Rect::from_min_size(
            Pos2::new(
                header.left() + 40.0,
                header.center().y - title.size().y / 2.0,
            ),
            title.size(),
        );
        ui.painter().galley(title_rect.min, title, p.text);
        self.audit.title = Some(title_rect);
        let label = lang.tr(if self.cavalry_running {
            K::Running
        } else {
            K::Stopped
        });
        let text = ui
            .painter()
            .layout_no_wrap(label.into(), t::font(12.0), p.muted);
        let status = egui::Rect::from_center_size(
            Pos2::new(
                header.right() - (text.size().x + 28.0) / 2.0,
                header.center().y,
            ),
            Vec2::new(text.size().x + 28.0, 24.0),
        );
        ui.painter().rect(
            status,
            6,
            p.surface,
            Stroke::new(1.0_f32, p.border),
            egui::StrokeKind::Inside,
        );
        ui.painter().circle_filled(
            Pos2::new(status.left() + 10.0, status.center().y),
            3.0,
            if self.cavalry_running {
                p.success
            } else {
                p.muted
            },
        );
        ui.painter().galley(
            Pos2::new(
                status.left() + 20.0,
                status.center().y - text.size().y / 2.0,
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
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, true, format!("Cavalry {label}"))
        });
        self.audit.status = Some(status);
        ui.add_space(t::MD);
        ui.label(t::heading(lang.tr(K::InstallTitle), 14.0, p));
        ui.add_space(t::SM);
        self.show_install_stage(ui);
        ui.add_space(t::MD);
        ui.separator();
        ui.add_space(t::SM);
        for key in [K::ScriptsFolder, K::PluginsFolder, K::About, K::Settings] {
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), egui::Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), lang.tr(key))
            });
            if response.hovered() || response.has_focus() {
                ui.painter().rect_filled(rect, 6, p.raised);
            }
            if response.has_focus() {
                ui.painter().rect_stroke(
                    rect,
                    6,
                    Stroke::new(1.0_f32, p.focus),
                    egui::StrokeKind::Inside,
                );
            }
            nav_icon(
                ui.painter(),
                Pos2::new(rect.left() + 16.0, rect.center().y),
                key,
                p.muted,
            );
            let text = ui
                .painter()
                .layout_no_wrap(lang.tr(key).into(), t::font(14.0), p.text);
            ui.painter().galley(
                Pos2::new(rect.left() + 32.0, rect.center().y - text.size().y / 2.0),
                text,
                p.text,
            );
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
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
    }
    fn show_install_stage(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        let dragging = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let height = (ui.available_height() - 248.0).clamp(144.0, 320.0);
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );
        let fill = if dragging {
            t::mix(p.surface, p.accent, 0.10)
        } else {
            p.surface
        };
        ui.painter().rect_filled(rect, 8, fill);
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            8,
            Stroke::new(
                if dragging { 2.0_f32 } else { 1.0_f32 },
                if dragging { p.focus } else { p.border },
            ),
            egui::StrokeKind::Inside,
        );
        let cx = rect.center().x;
        let top = rect.center().y - 46.0;
        let color = p.accent_text;
        for (a, b) in [
            (Pos2::new(cx, top), Pos2::new(cx, top + 22.0)),
            (Pos2::new(cx - 7.0, top + 15.0), Pos2::new(cx, top + 22.0)),
            (Pos2::new(cx + 7.0, top + 15.0), Pos2::new(cx, top + 22.0)),
            (
                Pos2::new(cx - 12.0, top + 32.0),
                Pos2::new(cx + 12.0, top + 32.0),
            ),
            (
                Pos2::new(cx - 12.0, top + 25.0),
                Pos2::new(cx - 12.0, top + 32.0),
            ),
            (
                Pos2::new(cx + 12.0, top + 25.0),
                Pos2::new(cx + 12.0, top + 32.0),
            ),
        ] {
            ui.painter()
                .line_segment([a, b], Stroke::new(2.0_f32, color));
        }
        for (text, y, font, color) in [
            (lang.tr(K::DropTitle), top + 48.0, t::medium(15.0), p.text),
            (
                ".js / .jsx / .jsc / .zip / .7z",
                top + 72.0,
                t::font(12.0),
                p.muted,
            ),
        ] {
            let g = t::label_truncated(ui, text, font, color, rect.width() - 16.0);
            ui.painter()
                .galley(Pos2::new(cx - g.size().x / 2.0, y), g, color);
        }
        response.on_hover_text(lang.tr(K::DropHint));
        ui.add_space(t::SM);
        let button = ui.add_sized(
            [ui.available_width(), 40.0],
            egui::Button::new(t::text(lang.tr(K::ChooseFiles), 14.0, p.on_accent))
                .fill(p.accent)
                .stroke(Stroke::new(1.0_f32, p.focus))
                .corner_radius(6),
        );
        if button.clicked() {
            self.pick_files(ui.ctx());
        }
        let status = if let Some(key) = self.busy {
            Some(lang.tr(key).to_owned())
        } else if !self.conflicts.is_empty() {
            Some(
                lang.tr(K::ConflictsWaiting)
                    .replace("{n}", &self.conflicts.len().to_string()),
            )
        } else if !self.queue.is_empty() {
            Some(
                lang.tr(K::Queued)
                    .replace("{n}", &self.queue.len().to_string()),
            )
        } else {
            None
        };
        // Always reserve one status slot: progress must not move the navigation controls.
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 32.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                if let Some(text) = status {
                    ui.add(egui::Label::new(t::text(text, 12.0, p.accent_text)).wrap());
                } else {
                    ui.add(egui::Label::new(t::text(lang.tr(K::DropHint), 12.0, p.muted)).wrap());
                }
            },
        );
    }
}

fn nav_icon(p: &egui::Painter, c: Pos2, key: K, color: egui::Color32) {
    let stroke = Stroke::new(1.5_f32, color);
    match key {
        K::ScriptsFolder => {
            p.rect_stroke(
                egui::Rect::from_center_size(c + Vec2::new(0.0, 1.0), Vec2::new(14.0, 10.0)),
                2,
                stroke,
                egui::StrokeKind::Inside,
            );
            p.line_segment(
                [c + Vec2::new(-6.0, -5.0), c + Vec2::new(-1.0, -5.0)],
                stroke,
            );
        }
        K::PluginsFolder => {
            p.rect_stroke(
                egui::Rect::from_center_size(c, Vec2::splat(11.0)),
                2,
                stroke,
                egui::StrokeKind::Inside,
            );
            p.circle_stroke(c + Vec2::new(1.0, -6.0), 2.0, stroke);
            p.circle_stroke(c + Vec2::new(6.0, 1.0), 2.0, stroke);
        }
        K::About => {
            p.circle_stroke(c, 7.0, stroke);
            p.circle_filled(c + Vec2::new(0.0, -3.0), 1.0, color);
            p.line_segment([c + Vec2::new(0.0, 0.0), c + Vec2::new(0.0, 4.0)], stroke);
        }
        K::Settings => {
            p.circle_stroke(c, 5.5, stroke);
            p.circle_stroke(c, 2.0, stroke);
            for n in 0..8 {
                let angle = n as f32 * std::f32::consts::TAU / 8.0;
                let v = Vec2::angled(angle);
                p.line_segment([c + v * 5.5, c + v * 8.0], stroke);
            }
        }
        _ => {}
    }
}
