use super::{
    app::{CavalryKitApp, Row},
    icons::{self, Icon},
    theme as t,
};
use crate::{i18n::Key as K, paths};
use egui::{Pos2, Rect, Stroke, Vec2};
impl CavalryKitApp {
    pub(crate) fn show_library(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.horizontal(|ui| {
            ui.label(t::heading(lang.tr(K::Library), 24.0, p));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(self.busy.is_none() && self.conflicts.is_empty(), |ui| {
                    if t::icon_button(ui, lang.tr(K::Refresh), Icon::Refresh, false, p).clicked() {
                        self.refresh_library();
                    }
                });
            });
        });
        ui.add_space(t::XS);
        ui.label(t::text(lang.tr(K::LibraryHint), 12.0, p.muted));
        ui.add_space(t::MD);
        self.show_install_stage(ui);
        ui.add_space(t::MD);
        self.search_box(ui);
        ui.add_space(t::MD);
        self.update_rows();
        let hint = if let Some(status) = self.busy {
            lang.tr(status)
        } else if self.paths.is_none() {
            lang.tr(K::DirectoriesUnavailable)
        } else {
            lang.tr(K::IdleHint)
        };
        let footer = ui
            .painter()
            .layout(hint.into(), t::font(11.0), p.muted, ui.available_width());
        let inner_height =
            (ui.available_height() - footer.size().y - t::MD - 2.0_f32 * t::MD).max(48.0);
        let card = egui::Frame::new()
            .fill(p.panel)
            .corner_radius(8)
            .inner_margin(t::MD)
            .show(ui, |ui| {
                ui.set_min_height(inner_height);
                let (header, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 24.0),
                    egui::Sense::hover(),
                );
                let label = format!(
                    "{}  ·  {}",
                    lang.tr(K::NameColumn),
                    lang.tr(K::Count)
                        .replace("{n}", &self.rows.len().to_string())
                );
                ui.painter().text(
                    header.left_top(),
                    egui::Align2::LEFT_TOP,
                    label,
                    t::font(11.0),
                    p.muted,
                );
                ui.painter().text(
                    header.right_top() - Vec2::new(8.0, 0.0),
                    egui::Align2::RIGHT_TOP,
                    lang.tr(K::ActionsColumn),
                    t::font(11.0),
                    p.muted,
                );
                ui.painter().hline(
                    header.x_range(),
                    header.bottom() - 1.0_f32,
                    Stroke::new(1.0_f32, p.line),
                );
                let height = (inner_height - 24.0).max(24.0);
                if self.rows.is_empty() {
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), height),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing.y = t::SM;
                            ui.add_space((height / 2.0_f32 - 48.0).max(0.0));
                            let searching = !self.search.trim().is_empty();
                            ui.label(t::heading(
                                lang.tr(if searching {
                                    K::NoResults
                                } else if self.busy.is_some() {
                                    K::ScanBusy
                                } else {
                                    K::EmptyLibrary
                                }),
                                16.0,
                                p,
                            ));
                            // Shortest supported windows keep the actionable empty state visible.
                            if height > 96.0 {
                                ui.add(
                                    egui::Label::new(t::text(
                                        lang.tr(if searching {
                                            K::NoResultsHint
                                        } else {
                                            K::EmptyHint
                                        }),
                                        12.0,
                                        p.muted,
                                    ))
                                    .wrap(),
                                );
                            }
                            if searching
                                && height > 80.0
                                && t::button(ui, lang.tr(K::ClearSearch), t::Button::Secondary, p)
                                    .clicked()
                            {
                                self.search.clear();
                            }
                        },
                    );
                } else {
                    ui.scope(|ui| {
                        // Keep the foreground paired with theme::scrollbar_opacity. Dimming
                        // it locally also dims egui's thumb below its contrast target.
                        egui::ScrollArea::vertical()
                            .id_salt("library-list")
                            .auto_shrink([false, false])
                            .max_height(height)
                            .show_rows(ui, t::ROW, self.rows.len(), |ui, range| {
                                for index in range {
                                    let (name, kind, meta, path) = match self.rows[index] {
                                        Row::Script(i) => {
                                            let e = &self.library.scripts[i];
                                            (&e.name, K::Scripts, e.group.as_deref(), &e.path)
                                        }
                                        Row::Plugin(i) => {
                                            let e = &self.library.plugins[i];
                                            (&e.name, K::Plugins, e.author.as_deref(), &e.path)
                                        }
                                    };
                                    let path_key = path.clone();
                                    ui.push_id((kind as u32, &path_key), |ui| {
                                        ui.add_enabled_ui(self.busy.is_none(), |ui| {
                                            let (rect, response) = ui.allocate_exact_size(
                                                Vec2::new(ui.available_width(), t::ROW),
                                                egui::Sense::hover(),
                                            );
                                            if response.hovered() {
                                                ui.painter().rect_filled(rect, 6, p.surface);
                                            }
                                            let tile = Rect::from_center_size(
                                                Pos2::new(rect.left() + 16.0, rect.center().y),
                                                Vec2::splat(32.0),
                                            );
                                            ui.painter().rect_filled(tile, 6, p.selected);
                                            icons::paint(
                                                ui.painter(),
                                                tile.shrink(8.0),
                                                if kind == K::Scripts {
                                                    Icon::Script
                                                } else {
                                                    Icon::Plugin
                                                },
                                                if kind == K::Plugins {
                                                    p.accent_text
                                                } else {
                                                    p.muted
                                                },
                                            );
                                            let reveal_width = (ui
                                                .painter()
                                                .layout_no_wrap(
                                                    lang.tr(K::Reveal).into(),
                                                    t::medium(14.0),
                                                    p.text,
                                                )
                                                .size()
                                                .x
                                                + 32.0)
                                                .max(64.0);
                                            let actions = reveal_width + t::CONTROL + t::SM;
                                            let x = rect.left() + 48.0;
                                            let width =
                                                (rect.width() - 48.0 - actions - t::SM).max(16.0);
                                            let name_g = t::label_truncated(
                                                ui,
                                                name,
                                                t::font(14.0),
                                                p.text,
                                                width,
                                            );
                                            let subtitle = if let Some(value) = meta {
                                                format!("{} · {value}", lang.tr(kind))
                                            } else {
                                                lang.tr(kind).into()
                                            };
                                            let sub_g = t::label_truncated(
                                                ui,
                                                &subtitle,
                                                t::font(12.0),
                                                p.muted,
                                                width,
                                            );
                                            let text_height =
                                                name_g.size().y + t::XS + sub_g.size().y;
                                            let name_y = rect.center().y - text_height / 2.0_f32;
                                            let sub_y = name_y + name_g.size().y + t::XS;
                                            ui.painter().galley(
                                                Pos2::new(x, name_y),
                                                name_g,
                                                p.text,
                                            );
                                            ui.painter().galley(
                                                Pos2::new(x, sub_y),
                                                sub_g,
                                                p.muted,
                                            );
                                            ui.painter().hline(
                                                x..=rect.right(),
                                                rect.bottom() - 1.0_f32,
                                                Stroke::new(1.0_f32, p.line),
                                            );
                                            response.on_hover_text(format!("{name}\n{subtitle}"));
                                            let action_rect = Rect::from_min_max(
                                                Pos2::new(
                                                    rect.right() - actions,
                                                    rect.center().y - t::CONTROL / 2.0_f32,
                                                ),
                                                Pos2::new(
                                                    rect.right(),
                                                    rect.center().y + t::CONTROL / 2.0_f32,
                                                ),
                                            );
                                            let mut child = ui.new_child(
                                                egui::UiBuilder::new()
                                                    .max_rect(action_rect)
                                                    .layout(egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    )),
                                            );
                                            if t::icon_button(
                                                &mut child,
                                                &format!("{} · {name}", lang.tr(K::Uninstall)),
                                                Icon::Trash,
                                                true,
                                                p,
                                            )
                                            .clicked()
                                            {
                                                self.confirm_uninstall = Some(path.clone());
                                            }
                                            if t::button(
                                                &mut child,
                                                lang.tr(K::Reveal),
                                                t::Button::Quiet,
                                                p,
                                            )
                                            .on_hover_text(lang.tr(K::RevealHint))
                                            .clicked()
                                            {
                                                paths::reveal_in_file_manager(path);
                                            }
                                        });
                                    });
                                }
                            });
                    });
                }
            });
        self.audit.card = Some(card.response.rect);
        ui.add_space(t::MD);
        let (rect, _) = ui.allocate_exact_size(footer.size(), egui::Sense::hover());
        ui.painter().galley(rect.min, footer, p.muted);
    }
    fn search_box(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), egui::Sense::hover());
        self.audit.search = Some(rect);
        ui.painter().rect_filled(rect, 8, p.panel);
        let id = egui::Id::new("library-search");
        let focused = ui.memory(|m| m.has_focus(id));
        ui.painter().rect_stroke(
            rect,
            8,
            Stroke::new(
                if focused { 2.0_f32 } else { 1.0_f32 },
                if focused { p.focus } else { p.line },
            ),
            egui::StrokeKind::Inside,
        );
        icons::paint(
            ui.painter(),
            Rect::from_center_size(
                Pos2::new(rect.left() + 20.0, rect.center().y),
                Vec2::splat(16.0),
            ),
            Icon::Search,
            p.muted,
        );
        let text_rect = Rect::from_min_max(
            rect.min + Vec2::new(40.0, 4.0),
            rect.max - Vec2::new(40.0, 4.0),
        );
        let edit = ui
            .put(
                text_rect,
                egui::TextEdit::singleline(&mut self.search)
                    .id(id)
                    .hint_text(t::text(lang.tr(K::Search), 14.0, p.muted))
                    .font(t::font(14.0))
                    .frame(false)
                    .margin(egui::Margin::symmetric(0, 5)),
            )
            .on_hover_text(lang.tr(K::Search));
        edit.widget_info(|| {
            let mut info = egui::WidgetInfo::text_edit(ui.is_enabled(), &self.search, &self.search);
            info.label = Some(lang.tr(K::Search).into());
            info
        });
        let clear_rect = Rect::from_center_size(
            Pos2::new(rect.right() - 20.0, rect.center().y),
            Vec2::splat(t::CONTROL),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(clear_rect));
        child.add_enabled_ui(!self.search.is_empty(), |ui| {
            if t::icon_button(ui, lang.tr(K::ClearSearch), Icon::Close, false, p).clicked() {
                self.search.clear();
                ui.memory_mut(|m| m.request_focus(id));
            }
        });
    }
}
