use super::{app::CavalryKitApp, icons::Icon, theme as t};
use crate::{
    i18n::Key as K,
    settings::{Language, RestartPrompt, Settings, Theme},
};
use egui::{Pos2, Rect, Stroke, Vec2};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    General,
    Appearance,
    Installation,
    Cavalry,
}
impl CavalryKitApp {
    pub(crate) fn show_settings(&mut self, ctx: &egui::Context) {
        let mut draft = self.settings_draft.as_ref().unwrap().clone();
        let lang = draft.language;
        let p = t::palette(&draft);
        let saving = self.busy == Some(K::SaveBusy);
        let can_save = self.busy.is_none();
        let error = self.settings_error.clone();
        let (mut save, mut cancel, mut reset, mut browse) = (false, false, false, false);
        let response = t::modal(ctx, "settings", p, 720.0, |ui| {
            let start = ui.cursor().top();
            ui.label(t::heading(lang.tr(K::SettingsTitle), 22.0, p));
            ui.add(egui::Label::new(t::text(lang.tr(K::PreviewHint), 12.0, p.muted)).wrap());
            ui.add_space(t::MD);
            let header_height = ui.cursor().top() - start;
            let body_height = (ctx.screen_rect().height()
                - header_height
                - 2.0_f32 * t::LG
                - t::CONTROL
                - t::MD
                - 48.0)
                .clamp(128.0, 360.0);
            let (body, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), body_height),
                egui::Sense::hover(),
            );
            self.audit.settings_body = Some(body);
            let nav_width = 144.0;
            let nav_rect = Rect::from_min_size(body.min, Vec2::new(nav_width, body_height));
            let mut nav = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("settings-sections")
                    .max_rect(nav_rect),
            );
            nav.spacing_mut().item_spacing.y = t::XS;
            nav.add_enabled_ui(!saving, |ui| {
                for (section, key, icon) in [
                    (Section::General, K::LanguageSection, Icon::Display),
                    (Section::Appearance, K::AppearanceSection, Icon::Palette),
                    (Section::Installation, K::BehaviorSection, Icon::Import),
                    (Section::Cavalry, K::CavalrySection, Icon::Cavalry),
                ] {
                    if t::nav_button(
                        ui,
                        lang.tr(key),
                        icon,
                        Some(self.settings_section == section),
                        None,
                        p,
                    )
                    .clicked()
                    {
                        self.settings_section = section;
                    }
                }
            });
            let content_rect = Rect::from_min_max(
                Pos2::new(body.left() + nav_width + t::LG, body.top()),
                body.max,
            );
            let mut content = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("settings-content")
                    .max_rect(content_rect),
            );
            egui::Frame::new()
                .fill(p.bg)
                .corner_radius(8)
                .inner_margin(t::MD)
                .show(&mut content, |ui| {
                    ui.set_min_size(content_rect.size() - Vec2::splat(2.0_f32 * t::MD));
                    egui::ScrollArea::vertical()
                        .id_salt(("settings-body", self.settings_section as u8))
                        .auto_shrink([false, false])
                        .max_height(body_height - 2.0_f32 * t::MD)
                        .show(ui, |ui| {
                            let page = super::motion::reveal(
                                ctx,
                                egui::Id::new("settings-page-reveal"),
                                egui::Id::new(self.settings_section as u8),
                                0.20,
                            );
                            let page = super::motion::phase(page, 0.0, 1.0);
                            ui.multiply_opacity(page);
                            ui.add_enabled_ui(!saving && page >= 0.85, |ui| {
                                if let Some(error) = &error {
                                    ui.label(t::text(lang.tr(K::SettingsError), 13.0, p.danger));
                                    egui::CollapsingHeader::new(lang.tr(K::TechnicalDetails)).show(
                                        ui,
                                        |ui| {
                                            ui.add(
                                                egui::Label::new(t::text(error, 12.0, p.muted))
                                                    .wrap(),
                                            );
                                        },
                                    );
                                    ui.add_space(t::MD);
                                }
                                match self.settings_section {
                                    Section::General => {
                                        label(ui, lang.tr(K::LanguageLabel), p);
                                        egui::ComboBox::from_id_salt("language")
                                            .width(240.0_f32.min(ui.available_width()))
                                            .selected_text(draft.language.name())
                                            .show_ui(ui, |ui| {
                                                for language in Language::ALL {
                                                    if ui
                                                        .selectable_value(
                                                            &mut draft.language,
                                                            language,
                                                            language.name(),
                                                        )
                                                        .clicked()
                                                    {
                                                        close_selected_popup(ui);
                                                    }
                                                }
                                            });
                                        hint(ui, lang.tr(K::LanguageHint), p);
                                        ui.add_space(t::LG);
                                        label(ui, lang.tr(K::ScaleLabel), p);
                                        ui.horizontal(|ui| {
                                            for scale in [90, 100, 110, 125] {
                                                let selected = draft.scale_percent == scale;
                                                let response = ui.add(
                                                    egui::Button::new(t::text(
                                                        format!("{scale}%"),
                                                        13.0,
                                                        if selected { p.text } else { p.muted },
                                                    ))
                                                    .fill(if selected {
                                                        p.selected
                                                    } else {
                                                        p.panel
                                                    })
                                                    .stroke(Stroke::new(
                                                        1.0_f32,
                                                        if selected { p.focus } else { p.line },
                                                    ))
                                                    .min_size(Vec2::new(56.0, t::CONTROL))
                                                    .corner_radius(6),
                                                );
                                                if response.clicked() {
                                                    draft.scale_percent = scale;
                                                }
                                            }
                                        });
                                        hint(ui, lang.tr(K::ScaleHint), p);
                                    }
                                    Section::Appearance => {
                                        label(ui, lang.tr(K::ThemeLabel), p);
                                        theme_choices(ui, &mut draft);
                                        ui.add_space(t::MD);
                                        if draft.theme == Theme::Custom {
                                            color_row(
                                                ui,
                                                lang.tr(K::AccentLabel),
                                                &mut draft.accent,
                                                p,
                                            );
                                            color_row(
                                                ui,
                                                lang.tr(K::BackgroundLabel),
                                                &mut draft.background,
                                                p,
                                            );
                                            hint(ui, lang.tr(K::ContrastHint), p);
                                            ui.add_space(t::SM);
                                        }
                                        t::toggle_row(
                                            ui,
                                            &mut draft.reduce_motion,
                                            lang.tr(K::MotionLabel),
                                            p,
                                        );
                                    }
                                    Section::Installation => {
                                        label(ui, lang.tr(K::RestartLabel), p);
                                        egui::ComboBox::from_id_salt("restart-mode")
                                            .width(ui.available_width().min(360.0))
                                            .selected_text(lang.tr(match draft.restart_prompt {
                                                RestartPrompt::All => K::PromptAll,
                                                RestartPrompt::Plugins => K::PromptPlugins,
                                                RestartPrompt::Never => K::PromptNever,
                                            }))
                                            .show_ui(ui, |ui| {
                                                for (mode, key) in [
                                                    (RestartPrompt::All, K::PromptAll),
                                                    (RestartPrompt::Plugins, K::PromptPlugins),
                                                    (RestartPrompt::Never, K::PromptNever),
                                                ] {
                                                    if ui
                                                        .selectable_value(
                                                            &mut draft.restart_prompt,
                                                            mode,
                                                            lang.tr(key),
                                                        )
                                                        .clicked()
                                                    {
                                                        close_selected_popup(ui);
                                                    }
                                                }
                                            });
                                        ui.add_space(t::MD);
                                        t::toggle_row(
                                            ui,
                                            &mut draft.refresh_on_focus,
                                            lang.tr(K::RefreshFocus),
                                            p,
                                        );
                                        ui.add_space(t::MD);
                                        egui::Frame::new()
                                            .fill(p.panel)
                                            .corner_radius(6)
                                            .inner_margin(t::MD)
                                            .show(ui, |ui| {
                                                hint(ui, lang.tr(K::SafetyHint), p);
                                            });
                                    }
                                    Section::Cavalry => {
                                        label(ui, lang.tr(K::AppPathLabel), p);
                                        ui.add(
                                            egui::TextEdit::singleline(&mut draft.cavalry_path)
                                                .id_salt("cavalry-path")
                                                .background_color(p.surface)
                                                .hint_text(t::text(
                                                    lang.tr(K::AutoDetect),
                                                    14.0,
                                                    p.muted,
                                                ))
                                                .desired_width(f32::INFINITY)
                                                .margin(egui::Margin::symmetric(8, 8)),
                                        );
                                        if t::button(
                                            ui,
                                            lang.tr(K::Browse),
                                            t::Button::Secondary,
                                            p,
                                        )
                                        .clicked()
                                        {
                                            browse = true;
                                        }
                                        hint(ui, lang.tr(K::PathHint), p);
                                        ui.add_space(t::LG);
                                        hint(ui, lang.tr(K::SettingsLocal), p);
                                    }
                                }
                            });
                        });
                });
            t::footer(ui, |ui| {
                ui.add_enabled_ui(can_save, |ui| {
                    let r = t::button(
                        ui,
                        lang.tr(if saving { K::SaveBusy } else { K::Save }),
                        t::Button::Primary,
                        p,
                    );
                    self.record(K::Save, &r);
                    save = r.clicked();
                });
                ui.add_enabled_ui(!saving, |ui| {
                    let r = t::button(ui, lang.tr(K::Cancel), t::Button::Secondary, p);
                    self.record(K::Cancel, &r);
                    cancel = r.clicked();
                    let r = t::button(ui, lang.tr(K::ResetDefaults), t::Button::Quiet, p)
                        .on_hover_text(lang.tr(K::ResetHint));
                    self.record(K::ResetDefaults, &r);
                    reset = r.clicked();
                });
            });
        });
        self.audit.dialog = Some(response.response.rect);
        if browse {
            self.pick_application(ctx);
        }
        if reset {
            draft = Settings::default();
        }
        self.settings_draft = Some(draft);
        if !saving && (cancel || response.should_close()) {
            self.cancel_settings(ctx);
        } else if save {
            self.save_settings();
        }
    }
}
fn label(ui: &mut egui::Ui, text: &str, p: t::Palette) {
    ui.label(t::heading(text, 14.0, p));
}
fn hint(ui: &mut egui::Ui, text: &str, p: t::Palette) {
    ui.add(egui::Label::new(t::text(text, 12.0, p.muted)).wrap());
}
fn color_row(ui: &mut egui::Ui, label: &str, color: &mut [u8; 3], p: t::Palette) {
    ui.horizontal(|ui| {
        ui.color_edit_button_srgb(color);
        ui.label(t::text(label, 13.0, p.text));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(t::text(
                format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]),
                12.0,
                p.muted,
            ));
        });
    });
}
fn theme_choices(ui: &mut egui::Ui, draft: &mut Settings) {
    let p = t::palette(draft);
    let lang = draft.language;
    let width = (ui.available_width() - 2.0_f32 * t::SM) / 3.0;
    ui.horizontal(|ui| {
        for (theme, key) in [
            (Theme::Cavalry, K::ThemeDark),
            (Theme::Light, K::ThemeLight),
            (Theme::Custom, K::ThemeCustom),
        ] {
            let selected = draft.theme == theme;
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(width, 100.0), egui::Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::RadioButton,
                    ui.is_enabled(),
                    selected,
                    lang.tr(key),
                )
            });
            ui.painter().rect(
                rect,
                t::RADIUS,
                t::mix(
                    p.panel,
                    p.surface,
                    t::motion(ui, response.id.with("theme-hover"), response.hovered()),
                ),
                Stroke::new(
                    if response.has_focus() {
                        2.0_f32
                    } else {
                        1.0_f32
                    },
                    if selected || response.has_focus() {
                        p.focus
                    } else {
                        p.line
                    },
                ),
                egui::StrokeKind::Inside,
            );
            let preview = t::palette(&Settings {
                theme,
                ..draft.clone()
            });
            let sample = Rect::from_min_max(
                rect.min + Vec2::splat(t::SM),
                Pos2::new(rect.right() - t::SM, rect.top() + 60.0),
            );
            ui.painter().rect_filled(sample, 4, preview.bg);
            let import = Rect::from_min_max(
                sample.min + Vec2::splat(4.0),
                Pos2::new(sample.left() + sample.width() * 0.30, sample.bottom() - 4.0),
            );
            let list = Rect::from_min_max(
                Pos2::new(import.right() + 4.0, import.top()),
                sample.max - Vec2::splat(4.0),
            );
            ui.painter().rect_filled(import, 3, preview.panel);
            ui.painter().rect_filled(list, 3, preview.panel);
            ui.painter().rect_filled(
                Rect::from_min_max(
                    Pos2::new(import.left() + 3.0, import.bottom() - 7.0),
                    Pos2::new(import.right() - 3.0, import.bottom() - 3.0),
                ),
                2,
                preview.accent,
            );
            for line in 0..3 {
                let y = list.top() + 8.0 + line as f32 * 10.0;
                ui.painter().hline(
                    list.left() + 4.0..=list.right() - 4.0,
                    y,
                    Stroke::new(
                        2.0_f32,
                        if line == 0 {
                            preview.accent
                        } else {
                            preview.line
                        },
                    ),
                );
            }
            let mut job = egui::text::LayoutJob::simple(
                lang.tr(key).into(),
                t::font(12.0),
                p.text,
                width - 2.0 * t::SM,
            );
            job.halign = egui::Align::Center;
            let g = ui.painter().layout_job(job);
            ui.painter().galley(
                Pos2::new(
                    rect.center().x,
                    sample.bottom() + (rect.bottom() - sample.bottom() - g.size().y) / 2.0,
                ),
                g,
                p.text,
            );
            if response.on_hover_text(lang.tr(key)).clicked() {
                draft.theme = theme;
            }
        }
    });
}
// A ComboBox uses egui's popup stack, not the menu stack.
fn close_selected_popup(ui: &mut egui::Ui) {
    ui.memory_mut(|memory| memory.close_popup());
    ui.ctx().request_repaint();
}
#[cfg(test)]
mod tests {
    #[test]
    fn keyboard_and_accessibility_choices_close_the_popup() {
        let ctx = egui::Context::default();
        let id = egui::Id::new("language-popup");
        ctx.memory_mut(|m| m.open_popup(id));
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, super::close_selected_popup);
        });
        assert!(!ctx.memory(|m| m.is_popup_open(id)));
    }
}
