use super::{
    app::{CavalryKitApp, Filter, Row},
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
        self.update_rows();
        let card_height = ui.available_height();
        let card = egui::Frame::new()
            .fill(egui::Color32::TRANSPARENT)
            .inner_margin(0)
            .show(ui, |ui| {
                ui.set_min_height(card_height);
                let (toolbar, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), t::CONTROL),
                    egui::Sense::hover(),
                );
                let count = lang
                    .tr(K::Count)
                    .replace("{n}", &self.rows.len().to_string());
                let count_g = ui.painter().layout_no_wrap(count, t::font(12.0), p.muted);
                let filter_width =
                    (toolbar.width() - t::CONTROL - t::MD - count_g.size().x - t::MD).min(240.0);
                let filter_rect =
                    Rect::from_min_size(toolbar.min, Vec2::new(filter_width, t::CONTROL));
                let mut filters = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("list-filters")
                        .max_rect(filter_rect),
                );
                let current = match self.filter {
                    Filter::All => 0,
                    Filter::Scripts => 1,
                    Filter::Plugins => 2,
                };
                let (chosen, rects) = t::segmented(
                    &mut filters,
                    &[lang.tr(K::All), lang.tr(K::Scripts), lang.tr(K::Plugins)],
                    current,
                    filter_width,
                    p,
                );
                self.audit.filters = rects;
                self.filter = [Filter::All, Filter::Scripts, Filter::Plugins][chosen];
                ui.painter().galley(
                    Pos2::new(
                        toolbar.right() - t::CONTROL - t::MD - count_g.size().x,
                        toolbar.center().y - count_g.size().y / 2.0,
                    ),
                    count_g,
                    p.muted,
                );
                let refresh_rect = Rect::from_min_size(
                    Pos2::new(toolbar.right() - t::CONTROL, toolbar.top()),
                    Vec2::splat(t::CONTROL),
                );
                let mut refresh = ui.new_child(egui::UiBuilder::new().max_rect(refresh_rect));
                refresh.add_enabled_ui(self.busy.is_none() && self.conflicts.is_empty(), |ui| {
                    if t::icon_button(ui, lang.tr(K::Refresh), Icon::Refresh, false, p).clicked() {
                        self.refresh_library();
                    }
                });
                ui.add_space(t::MD);
                self.search_box(ui);
                self.update_rows();
                ui.add_space(t::MD);
                let height = ui.available_height().max(48.0);
                let reveal = super::motion::reveal(
                    ui.ctx(),
                    egui::Id::new("library-reveal"),
                    egui::Id::new(self.filter as u8),
                    0.24,
                );
                let reveal = super::motion::phase(reveal, 0.0, 1.0);
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
                                            let rect = Rect::from_min_max(
                                                rect.min,
                                                rect.max - Vec2::new(0.0, t::SM),
                                            );
                                            let hover = t::motion(
                                                ui,
                                                response.id.with("row-hover"),
                                                response.hovered(),
                                            );
                                            let background =
                                                t::mix(p.panel, p.surface, hover * 0.8);
                                            ui.painter().rect_filled(rect, t::RADIUS, background);
                                            if hover > 0.0 {
                                                ui.painter().rect_filled(
                                                    Rect::from_min_size(
                                                        rect.min + Vec2::new(0.0, 12.0),
                                                        Vec2::new(2.0, rect.height() - 24.0),
                                                    ),
                                                    1,
                                                    p.accent_text.gamma_multiply(hover * 0.7),
                                                );
                                            }
                                            let dy = 4.0 * (1.0 - reveal);
                                            let tile = Rect::from_center_size(
                                                Pos2::new(rect.left() + 28.0, rect.center().y + dy),
                                                Vec2::splat(28.0),
                                            );
                                            let mut painter = ui.painter().clone();
                                            painter.multiply_opacity(reveal);
                                            painter.rect_filled(
                                                tile,
                                                6,
                                                if kind == K::Plugins {
                                                    p.selected
                                                } else {
                                                    t::mix(p.surface, p.panel, 0.5)
                                                },
                                            );
                                            icons::paint(
                                                &painter,
                                                tile.shrink(6.0),
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
                                                    t::font(13.0),
                                                    p.text,
                                                )
                                                .size()
                                                .x
                                                + 32.0)
                                                .max(64.0);
                                            let actions = reveal_width + t::CONTROL + t::SM;
                                            let x = rect.left() + 56.0;
                                            let width =
                                                (rect.width() - 56.0 - actions - 2.0 * t::SM)
                                                    .max(16.0);
                                            let name_g = t::label_truncated(
                                                ui,
                                                name,
                                                t::font(13.0),
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
                                            let name_y =
                                                rect.center().y - text_height / 2.0_f32 + dy;
                                            let sub_y = name_y + name_g.size().y + t::XS;
                                            painter.galley(Pos2::new(x, name_y), name_g, p.text);
                                            painter.galley(Pos2::new(x, sub_y), sub_g, p.muted);
                                            response.on_hover_text(format!("{name}\n{subtitle}"));
                                            let action_rect = Rect::from_min_max(
                                                Pos2::new(
                                                    rect.right() - actions,
                                                    rect.center().y - t::CONTROL / 2.0_f32,
                                                ),
                                                Pos2::new(
                                                    rect.right() - t::SM,
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
                                            child.multiply_opacity(reveal);
                                            if reveal < 0.85 {
                                                child.disable();
                                            }
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
    }
    fn search_box(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), egui::Sense::hover());
        self.audit.search = Some(rect);
        ui.painter().rect_filled(rect, t::RADIUS, p.panel);
        let id = egui::Id::new("library-search");
        let focused = ui.memory(|m| m.has_focus(id));
        ui.painter().rect_stroke(
            rect,
            8,
            Stroke::new(if focused { 1.0_f32 } else { 0.0_f32 }, p.focus),
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
        if !self.search.is_empty()
            && t::icon_button(&mut child, lang.tr(K::ClearSearch), Icon::Close, false, p).clicked()
        {
            self.search.clear();
            child.memory_mut(|m| m.request_focus(id));
        }
    }
}
