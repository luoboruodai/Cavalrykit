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
        let area = ui.max_rect();
        let (toolbar, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), t::CONTROL),
            egui::Sense::hover(),
        );
        let count = lang.tr(K::Count).replace(
            "{n}",
            &(self.library.scripts.len() + self.library.plugins.len()).to_string(),
        );
        let count_width = ui
            .painter()
            .layout_no_wrap(count, t::medium(12.0), p.muted)
            .size()
            .x
            + 2.0 * t::SM;
        let filter_width = (toolbar.width() - count_width - t::MD).min(280.0);
        let filter_rect = Rect::from_min_size(toolbar.min, Vec2::new(filter_width, t::CONTROL));
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
        self.update_rows();
        ui.add_space(t::MD);
        let (search_row, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), t::SEARCH_HEIGHT),
            egui::Sense::hover(),
        );
        let search_rect = Rect::from_min_max(
            search_row.min,
            search_row.max - Vec2::new(t::CONTROL + t::SM, 0.0),
        );
        let mut search_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("search-row")
                .max_rect(search_rect),
        );
        self.search_box(&mut search_ui);
        let refresh_rect = Rect::from_center_size(
            Pos2::new(search_row.right() - t::CONTROL / 2.0, search_row.center().y),
            Vec2::splat(t::CONTROL),
        );
        let mut refresh = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("refresh")
                .max_rect(refresh_rect),
        );
        let r = refresh
            .add_enabled_ui(self.busy.is_none() && self.conflicts.is_empty(), |ui| {
                t::icon_button(ui, lang.tr(K::Refresh), Icon::Refresh, false, p)
            })
            .inner;
        self.audit.refresh = Some(r.rect);
        if r.clicked() {
            self.refresh_library();
        }
        self.update_rows();
        let tone = t::category(p, self.filter == Filter::Plugins);
        let label = lang
            .tr(K::Count)
            .replace("{n}", &self.rows.len().to_string());
        let width = ui
            .painter()
            .layout_no_wrap(label.clone(), t::medium(12.0), tone.color)
            .size()
            .x
            + 2.0 * t::SM;
        let badge = Rect::from_min_size(
            Pos2::new(filter_rect.right() + t::MD, toolbar.center().y - 14.0),
            Vec2::new(width, 28.0),
        );
        t::count_badge(ui, badge, &label, tone);
        self.audit.count = Some(badge);
        self.audit.count_color = Some(tone.color);
        self.audit.count_label = label;

        ui.add_space(t::MD);
        let height = ui.available_height().max(48.0);
        // Empty states are static; don't start a repaint loop for an animation they don't use.
        let reveal = if self.rows.is_empty() {
            1.0
        } else {
            super::motion::reveal(
                ui.ctx(),
                egui::Id::new("library-reveal"),
                egui::Id::new(self.filter as u8),
                0.42,
            )
        };
        if self.rows.is_empty() {
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), height),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.y = t::SM;
                    ui.add_space((height / 2.0 - 48.0).max(0.0));
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
                        && t::button(ui, lang.tr(K::ClearSearch), t::Button::Secondary, p).clicked()
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
                    // Use the stable global row index, not the viewport range, so scrolling
                    // during the bounded reveal cannot retime an existing row or its actions.
                    for index in range.clone() {
                        let start = (index.min(8) as f32) * 0.045;
                        let row_reveal = super::motion::phase(reveal, start, start + 0.62);
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
                        let key = path.clone();
                        ui.push_id((kind as u32, &key), |ui| {
                            ui.add_enabled_ui(self.busy.is_none(), |ui| {
                                let (slot, response) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), t::ROW),
                                    egui::Sense::hover(),
                                );
                                let rect =
                                    Rect::from_min_max(slot.min, slot.max - Vec2::new(0.0, t::SM));
                                let tone = t::category(p, kind == K::Plugins);
                                self.audit.row_colors.push(tone.color);
                                let hover = t::motion(
                                    ui,
                                    response.id.with("row-hover"),
                                    ui.is_enabled() && response.contains_pointer(),
                                );
                                let bg = t::mix(p.panel, p.surface, hover * 0.75);
                                ui.painter().rect(
                                    rect,
                                    t::CARD_RADIUS,
                                    bg,
                                    Stroke::new(hover, tone.color),
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().rect_filled(
                                    Rect::from_min_size(
                                        rect.min + Vec2::new(0.0, 8.0),
                                        Vec2::new(3.0, rect.height() - 16.0),
                                    ),
                                    1,
                                    tone.color,
                                );
                                let mut painter = ui.painter().clone();
                                painter.multiply_opacity(row_reveal);
                                let dy = 3.0 * (1.0 - row_reveal);
                                let tile = Rect::from_center_size(
                                    Pos2::new(rect.left() + 28.0, rect.center().y + dy),
                                    Vec2::splat(28.0),
                                );
                                painter.rect_filled(tile, 6, tone.tint);
                                icons::paint(
                                    &painter,
                                    tile.shrink(6.0),
                                    if kind == K::Scripts {
                                        Icon::Script
                                    } else {
                                        Icon::Plugin
                                    },
                                    tone.color,
                                );
                                let actions = 2.0 * t::CONTROL + t::MD;
                                let x = rect.left() + 56.0 + 2.0 * hover;
                                let width =
                                    (rect.width() - 56.0 - actions - 2.0 * t::SM - 2.0).max(16.0);
                                let title =
                                    t::label_truncated(ui, name, t::font(14.0), p.text, width);
                                let subtitle = if let Some(value) = meta {
                                    format!("{} · {value}", lang.tr(kind))
                                } else {
                                    lang.tr(kind).into()
                                };
                                let detail = t::label_truncated(
                                    ui,
                                    &subtitle,
                                    t::font(11.0),
                                    p.muted,
                                    width,
                                );
                                let text_height = title.size().y + t::XS + detail.size().y;
                                let y = rect.center().y - text_height / 2.0 + dy;
                                let title_h = title.size().y;
                                // Only long/elided text needs a tooltip; don't obscure the next row on every hover.
                                if title.elided || detail.elided {
                                    response.on_hover_text(format!("{name}\n{subtitle}"));
                                }
                                painter.galley(Pos2::new(x, y), title, p.text);
                                painter.galley(Pos2::new(x, y + title_h + t::XS), detail, p.muted);
                                let action_rect = Rect::from_min_max(
                                    Pos2::new(
                                        rect.right() - t::SM - actions,
                                        rect.center().y - t::CONTROL / 2.0,
                                    ),
                                    Pos2::new(
                                        rect.right() - t::SM,
                                        rect.center().y + t::CONTROL / 2.0,
                                    ),
                                );
                                let mut child = ui.new_child(
                                    egui::UiBuilder::new()
                                        .max_rect(action_rect)
                                        .layout(egui::Layout::right_to_left(egui::Align::Center)),
                                );
                                child.spacing_mut().item_spacing.x = t::MD;
                                child.multiply_opacity(row_reveal);
                                if row_reveal < 0.85 {
                                    child.disable();
                                }
                                let remove = t::icon_button(
                                    &mut child,
                                    &format!("{} · {name}", lang.tr(K::Uninstall)),
                                    Icon::Trash,
                                    true,
                                    p,
                                );
                                if remove.clicked() {
                                    self.confirm_uninstall = Some(path.clone());
                                }
                                let show = t::icon_button(
                                    &mut child,
                                    &format!("{} · {name}", lang.tr(K::Reveal)),
                                    Icon::Reveal,
                                    false,
                                    p,
                                );
                                if show.clicked() {
                                    paths::reveal_in_file_manager(path);
                                }
                                self.audit.row_actions.push((show.rect, remove.rect));
                            });
                        });
                    }
                });
        }
        self.audit.card = Some(area);
    }
    fn search_box(&mut self, ui: &mut egui::Ui) {
        let p = self.p();
        let lang = self.prefs().language;
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), t::SEARCH_HEIGHT),
            egui::Sense::hover(),
        );
        self.audit.search = Some(rect);
        let id = egui::Id::new("library-search");
        let clear_width = if self.search.is_empty() {
            0.0
        } else {
            t::CONTROL
        };
        let focus_rect = Rect::from_min_max(rect.min, rect.max - Vec2::new(clear_width, 0.0));
        let r = ui.interact(focus_rect, id.with("whole-field"), egui::Sense::CLICK);
        if r.clicked() {
            ui.memory_mut(|m| m.request_focus(id));
        }
        let hovered = r.hovered();
        r.on_hover_cursor(egui::CursorIcon::Text);
        let focused = ui.memory(|m| m.has_focus(id));
        let tone = t::category(p, self.filter == Filter::Plugins);
        let hover = t::motion(ui, id.with("hover"), hovered || focused);
        self.audit.search_hover = hover;
        ui.painter()
            .rect_filled(rect, 10, t::mix(p.panel, tone.tint, hover * 0.55));
        ui.painter().rect_stroke(
            rect,
            10,
            Stroke::new(if focused { 1.5 } else { hover }, tone.color),
            egui::StrokeKind::Inside,
        );
        icons::paint(
            ui.painter(),
            Rect::from_center_size(
                Pos2::new(rect.left() + 20.0, rect.center().y),
                Vec2::splat(16.0),
            ),
            Icon::Search,
            t::mix(p.muted, tone.color, hover),
        );
        let text_rect = Rect::from_min_max(
            rect.min + Vec2::new(40.0, 0.0),
            rect.max - Vec2::new(clear_width.max(t::SM), 0.0),
        );
        let edit = ui
            .put(
                text_rect,
                egui::TextEdit::singleline(&mut self.search)
                    .id(id)
                    .hint_text(t::text(lang.tr(K::Search), 14.0, p.muted))
                    .font(t::font(14.0))
                    .frame(false)
                    .margin(egui::Margin::symmetric(0, 10)),
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
        let mut clear = ui.new_child(egui::UiBuilder::new().max_rect(clear_rect));
        if !self.search.is_empty() {
            self.audit.clear_search = Some(clear_rect);
        }
        if !self.search.is_empty()
            && t::icon_button(&mut clear, lang.tr(K::ClearSearch), Icon::Close, false, p).clicked()
        {
            self.search.clear();
            clear.memory_mut(|m| m.request_focus(id));
        }
    }
}
