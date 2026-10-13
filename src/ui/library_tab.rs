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
            search_row.max - Vec2::new(t::SEARCH_HEIGHT + t::SM, 0.0),
        );
        let mut search_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("search-row")
                .max_rect(search_rect),
        );
        self.search_box(&mut search_ui);
        let refresh_rect = Rect::from_center_size(
            Pos2::new(
                search_row.right() - t::SEARCH_HEIGHT / 2.0,
                search_row.center().y,
            ),
            Vec2::splat(t::SEARCH_HEIGHT),
        );
        let mut refresh = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("refresh")
                .max_rect(refresh_rect),
        );
        let r = refresh
            .add_enabled_ui(self.busy.is_none() && self.conflicts.is_empty(), |ui| {
                t::refresh_button(ui, lang.tr(K::Refresh), p)
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
        let view_changed = self
            .list_view_key
            .as_ref()
            .is_none_or(|(filter, query)| *filter != self.filter || query != self.search.trim());
        if view_changed {
            self.list_view_key = Some((self.filter, self.search.trim().to_owned()));
            self.selected_row = None;
            ui.memory_mut(|m| m.close_popup());
        }
        let reveal = if self.rows.is_empty() {
            1.0
        } else {
            super::motion::reveal(
                ui.ctx(),
                egui::Id::new("library-reveal"),
                egui::Id::new(self.filter as u8),
                0.70,
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
            let mut scroll = egui::ScrollArea::vertical()
                .id_salt("library-list")
                .auto_shrink([false, false])
                .max_height(height);
            if view_changed {
                scroll = scroll.vertical_scroll_offset(0.0);
            }
            scroll.show_rows(ui, t::ROW, self.rows.len(), |ui, range| {
                for index in range {
                    let start = index.min(8) as f32 * 0.065;
                    let row_reveal = super::motion::phase(reveal, start, start + 0.48);
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
                        ui.add_enabled_ui(self.busy.is_none() && row_reveal >= 0.99, |ui| {
                            let (slot, _) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), t::ROW),
                                egui::Sense::hover(),
                            );
                            let hit =
                                Rect::from_min_max(slot.min, slot.max - Vec2::new(0.0, t::SM));
                            // The menu owns its fixed 40pt cell. A row click only selects; never executes.
                            let action = Rect::from_center_size(
                                Pos2::new(hit.right() - t::SM - t::CONTROL / 2.0, hit.center().y),
                                Vec2::splat(t::CONTROL),
                            );
                            let select_rect = Rect::from_min_max(
                                hit.min,
                                Pos2::new(action.left() - t::SM, hit.bottom()),
                            );
                            let selected = self.selected_row.as_ref() == Some(path);
                            let response = ui.interact(
                                select_rect,
                                ui.id().with("select"),
                                egui::Sense::click(),
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::WidgetType::SelectableLabel,
                                    ui.is_enabled(),
                                    selected,
                                    name,
                                )
                            });
                            if response.clicked() {
                                self.selected_row = Some(path.clone());
                            }
                            let popup_id = ui.id().with("more-menu");
                            let active = ui.is_enabled()
                                && (selected
                                    || response.has_focus()
                                    || ui.rect_contains_pointer(hit)
                                    || ui.memory(|m| m.is_popup_open(popup_id)));
                            let hover = t::motion(ui, ui.id().with("row-hover"), active);
                            let tone = t::category(p, kind == K::Plugins);
                            self.audit.row_colors.push(tone.color);
                            self.audit.row_hits.push(hit);
                            let dy = 14.0 * (1.0 - row_reveal);
                            let rect = hit.shrink2(Vec2::new(t::SM * (1.0 - hover), 0.0));
                            self.audit
                                .row_visuals
                                .push((rect.translate(Vec2::new(0.0, dy)), row_reveal));
                            let mut more = None;
                            ui.with_visual_transform(
                                egui::emath::TSTransform::from_translation(Vec2::new(0.0, dy)),
                                |ui| {
                                    ui.multiply_opacity(row_reveal);
                                    ui.painter().rect(
                                        rect,
                                        t::CARD_RADIUS,
                                        t::mix(p.panel, p.surface, hover),
                                        Stroke::new(
                                            if response.has_focus() { 1.5 } else { hover },
                                            tone.color,
                                        ),
                                        egui::StrokeKind::Inside,
                                    );
                                    ui.painter().rect_filled(
                                        Rect::from_min_size(
                                            rect.min + Vec2::new(0.0, 4.0 * (1.0 - hover)),
                                            Vec2::new(
                                                2.0 + 2.0 * hover,
                                                rect.height() - 8.0 * (1.0 - hover),
                                            ),
                                        ),
                                        2,
                                        tone.color,
                                    );
                                    let tile = Rect::from_center_size(
                                        Pos2::new(rect.left() + 28.0, rect.center().y),
                                        Vec2::splat(28.0),
                                    );
                                    ui.painter().rect_filled(tile, 6, tone.tint);
                                    icons::paint(
                                        ui.painter(),
                                        tile.shrink(6.0),
                                        if kind == K::Scripts {
                                            Icon::Script
                                        } else {
                                            Icon::Plugin
                                        },
                                        tone.color,
                                    );
                                    let x = rect.left() + 52.0;
                                    let width = (action.left() - t::MD - x).max(16.0);
                                    let title =
                                        t::label_truncated(ui, name, t::font(14.0), p.text, width);
                                    let subtitle = meta
                                        .map(|m| format!("{} · {m}", lang.tr(kind)))
                                        .unwrap_or_else(|| lang.tr(kind).into());
                                    let detail = t::label_truncated(
                                        ui,
                                        &subtitle,
                                        t::font(11.0),
                                        t::helper_ink(p),
                                        width,
                                    );
                                    let title_h = title.size().y;
                                    let y =
                                        rect.center().y - (title_h + t::XS + detail.size().y) / 2.0;
                                    if title.elided || detail.elided {
                                        response.on_hover_text(format!("{name}\n{subtitle}"));
                                    }
                                    ui.painter().galley(Pos2::new(x, y), title, p.text);
                                    ui.painter().galley(
                                        Pos2::new(x, y + title_h + t::XS),
                                        detail,
                                        p.muted,
                                    );
                                    let mut child =
                                        ui.new_child(egui::UiBuilder::new().max_rect(action));
                                    child.multiply_opacity(row_reveal);
                                    more = Some(t::icon_button(
                                        &mut child,
                                        &format!("{} · {name}", lang.tr(K::More)),
                                        Icon::More,
                                        false,
                                        p,
                                    ));
                                },
                            );
                            let more = more.unwrap();
                            self.audit.row_menu_buttons.push(more.rect);
                            if more.clicked() {
                                ui.memory_mut(|m| m.toggle_popup(popup_id));
                            }
                            // Popup Areas do not inherit disabled parent UIs. Close only this
                            // row popup when a modal/busy/reveal disables the background.
                            if !ui.is_enabled() && ui.memory(|m| m.is_popup_open(popup_id)) {
                                ui.memory_mut(|m| m.close_popup());
                            }
                            egui::popup::popup_below_widget(
                                ui,
                                popup_id,
                                &more,
                                egui::popup::PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    ui.set_min_width(176.0);
                                    ui.spacing_mut().item_spacing.y = t::SM;
                                    let show =
                                        t::button(ui, lang.tr(K::Reveal), t::Button::Quiet, p);
                                    ui.separator();
                                    let remove = t::button(
                                        ui,
                                        lang.tr(K::Uninstall),
                                        t::Button::QuietDanger,
                                        p,
                                    );
                                    self.audit.row_actions.push((show.rect, remove.rect));
                                    if show.clicked() {
                                        ui.memory_mut(|m| m.close_popup());
                                        paths::reveal_in_file_manager(path);
                                    }
                                    if remove.clicked() {
                                        ui.memory_mut(|m| m.close_popup());
                                        self.confirm_uninstall = Some(path.clone());
                                    }
                                },
                            );
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
        // The TextEdit covers most of the field, so Response::hovered alone misses it.
        let hovered = ui.is_enabled() && ui.rect_contains_pointer(rect);
        r.on_hover_cursor(egui::CursorIcon::Text);
        let focused = ui.memory(|m| m.has_focus(id));
        let tone = t::category(p, false);
        let hover = t::motion(
            ui,
            id.with("hover"),
            ui.is_enabled() && (hovered || focused),
        );
        self.audit.search_hover = hover;
        ui.painter().rect(
            rect,
            10,
            t::mix(p.panel, tone.tint, hover * 0.6),
            Stroke::new(if focused { 1.5 } else { hover }, tone.color),
            egui::StrokeKind::Inside,
        );
        let icon = Rect::from_center_size(
            Pos2::new(rect.left() + 20.0, rect.center().y),
            Vec2::splat(16.0),
        );
        self.audit.search_icon = Some(icon);
        icons::paint(
            ui.painter(),
            icon,
            Icon::Search,
            t::mix(p.muted, tone.color, hover),
        );
        let text_rect = Rect::from_min_max(
            rect.min + Vec2::new(40.0, 0.0),
            rect.max - Vec2::new(clear_width.max(t::SM), 0.0),
        );
        let mut text_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("search-editor")
                .max_rect(text_rect),
        );
        let edit = egui::TextEdit::singleline(&mut self.search)
            .id(id)
            .font(t::font(14.0))
            .frame(false)
            .margin(egui::Margin::ZERO)
            .desired_width(text_rect.width())
            .min_size(text_rect.size())
            .vertical_align(egui::Align::Center)
            .show(&mut text_ui);
        self.audit.search_text = Some(Rect::from_min_size(edit.galley_pos, edit.galley.size()));
        let hint = super::search_hint::current(
            ui.ctx(),
            ui.is_enabled()
                && ui.input(|i| i.focused)
                && !edit.response.has_focus()
                && self.search.is_empty(),
        );
        if self.search.is_empty() {
            let label = lang.tr(hint.key);
            self.audit.search_hint = label.into();
            let g = t::label_truncated(
                ui,
                label,
                t::font(14.0),
                t::placeholder_ink(p),
                text_rect.width(),
            );
            let pos = Pos2::new(
                text_rect.left(),
                text_rect.center().y - g.size().y / 2.0 + hint.offset,
            );
            let mut painter = ui
                .painter()
                .with_clip_rect(text_rect.intersect(ui.clip_rect()));
            painter.multiply_opacity(hint.opacity);
            painter.galley(pos, g, t::placeholder_ink(p));
        }
        edit.response.widget_info(|| {
            let mut info = egui::WidgetInfo::text_edit(ui.is_enabled(), &self.search, &self.search);
            info.label = Some(lang.tr(K::Search).into());
            info
        });
        if !self.search.is_empty() {
            let clear_rect = Rect::from_center_size(
                Pos2::new(rect.right() - 20.0, rect.center().y),
                Vec2::splat(t::CONTROL),
            );
            let mut clear = ui.new_child(egui::UiBuilder::new().max_rect(clear_rect));
            self.audit.clear_search = Some(clear_rect);
            if t::icon_button(&mut clear, lang.tr(K::ClearSearch), Icon::Close, false, p).clicked()
            {
                self.search.clear();
                clear.memory_mut(|m| m.request_focus(id));
            }
        }
    }
}
