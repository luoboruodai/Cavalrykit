use super::{
    app::{CavalryKitApp, Filter, Row},
    theme as t,
};
use crate::{i18n::Key as K, paths};
use egui::{Pos2, Stroke, Vec2};
impl CavalryKitApp {
    pub(crate) fn show_library(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        ui.horizontal(|ui| {
            ui.label(t::heading(lang.tr(K::Library), 22.0, p));
            ui.label(t::text(
                lang.tr(K::Count).replace(
                    "{n}",
                    &(self.library.scripts.len() + self.library.plugins.len()).to_string(),
                ),
                12.0,
                p.muted,
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(self.busy.is_none() && self.conflicts.is_empty(), |ui| {
                    if t::button(ui, lang.tr(K::Refresh), t::Button::Secondary, p).clicked() {
                        self.refresh_library();
                    }
                });
            });
        });
        ui.add_space(t::MD);
        let wide = ui.available_width() >= 600.0;
        if wide {
            ui.horizontal(|ui| {
                self.filters(ui);
                ui.add_space(t::SM);
                self.search_box(ui);
            });
        } else {
            self.filters(ui);
            self.search_box(ui);
        }
        ui.add_space(t::MD);
        self.update_rows();
        let height = (ui.available_height() - 72.0).max(96.0);
        let card = egui::Frame::new()
            .fill(p.surface)
            .stroke(Stroke::new(1.0_f32, p.border))
            .corner_radius(8)
            .inner_margin(t::MD)
            .show(ui, |ui| {
                ui.set_min_height(height);
                ui.set_max_height(height);

                if self.rows.is_empty() {
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), height),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.add_space((height / 2.0 - 56.0).max(16.0));
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
                            ui.add(
                                egui::Label::new(t::text(
                                    lang.tr(if searching {
                                        K::NoResultsHint
                                    } else {
                                        K::EmptyHint
                                    }),
                                    13.0,
                                    p.muted,
                                ))
                                .wrap(),
                            );
                            if searching
                                && t::button(ui, lang.tr(K::ClearSearch), t::Button::Secondary, p)
                                    .clicked()
                            {
                                self.search.clear();
                            }
                        },
                    );
                } else {
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
                                            ui.painter().rect_filled(rect, 6, p.raised);
                                        }
                                        let tile = egui::Rect::from_center_size(
                                            Pos2::new(rect.left() + 16.0, rect.center().y),
                                            Vec2::splat(32.0),
                                        );
                                        let tile_color = t::mix(p.surface, p.accent, 0.10);
                                        ui.painter().rect_filled(tile, 6, tile_color);
                                        let code = if kind == K::Scripts { "JS" } else { "PL" };
                                        let g = ui.painter().layout_no_wrap(
                                            code.into(),
                                            t::medium(11.0),
                                            t::ink(tile_color),
                                        );
                                        ui.painter().galley(
                                            tile.center() - g.size() / 2.0,
                                            g,
                                            t::ink(tile_color),
                                        );
                                        let actions = [K::Reveal, K::Uninstall]
                                            .into_iter()
                                            .map(|key| {
                                                ui.painter()
                                                    .layout_no_wrap(
                                                        lang.tr(key).into(),
                                                        t::medium(14.0),
                                                        p.text,
                                                    )
                                                    .size()
                                                    .x
                                                    .max(32.0)
                                                    + 32.0
                                            })
                                            .sum::<f32>()
                                            + t::SM;
                                        let x = rect.left() + 40.0;
                                        let width = (rect.width() - 40.0 - actions - 8.0).max(24.0);
                                        let name_g = t::label_truncated(
                                            ui,
                                            name,
                                            t::font(14.0),
                                            p.text,
                                            width,
                                        );
                                        let name_h = name_g.size().y;
                                        ui.painter().galley(
                                            Pos2::new(x, rect.center().y - 18.0),
                                            name_g,
                                            p.text,
                                        );
                                        let subtitle = if let Some(value) = meta {
                                            format!("{} · {value}", lang.tr(kind))
                                        } else {
                                            lang.tr(kind).into()
                                        };
                                        let g = t::label_truncated(
                                            ui,
                                            &subtitle,
                                            t::font(12.0),
                                            p.muted,
                                            width,
                                        );
                                        ui.painter().galley(
                                            Pos2::new(x, rect.center().y - 18.0 + name_h + 4.0),
                                            g,
                                            p.muted,
                                        );
                                        response.on_hover_text(name);
                                        let action_rect = egui::Rect::from_min_max(
                                            Pos2::new(
                                                rect.right() - actions,
                                                rect.center().y - 18.0,
                                            ),
                                            Pos2::new(rect.right(), rect.center().y + 18.0),
                                        );
                                        let mut actions_ui = ui.new_child(
                                            egui::UiBuilder::new().max_rect(action_rect).layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                            ),
                                        );
                                        if t::button(
                                            &mut actions_ui,
                                            lang.tr(K::Uninstall),
                                            t::Button::QuietDanger,
                                            p,
                                        )
                                        .clicked()
                                        {
                                            self.confirm_uninstall = Some(path.clone());
                                        }
                                        if t::button(
                                            &mut actions_ui,
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
                }
            });
        self.audit.card = Some(card.response.rect);
        ui.add_space(t::SM);
        let hint = if let Some(status) = self.busy {
            lang.tr(status)
        } else if self.paths.is_none() {
            lang.tr(K::DirectoriesUnavailable)
        } else {
            lang.tr(K::IdleHint)
        };
        ui.add(
            egui::Label::new(t::text(
                format!("v{} · {hint}", env!("CARGO_PKG_VERSION")),
                12.0,
                p.muted,
            ))
            .wrap(),
        );
    }
    fn filters(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        ui.horizontal(|ui| {
            for (filter, key) in [
                (Filter::All, K::All),
                (Filter::Scripts, K::Scripts),
                (Filter::Plugins, K::Plugins),
            ] {
                let selected = self.filter == filter;
                let response = ui.add_sized(
                    [64.0, 36.0],
                    egui::SelectableLabel::new(
                        selected,
                        t::text(
                            lang.tr(key),
                            14.0,
                            if selected { p.on_accent } else { p.muted },
                        ),
                    ),
                );
                if response.clicked() {
                    self.filter = filter;
                }
            }
        });
    }
    fn search_box(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        ui.horizontal(|ui| {
            let clear = 32.0;
            let width = (ui.available_width() - clear - t::SM).max(96.0);
            let response = ui.add_sized(
                [width, 36.0],
                egui::TextEdit::singleline(&mut self.search)
                    .id(egui::Id::new("library-search"))
                    .hint_text(lang.tr(K::Search))
                    .font(t::font(14.0))
                    .margin(egui::Margin::symmetric(8, 8)),
            );
            response.on_hover_text(lang.tr(K::Search));
            let clear_response = ui
                .scope(|ui| {
                    ui.spacing_mut().button_padding = Vec2::splat(t::SM);
                    ui.add_enabled(
                        !self.search.is_empty(),
                        egui::Button::new(t::text("×", 18.0, p.muted))
                            .min_size(Vec2::new(clear, 36.0)),
                    )
                })
                .inner
                .on_hover_text(lang.tr(K::ClearSearch));
            clear_response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    lang.tr(K::ClearSearch),
                )
            });
            if clear_response.clicked() {
                self.search.clear();
            }
        });
    }
}
