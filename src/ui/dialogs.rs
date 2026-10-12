use super::{app::CavalryKitApp, theme as t};
use crate::{i18n::Key as K, installer::ConflictChoice, worker::Job};
use egui::Vec2;
use std::sync::atomic::Ordering;
impl CavalryKitApp {
    pub(crate) fn show_conflict_dialog(&mut self, ctx: &egui::Context) {
        let p = self.p();
        let lang = self.prefs().language;
        let unit = self.conflicts.front().unwrap();
        let name = unit.display.clone();
        let dest = unit.dest.to_string_lossy().into_owned();
        let remaining = self.conflicts.len();
        let mut choice = None;
        let mut cancel = false;
        let response = t::modal(ctx, "conflict", p, 576.0, |ui| {
            ui.label(t::heading(lang.tr(K::ConflictTitle), 22.0, p));
            ui.add_space(t::SM);
            ui.label(lang.tr(K::ConflictHint));
            ui.add_space(t::MD);
            ui.add(egui::Label::new(t::heading(name, 16.0, p)).wrap());
            egui::CollapsingHeader::new(lang.tr(K::ExistingPath)).show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(t::text(dest, 12.0, p.muted)).wrap());
                    });
            });
            ui.add_space(t::MD);
            ui.label(t::text(
                lang.tr(K::ConflictsWaiting)
                    .replace("{n}", &remaining.to_string()),
                12.0,
                p.muted,
            ));
            if remaining > 1 {
                t::checkbox(ui, &mut self.apply_all, lang.tr(K::ApplyAll), p);
            }
            t::footer(ui, |ui| {
                let r = t::button(ui, lang.tr(K::Replace), t::Button::Danger, p);
                self.record(K::Replace, &r);
                if r.clicked() {
                    choice = Some(ConflictChoice::Overwrite);
                }
                let r = t::button(ui, lang.tr(K::KeepBoth), t::Button::Secondary, p);
                self.record(K::KeepBoth, &r);
                if r.clicked() {
                    choice = Some(ConflictChoice::KeepBoth);
                }
                let r = t::button(ui, lang.tr(K::Skip), t::Button::Secondary, p);
                self.record(K::Skip, &r);
                if r.clicked() {
                    choice = Some(ConflictChoice::Skip);
                }
                let r = t::button(ui, lang.tr(K::CancelBatch), t::Button::Quiet, p);
                self.record(K::CancelBatch, &r);
                if r.clicked() {
                    cancel = true;
                }
            });
        });
        self.audit.dialog = Some(response.response.rect);
        // Escape/backdrop means cancel this batch, never accept an overwrite.
        if cancel || response.should_close() {
            self.cancel_batch();
        } else if let Some(choice) = choice {
            self.resolve_current(choice);
        }
    }
    pub(crate) fn show_result_dialog(&mut self, ctx: &egui::Context) {
        let p = self.p();
        let lang = self.prefs().language;
        let outcome = self.outcome.as_ref().unwrap();
        let (ok, skipped, failed) = (
            outcome.installed_scripts.len() + outcome.installed_plugins.len(),
            outcome.skipped.len(),
            outcome.failures.len(),
        );
        let count = ok + skipped + failed;
        let pages = count.div_ceil(12).max(1);
        self.result_page = self.result_page.min(pages - 1);
        let page = self.result_page;
        let title = if failed == 0 {
            K::ResultTitle
        } else {
            K::ResultIssue
        };
        let summary = lang
            .tr(K::ResultSummary)
            .replace("{ok}", &ok.to_string())
            .replace("{skip}", &skipped.to_string())
            .replace("{fail}", &failed.to_string());
        let restart_hint = outcome.restart_hint;
        // Only materialize the current page so large imports never create unbounded result UI work.
        let mut rows = Vec::new();
        for i in page * 12..((page + 1) * 12).min(count) {
            let scripts = outcome.installed_scripts.len();
            let plugins = outcome.installed_plugins.len();
            if i < scripts {
                rows.push((K::Installed, outcome.installed_scripts[i].clone(), None));
            } else if i < scripts + plugins {
                rows.push((
                    K::Installed,
                    outcome.installed_plugins[i - scripts].clone(),
                    None,
                ));
            } else if i < ok + skipped {
                rows.push((K::Skipped, outcome.skipped[i - ok].clone(), None));
            } else {
                let (name, error) = &outcome.failures[i - ok - skipped];
                rows.push((K::Failed, name.clone(), Some(error.clone())));
            }
        }
        let mut close = false;
        let response = t::modal(ctx, "result", p, 576.0, |ui| {
            ui.label(t::heading(lang.tr(title), 22.0, p));
            ui.add_space(t::SM);
            ui.label(t::text(summary, 14.0, p.muted));
            ui.add_space(t::MD);
            egui::ScrollArea::vertical()
                .id_salt("result-list")
                .max_height((ctx.screen_rect().height() - 280.0).clamp(96.0, 320.0))
                .show(ui, |ui| {
                    for (status, name, error) in rows {
                        let color = match status {
                            K::Installed => p.success,
                            K::Failed => p.danger,
                            _ => p.muted,
                        };
                        ui.label(t::text(lang.tr(status), 12.0, color));
                        ui.add(egui::Label::new(t::text(name, 14.0, p.text)).wrap());
                        if let Some(detail) = error {
                            ui.add(
                                egui::Label::new(t::text(lang.tr(K::ErrorAdvice), 12.0, p.muted))
                                    .wrap(),
                            );
                            egui::CollapsingHeader::new(lang.tr(K::TechnicalDetails))
                                .id_salt(ui.next_auto_id())
                                .show(ui, |ui| {
                                    ui.add(egui::Label::new(t::text(detail, 12.0, p.muted)).wrap());
                                });
                        }
                        ui.add_space(t::SM);
                    }
                });
            if pages > 1 {
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(page > 0, |ui| {
                        if t::button(ui, lang.tr(K::Previous), t::Button::Quiet, p).clicked() {
                            self.result_page -= 1;
                        }
                    });
                    ui.label(t::text(
                        lang.tr(K::Page)
                            .replace("{page}", &(page + 1).to_string())
                            .replace("{total}", &pages.to_string()),
                        12.0,
                        p.muted,
                    ));
                    ui.add_enabled_ui(page + 1 < pages, |ui| {
                        if t::button(ui, lang.tr(K::Next), t::Button::Quiet, p).clicked() {
                            self.result_page += 1;
                        }
                    });
                });
            }
            if restart_hint {
                ui.add_space(t::SM);
                ui.label(t::text(lang.tr(K::PluginReload), 13.0, p.accent_text));
            }
            if !self.queue.is_empty() {
                ui.label(t::text(
                    lang.tr(K::Queued)
                        .replace("{n}", &self.queue.len().to_string()),
                    12.0,
                    p.muted,
                ));
            }
            t::footer(ui, |ui| {
                let key = if self.queue.is_empty() {
                    K::Done
                } else {
                    K::Next
                };
                let r = t::button(ui, lang.tr(key), t::Button::Primary, p);
                self.record(key, &r);
                close = r.clicked();
            });
        });
        self.audit.dialog = Some(response.response.rect);
        if close || response.should_close() {
            self.dismiss_outcome();
            ctx.request_repaint();
        }
    }
    pub(crate) fn show_restart_dialog(&mut self, ctx: &egui::Context) {
        let p = self.p();
        let lang = self.prefs().language;
        let running = self.cavalry_running;
        let mut go = false;
        let mut later = false;
        let response = t::modal(ctx, "restart-offer", p, 576.0, |ui| {
            ui.label(t::heading(
                lang.tr(if running {
                    K::RestartTitle
                } else {
                    K::LaunchTitle
                }),
                22.0,
                p,
            ));
            ui.add_space(t::SM);
            ui.label(lang.tr(K::RestartHint));
            ui.add_space(t::MD);
            if running {
                ui.add(egui::Label::new(t::text(lang.tr(K::SaveWarning), 14.0, p.warning)).wrap());
            } else {
                ui.label(t::text(lang.tr(K::ManualRestartHint), 13.0, p.muted));
            }
            t::footer(ui, |ui| {
                let key = if running { K::RestartNow } else { K::LaunchNow };
                let r = t::button(ui, lang.tr(key), t::Button::Primary, p);
                self.record(key, &r);
                go = r.clicked();
                let r = t::button(
                    ui,
                    lang.tr(if running { K::Later } else { K::LaterOpen }),
                    t::Button::Secondary,
                    p,
                );
                self.record(K::Later, &r);
                later = r.clicked();
            });
        });
        self.audit.dialog = Some(response.response.rect);
        if later || response.should_close() {
            self.decline_restart();
        } else if go {
            self.begin_restart(running);
        }
    }
    pub(crate) fn show_restart_progress(&mut self, ctx: &egui::Context) {
        let p = self.p();
        let lang = self.prefs().language;
        let mut cancel = false;
        let response = t::modal(ctx, "restart-progress", p, 576.0, |ui| {
            ui.label(t::heading(lang.tr(K::RestartBusy), 18.0, p));
            ui.add_space(t::MD);
            ui.label(lang.tr(K::RestartWaiting));
            ui.add_space(t::SM);
            ui.label(t::text(lang.tr(K::StopWaitingHint), 12.0, p.muted));
            t::footer(ui, |ui| {
                let r = t::button(ui, lang.tr(K::StopWaiting), t::Button::Secondary, p);
                self.record(K::StopWaiting, &r);
                cancel = r.clicked();
            });
        });
        self.audit.dialog = Some(response.response.rect);
        if cancel || response.should_close() {
            if let Some(token) = &self.restart_cancel {
                token.store(true, Ordering::SeqCst);
            }
        }
    }
    pub(crate) fn show_uninstall_dialog(&mut self, ctx: &egui::Context) {
        let p = self.p();
        let lang = self.prefs().language;
        let path = self.confirm_uninstall.as_ref().unwrap().clone();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let mut action = None;
        let response = t::modal(ctx, "uninstall", p, 528.0, |ui| {
            ui.label(t::heading(lang.tr(K::UninstallTitle), 22.0, p));
            ui.add_space(t::SM);
            ui.label(lang.tr(K::UninstallHint));
            ui.add_space(t::MD);
            ui.add(egui::Label::new(t::heading(name, 16.0, p)).wrap());
            t::footer(ui, |ui| {
                let r = t::button(ui, lang.tr(K::MoveTrash), t::Button::Danger, p);
                self.record(K::MoveTrash, &r);
                if r.clicked() {
                    action = Some(true);
                }
                let r = t::button(ui, lang.tr(K::Cancel), t::Button::Secondary, p);
                self.record(K::Cancel, &r);
                if r.clicked() {
                    action = Some(false);
                }
            });
        });
        self.audit.dialog = Some(response.response.rect);
        if action == Some(true) {
            self.confirm_uninstall = None;
            self.submit(Job::Uninstall(path), K::RemoveBusy);
        } else if action == Some(false) || response.should_close() {
            self.confirm_uninstall = None;
        }
    }
    pub(crate) fn show_about_dialog(&mut self, ctx: &egui::Context) {
        let p = self.p();
        let lang = self.prefs().language;
        let mut close = false;
        let response = t::modal(ctx, "about", p, 528.0, |ui| {
            ui.horizontal(|ui| {
                ui.image((self.logo.id(), Vec2::splat(40.0)));
                ui.label(t::heading("CavalryKit", 22.0, p));
            });
            ui.add_space(t::MD);
            ui.label(lang.tr(K::AboutText));
            ui.add_space(t::SM);
            ui.label(t::text(
                format!(
                    "v{} · {}",
                    env!("CARGO_PKG_VERSION"),
                    lang.tr(K::Disclaimer)
                ),
                12.0,
                p.muted,
            ));
            t::footer(ui, |ui| {
                let r = t::button(ui, lang.tr(K::Done), t::Button::Primary, p);
                self.record(K::Done, &r);
                close = r.clicked();
            });
        });
        self.audit.dialog = Some(response.response.rect);
        if close || response.should_close() {
            self.about_open = false;
        }
    }
    pub(crate) fn show_notice(&mut self, ctx: &egui::Context) {
        let p = self.p();
        let lang = self.prefs().language;
        let (key, detail) = self.notice.as_ref().unwrap().clone();
        let mut close = false;
        let response = t::modal(ctx, "notice", p, 576.0, |ui| {
            ui.label(t::heading(lang.tr(K::OperationError), 20.0, p));
            ui.add_space(t::MD);
            ui.label(lang.tr(key));
            if let Some(detail) = detail {
                egui::CollapsingHeader::new(lang.tr(K::TechnicalDetails)).show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(160.0)
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(t::text(detail, 12.0, p.muted)).wrap());
                        });
                });
            }
            t::footer(ui, |ui| {
                let r = t::button(ui, lang.tr(K::Done), t::Button::Primary, p);
                self.record(K::Done, &r);
                close = r.clicked();
            });
        });
        self.audit.dialog = Some(response.response.rect);
        if close || response.should_close() {
            self.notice = None;
        }
    }
}
