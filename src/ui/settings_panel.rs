use super::{app::CavalryKitApp, theme as t};
use crate::{
    i18n::Key as K,
    settings::{Language, RestartPrompt, Settings, Theme},
};
impl CavalryKitApp {
    pub(crate) fn show_settings(&mut self, ctx: &egui::Context) {
        let mut draft = self.settings_draft.as_ref().unwrap().clone();
        let lang = draft.language;
        let p = t::palette(&draft);
        let saving = self.busy == Some(K::SaveBusy);
        let can_save = self.busy.is_none();
        let error = self.settings_error.clone();
        let mut save = false;
        let mut cancel = false;
        let mut reset = false;
        let mut browse = false;
        let response = t::modal(ctx, "settings", p, 640.0, |ui| {
            ui.label(t::heading(lang.tr(K::SettingsTitle), 22.0, p));
            ui.add_space(t::SM);
            ui.label(t::text(lang.tr(K::PreviewHint), 13.0, p.muted));
            ui.add_space(t::MD);
            egui::ScrollArea::vertical()
                .id_salt("settings-body")
                .auto_shrink([false, true])
                .max_height((ctx.screen_rect().height() - 220.0).clamp(96.0, 440.0))
                .show(ui, |ui| {
                    ui.add_enabled_ui(!saving, |ui| {
                        if let Some(error) = error {
                            ui.label(t::text(lang.tr(K::SettingsError), 13.0, p.danger));
                            egui::CollapsingHeader::new(lang.tr(K::TechnicalDetails)).show(
                                ui,
                                |ui| {
                                    ui.add(egui::Label::new(t::text(error, 12.0, p.muted)).wrap());
                                },
                            );
                            ui.add_space(t::MD);
                        }
                        section(ui, lang.tr(K::LanguageSection), p);
                        ui.label(lang.tr(K::LanguageLabel));
                        egui::ComboBox::from_id_salt("language")
                            .width(240.0)
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
                        ui.label(t::text(lang.tr(K::LanguageHint), 12.0, p.muted));
                        ui.add_space(t::SM);
                        ui.label(lang.tr(K::ScaleLabel));
                        egui::ComboBox::from_id_salt("scale")
                            .width(240.0)
                            .selected_text(format!("{}%", draft.scale_percent))
                            .show_ui(ui, |ui| {
                                for size in [90, 100, 110, 125] {
                                    if ui
                                        .selectable_value(
                                            &mut draft.scale_percent,
                                            size,
                                            format!("{size}%"),
                                        )
                                        .clicked()
                                    {
                                        close_selected_popup(ui);
                                    }
                                }
                            });
                        ui.add_space(t::LG);
                        section(ui, lang.tr(K::AppearanceSection), p);
                        ui.label(lang.tr(K::ThemeLabel));
                        egui::ComboBox::from_id_salt("theme")
                            .width(240.0)
                            .selected_text(lang.tr(match draft.theme {
                                Theme::Cavalry => K::ThemeDark,
                                Theme::Light => K::ThemeLight,
                                Theme::Custom => K::ThemeCustom,
                            }))
                            .show_ui(ui, |ui| {
                                for (theme, key) in [
                                    (Theme::Cavalry, K::ThemeDark),
                                    (Theme::Light, K::ThemeLight),
                                    (Theme::Custom, K::ThemeCustom),
                                ] {
                                    if ui
                                        .selectable_value(&mut draft.theme, theme, lang.tr(key))
                                        .clicked()
                                    {
                                        close_selected_popup(ui);
                                    }
                                }
                            });
                        if draft.theme == Theme::Custom {
                            ui.add_space(t::SM);
                            ui.horizontal(|ui| {
                                ui.color_edit_button_srgb(&mut draft.accent);
                                ui.label(lang.tr(K::AccentLabel));
                                ui.label(t::text(hex(draft.accent), 12.0, p.muted));
                            });
                            ui.horizontal(|ui| {
                                ui.color_edit_button_srgb(&mut draft.background);
                                ui.label(lang.tr(K::BackgroundLabel));
                                ui.label(t::text(hex(draft.background), 12.0, p.muted));
                            });
                            ui.label(t::text(lang.tr(K::ContrastHint), 12.0, p.muted));
                        }
                        ui.add_space(t::SM);
                        ui.checkbox(&mut draft.reduce_motion, lang.tr(K::MotionLabel));
                        ui.add_space(t::LG);
                        section(ui, lang.tr(K::BehaviorSection), p);
                        ui.label(lang.tr(K::RestartLabel));
                        egui::ComboBox::from_id_salt("restart-mode")
                            .width(320.0)
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
                        ui.add_space(t::SM);
                        ui.checkbox(&mut draft.refresh_on_focus, lang.tr(K::RefreshFocus));
                        ui.label(t::text(lang.tr(K::SafetyHint), 12.0, p.muted));
                        ui.add_space(t::LG);
                        section(ui, lang.tr(K::CavalrySection), p);
                        ui.label(lang.tr(K::AppPathLabel));
                        ui.add(
                            egui::TextEdit::singleline(&mut draft.cavalry_path)
                                .id_salt("cavalry-path")
                                .hint_text(lang.tr(K::AutoDetect))
                                .desired_width(f32::INFINITY)
                                .margin(egui::Margin::symmetric(8, 8)),
                        );
                        if t::button(ui, lang.tr(K::Browse), t::Button::Secondary, p).clicked() {
                            browse = true;
                        }
                        ui.label(t::text(lang.tr(K::PathHint), 12.0, p.muted));
                        ui.add_space(t::MD);
                        ui.label(t::text(lang.tr(K::SettingsLocal), 12.0, p.muted));
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
fn section(ui: &mut egui::Ui, label: &str, p: t::Palette) {
    ui.label(t::heading(label, 15.0, p));
    ui.add_space(t::XS);
}
fn hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

// ComboBox is a popup, not an egui menu. Explicitly close it for keyboard/AccessKit
// activation too, where no pointer clicked_elsewhere event is generated.
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
