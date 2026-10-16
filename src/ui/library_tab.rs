use super::{
    app::{CavalryKitApp, Filter, Row},
    icons::{self, Icon},
    theme as t,
};
use crate::{i18n::Key as K, paths};
use egui::{Pos2, Rect, Stroke, Vec2};
impl CavalryKitApp {
    pub(crate) fn show_library(&mut self, ui: &mut egui::Ui) {
        let p = self.paint_palette(ui.ctx());
        let lang = self.prefs().language;
        ui.spacing_mut().item_spacing.y = 0.0;
        self.update_rows();
        let area = ui.max_rect();
        // Reserve the scrollbar lane even for a short/filtered list; align toolbar to
        // the resting card, not the viewport or the outer hover extent.
        let row_width = (ui.available_width() - t::LIST_GUTTER).max(160.0);
        let (toolbar_slot, _) =
            ui.allocate_exact_size(Vec2::new(row_width, t::CONTROL), egui::Sense::hover());
        let toolbar = toolbar_slot.shrink2(Vec2::new(t::ROW_INSET, 0.0));
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
        let (search_slot, _) =
            ui.allocate_exact_size(Vec2::new(row_width, t::SEARCH_HEIGHT), egui::Sense::hover());
        let search_row = search_slot.shrink2(Vec2::new(t::ROW_INSET, 0.0));
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
        let count_ink = t::count_badge(ui, badge, &label, tone, p);
        self.audit.count = Some(badge);
        self.audit.count_color = Some(count_ink);
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
            let searching = !self.search.trim().is_empty();
            let scanning = !searching && self.busy.is_some();
            // Soft entrance once per state change; typing in the box must not replay it.
            let enter = super::motion::reveal(
                ui.ctx(),
                egui::Id::new("empty-state"),
                egui::Id::new((searching, scanning, self.filter as u8)),
                0.45,
            );
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), height),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.multiply_opacity(enter);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let tone = t::category(p, false);
                    let compact = height < 440.0;
                    let cluster_size = if compact {
                        Vec2::new(200.0, 120.0)
                    } else {
                        Vec2::new(320.0, 160.0)
                    };
                    let hero_gap = if compact { t::LG } else { 32.0 };
                    let title_hint_gap = if compact { t::SM } else { t::MD };
                    let hint_button_gap = if compact { t::MD } else { t::LG };
                    let card_width = ui.available_width().min(680.0);
                    let content_width = (card_width - 2.0 * t::LG).max(80.0);
                    let show_hint = height > 96.0;
                    let show_button = searching && height > 140.0;
                    let title = ui.painter().layout_no_wrap(
                        lang.tr(if searching {
                            K::NoResults
                        } else if scanning {
                            K::ScanBusy
                        } else {
                            K::EmptyLibrary
                        })
                        .into(),
                        t::medium(if compact { 18.0 } else { 22.0 }),
                        p.text,
                    );
                    let hint_text: String = if searching {
                        lang.tr(K::NoResultsHint).into()
                    } else {
                        lang.tr(K::EmptyHint).into()
                    };
                    self.audit.empty_hint = Some(hint_text.clone());
                    let hint = show_hint.then(|| {
                        let mut job = egui::text::LayoutJob::simple(
                            hint_text,
                            t::font(if compact { 13.0 } else { 14.0 }),
                            t::helper_ink(p),
                            content_width,
                        );
                        job.halign = egui::Align::Center;
                        ui.painter().layout_job(job)
                    });
                    let stack = cluster_size.y
                        + hero_gap
                        + title.size().y
                        + if show_hint {
                            title_hint_gap + hint.as_ref().map_or(0.0, |g| g.size().y)
                        } else {
                            0.0
                        }
                        + if show_button {
                            hint_button_gap + t::CONTROL
                        } else {
                            0.0
                        };
                    let card_height =
                        (stack + 2.0 * t::LG).max(if compact { 280.0 } else { 400.0 });
                    let card_size = Vec2::new(card_width, card_height);
                    ui.add_space(((height - card_size.y) / 2.0).max(0.0));
                    let (_, response) = ui.allocate_exact_size(card_size, egui::Sense::hover());
                    let hover = t::motion(
                        ui,
                        response.id.with("empty-card-hover"),
                        ui.rect_contains_pointer(response.rect),
                    );
                    let visual_rect = egui::Rect::from_center_size(
                        response.rect.center(),
                        response.rect.size() * ((1.0 + hover * 0.02) / 1.02),
                    );
                    ui.painter().rect(
                        visual_rect,
                        12,
                        t::mix(p.panel, tone.tint, hover * 0.35),
                        Stroke::new(
                            0.5 + 0.25 * hover,
                            t::mix(p.line, tone.color, 0.02 + 0.16 * hover),
                        ),
                        egui::StrokeKind::Inside,
                    );
                    self.audit.empty_card = Some(visual_rect);
                    self.audit.empty_card_hit = Some(response.rect);
                    let mut content = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt("empty-card-content")
                            .max_rect(response.rect.shrink(t::LG))
                            .layout(egui::Layout::top_down(egui::Align::Center)),
                    );
                    content.spacing_mut().item_spacing.y = 0.0;
                    content
                        .add_space(((response.rect.height() - 2.0 * t::LG - stack) / 2.0).max(0.0));
                    let (cluster, _) =
                        content.allocate_exact_size(cluster_size, egui::Sense::hover());
                    let geometry_scale = if compact { 0.68 } else { 1.0 };
                    let decoration_ink = t::mix(p.muted, tone.color, 0.45 + 0.25 * hover);
                    let side_size = if compact {
                        Vec2::new(50.0, 40.0)
                    } else {
                        Vec2::new(76.0, 56.0)
                    };
                    let side_offset = if compact { 68.0 } else { 100.0 };
                    let side_spread = if compact { 16.0 } else { 20.0 };
                    let side_icon = if compact { 20.0 } else { 24.0 };
                    for (icon, direction) in [(Icon::Script, -1.0), (Icon::Plugin, 1.0)] {
                        let size = side_size * (1.0 + 0.06 * hover);
                        let rest_y = if direction < 0.0 { 16.0 } else { 4.0 };
                        let angle =
                            (if direction < 0.0 { -0.10 } else { 0.08 }) * (1.0 + 0.35 * hover);
                        let rect = Rect::from_center_size(
                            cluster.center()
                                + Vec2::new(
                                    direction * (side_offset + side_spread * hover),
                                    (rest_y - 6.0 * hover) * geometry_scale,
                                ),
                            size,
                        );
                        self.audit.empty_decorations.push(rect.expand(4.0));
                        self.audit.empty_tile_angles.push(angle);
                        icons::rect_rotated(
                            content.painter(),
                            rect,
                            12.0,
                            angle,
                            t::mix(p.surface, tone.tint, 0.30 * hover),
                            Stroke::new(0.5_f32, t::mix(p.border, tone.color, 0.12 + 0.38 * hover)),
                        );
                        icons::paint_rotated(
                            content.painter(),
                            Rect::from_center_size(
                                rect.center(),
                                Vec2::splat(side_icon * (1.0 + 0.08 * hover)),
                            ),
                            icon,
                            decoration_ink,
                            angle,
                        );
                    }
                    for (offset, radius) in [
                        (Vec2::new(-144.0, -64.0), 4.0),
                        (Vec2::new(140.0, -60.0), 3.0),
                        (Vec2::new(-136.0, 64.0), 3.0),
                        (Vec2::new(144.0, 60.0), 4.0),
                    ] {
                        let center = cluster.center()
                            + (offset + offset.normalized() * (6.0 * hover)) * geometry_scale;
                        let radius = radius * (1.0 + 0.12 * hover) * geometry_scale;
                        self.audit
                            .empty_dots
                            .push(Rect::from_center_size(center, Vec2::splat(radius * 2.0)));
                        content.painter().circle_stroke(
                            center,
                            radius,
                            Stroke::new(1.5_f32, decoration_ink),
                        );
                    }
                    let center_size = (if compact { 64.0 } else { 88.0 }) * (1.0 + 0.08 * hover);
                    let tile = Rect::from_center_size(
                        cluster.center() + Vec2::new(0.0, (-4.0 - 4.0 * hover) * geometry_scale),
                        Vec2::splat(center_size),
                    );
                    self.audit.empty_decorations.push(tile);
                    self.audit.empty_tile_angles.push(0.0);
                    content.painter().rect_filled(
                        tile,
                        16,
                        t::mix(tone.tint, tone.color, 0.08 * hover),
                    );
                    icons::paint(
                        content.painter(),
                        Rect::from_center_size(tile.center(), Vec2::splat(center_size * 0.40)),
                        if searching {
                            Icon::Search
                        } else if scanning {
                            Icon::Refresh
                        } else {
                            Icon::Import
                        },
                        tone.color,
                    );
                    self.audit.empty_cluster = Some(cluster);
                    content.add_space(hero_gap);
                    let (title_slot, _) = content.allocate_exact_size(
                        Vec2::new(content.available_width(), title.size().y),
                        egui::Sense::hover(),
                    );
                    self.audit.empty_title_slot = Some(title_slot);
                    let title_pos =
                        t::centered_ink(title_slot, &title) + Vec2::new(0.0, -4.0 * hover);
                    self.audit.empty_title_visual =
                        Some(Rect::from_min_size(title_pos, title.size()));
                    content.painter().galley(
                        title_pos,
                        title,
                        t::mix(p.text, tone.color, 0.10 * hover),
                    );
                    if let Some(hint) = hint {
                        content.add_space(title_hint_gap);
                        let (hint_slot, _) = content.allocate_exact_size(
                            Vec2::new(content.available_width(), hint.size().y),
                            egui::Sense::hover(),
                        );
                        self.audit.empty_hint_slot = Some(hint_slot);
                        let hint_pos =
                            t::centered_ink(hint_slot, &hint) + Vec2::new(0.0, -4.0 * hover);
                        self.audit.empty_hint_visual =
                            Some(Rect::from_min_size(hint_pos, hint.size()));
                        content.painter().galley(
                            hint_pos,
                            hint,
                            t::mix(t::helper_ink(p), tone.color, 0.20 * hover),
                        );
                    }
                    if show_button {
                        content.add_space(hint_button_gap);
                        let (clear, clear_visual, clear_icon) = t::secondary_icon_button(
                            &mut content,
                            lang.tr(K::ClearSearch),
                            Icon::Close,
                            p,
                            hover,
                        );
                        self.audit.empty_clear = Some(clear.rect);
                        self.audit.empty_clear_visual = Some(clear_visual);
                        self.audit.empty_clear_icon = Some(clear_icon);
                        if clear.clicked() {
                            self.search.clear();
                        }
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
                        // Input readiness is NOT the disabled visual state. During reveal
                        // we only remove click sense, so no gray tint disappears at 99%.
                        ui.add_enabled_ui(self.busy.is_none(), |ui| {
                            let ready = row_reveal >= 0.99;
                            let (slot, _) = ui.allocate_exact_size(
                                Vec2::new(row_width.min(ui.available_width()), t::ROW),
                                egui::Sense::hover(),
                            );
                            let hit =
                                Rect::from_min_max(slot.min, slot.max - Vec2::new(0.0, t::SM));
                            let dots = Rect::from_center_size(
                                Pos2::new(hit.right() - t::ROW_INSET - t::MD, hit.center().y),
                                Vec2::splat(12.0),
                            );
                            let remove_rect = Rect::from_center_size(
                                Pos2::new(dots.left() - t::SM - t::CONTROL / 2.0, hit.center().y),
                                Vec2::splat(t::CONTROL),
                            );
                            let show_rect =
                                remove_rect.translate(Vec2::new(-t::CONTROL - t::MD, 0.0));
                            let select_rect = Rect::from_min_max(
                                hit.min,
                                Pos2::new(show_rect.left() - t::SM, hit.bottom()),
                            );
                            let selected = self.selected_row.as_ref() == Some(path);
                            let response = ui.interact(
                                select_rect,
                                ui.id().with("select"),
                                if ready {
                                    egui::Sense::click()
                                } else {
                                    egui::Sense::hover()
                                },
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::WidgetType::SelectableLabel,
                                    ui.is_enabled() && ready,
                                    selected,
                                    name,
                                )
                            });
                            if response.clicked() {
                                self.selected_row = Some(path.clone());
                            }
                            let hover = t::motion(
                                ui,
                                ui.id().with("row-hover"),
                                ui.is_enabled()
                                    && ready
                                    && (selected
                                        || response.has_focus()
                                        || ui.rect_contains_pointer(hit)),
                            );
                            let tone = t::category(p, kind == K::Plugins);
                            self.audit.row_colors.push(tone.color);
                            self.audit.row_hits.push(hit);
                            let dy = 12.0 * (1.0 - row_reveal);
                            let rect = hit.shrink2(Vec2::new(t::ROW_INSET * (1.0 - hover), 0.0));
                            self.audit
                                .row_visuals
                                .push((rect.translate(Vec2::new(0.0, dy)), row_reveal));
                            self.audit.row_decorations.push(
                                dots.translate(t::row_action_offset(hover) + Vec2::new(0.0, dy)),
                            );
                            // A constant-height inset rail follows the surface instead of
                            // abruptly growing into the rounded corners on both axes.
                            let stripe = Rect::from_min_size(
                                rect.min + Vec2::new(1.5, 8.0),
                                Vec2::new(2.0 + hover * 0.5, rect.height() - 16.0),
                            );
                            self.audit
                                .row_accent
                                .push(stripe.translate(Vec2::new(0.0, dy)));
                            let paint_row = |ui: &mut egui::Ui| {
                                ui.multiply_opacity(row_reveal);
                                ui.painter().rect(
                                    rect,
                                    t::CARD_RADIUS,
                                    t::mix(p.panel, p.surface, hover * 0.75),
                                    Stroke::new(
                                        if response.has_focus() {
                                            1.5
                                        } else {
                                            hover * 0.75
                                        },
                                        t::mix(p.panel, tone.color, 0.8),
                                    ),
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().rect_filled(
                                    stripe,
                                    2,
                                    t::mix(p.panel, tone.color, 0.68 + 0.32 * hover),
                                );
                                let tile = Rect::from_center_size(
                                    Pos2::new(rect.left() + 26.0, rect.center().y),
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
                                let width = (show_rect.left() - t::SM - x).max(16.0);
                                let title = t::label_truncated(
                                    ui,
                                    name,
                                    t::content_font(14.0),
                                    p.text,
                                    width,
                                );
                                let subtitle = meta
                                    .map(|m| format!("{} · {m}", lang.tr(kind)))
                                    .unwrap_or_else(|| lang.tr(kind).into());
                                let detail = t::label_truncated(
                                    ui,
                                    &subtitle,
                                    t::content_font(10.5),
                                    t::helper_ink(p),
                                    width,
                                );
                                let title_ink = t::ink_bounds(&title);
                                let detail_ink = t::ink_bounds(&detail);
                                let top = rect.center().y
                                    - (title_ink.height() + t::XS + detail_ink.height()) / 2.0;
                                let title_pos =
                                    Pos2::new(x - title_ink.left(), top - title_ink.top());
                                let detail_pos = Pos2::new(
                                    x - detail_ink.left(),
                                    top + title_ink.height() + t::XS - detail_ink.top(),
                                );
                                self.audit.row_text_ink.push((
                                    title_ink.translate(title_pos.to_vec2() + Vec2::new(0.0, dy)),
                                    detail_ink.translate(detail_pos.to_vec2() + Vec2::new(0.0, dy)),
                                ));
                                if title.elided || detail.elided {
                                    response.on_hover_text(format!("{name}\n{subtitle}"));
                                }
                                ui.painter().galley(title_pos, title, p.text);
                                ui.painter().galley(detail_pos, detail, t::helper_ink(p));
                                t::decorative_dots(ui.painter(), dots.center(), p, hover);
                                let mut show = ui.new_child(
                                    egui::UiBuilder::new()
                                        .id_salt("reveal-action")
                                        .max_rect(show_rect),
                                );
                                let reveal = t::icon_button_guarded(
                                    &mut show,
                                    &format!("{} · {name}", lang.tr(K::Reveal)),
                                    Icon::Reveal,
                                    false,
                                    p,
                                    ready,
                                    Some(hover),
                                );
                                let mut remove = ui.new_child(
                                    egui::UiBuilder::new()
                                        .id_salt("remove-action")
                                        .max_rect(remove_rect),
                                );
                                let uninstall = t::icon_button_guarded(
                                    &mut remove,
                                    &format!("{} · {name}", lang.tr(K::Uninstall)),
                                    Icon::Trash,
                                    true,
                                    p,
                                    ready,
                                    Some(hover),
                                );
                                self.audit.row_actions.push((reveal.rect, uninstall.rect));
                                self.audit.row_action_art.push((
                                    t::action_glyph_rect(reveal.rect, Some(hover))
                                        .translate(Vec2::new(0.0, dy)),
                                    t::action_glyph_rect(uninstall.rect, Some(hover))
                                        .translate(Vec2::new(0.0, dy)),
                                ));
                                self.audit.row_action_ink.push(t::row_action_ink(p, hover));
                                if reveal.clicked() {
                                    paths::reveal_in_file_manager(path);
                                }
                                if uninstall.clicked() {
                                    self.confirm_uninstall = Some(path.clone());
                                }
                            };
                            if dy.abs() > f32::EPSILON {
                                ui.with_visual_transform(
                                    egui::emath::TSTransform::from_translation(Vec2::new(0.0, dy)),
                                    paint_row,
                                );
                            } else {
                                paint_row(ui);
                            }
                        });
                    });
                }
            });
        }
        self.audit.card = Some(area);
    }
    fn search_box(&mut self, ui: &mut egui::Ui) {
        let p = self.paint_palette(ui.ctx());
        let lang = self.prefs().language;
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), t::SEARCH_HEIGHT),
            egui::Sense::hover(),
        );
        self.audit.search = Some(rect);
        let id = egui::Id::new("library-search");
        let focus_rect = rect;
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
            rect.max - Vec2::new(t::SM, 0.0),
        );
        let sample = ui.painter().layout_no_wrap(
            if self.search.is_empty() {
                lang.tr(K::Search).into()
            } else {
                self.search.clone()
            },
            t::font(13.0),
            p.text,
        );
        let optical_shift = sample.rect.center().y - t::ink_bounds(&sample).center().y;
        let editor_rect = text_rect.translate(Vec2::new(0.0, optical_shift));
        let mut text_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("search-editor")
                .max_rect(editor_rect),
        );
        let edit = egui::TextEdit::singleline(&mut self.search)
            .id(id)
            .font(t::font(13.0))
            .frame(false)
            .margin(egui::Margin::ZERO)
            .desired_width(text_rect.width())
            .min_size(text_rect.size())
            .vertical_align(egui::Align::Center)
            .show(&mut text_ui);
        self.audit.search_text =
            Some(t::ink_bounds(&edit.galley).translate(edit.galley_pos.to_vec2()));
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
                t::font(13.0),
                t::placeholder_ink(p),
                text_rect.width(),
            );
            let pos = t::left_ink(text_rect.left(), text_rect.center().y + hint.offset, &g);
            self.audit.search_text = Some(t::ink_bounds(&g).translate(pos.to_vec2()));
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
    }
}
