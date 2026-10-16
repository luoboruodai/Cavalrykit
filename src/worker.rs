//! A single serialized I/O worker. No scanning, extraction, copying or process probes in update().
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::time::{Duration, Instant};

use crate::{
    installer::{self, ConflictChoice, InstallPlan, Outcome, PlannedUnit},
    library::{self, Library},
    paths::{self, CavalryPaths},
};

pub enum Job {
    Scan,
    Plan(Vec<PathBuf>),
    Install(InstallPlan, Vec<(PlannedUnit, ConflictChoice)>),
    Uninstall(PathBuf),
    SaveSettings(PathBuf, crate::settings::Settings),
    Restart(String, bool, std::sync::Arc<std::sync::atomic::AtomicBool>),
    Discard(InstallPlan),
}

pub enum Event {
    Library(Library),
    Planned(InstallPlan),
    Installed(Outcome, Library),
    Uninstalled(Result<(), String>, Library),
    Running(bool),
    SettingsSaved(Result<(), String>),
    Restarted(Result<crate::restart::RestartResult, crate::restart::RestartError>),
    Discarded,
    Failed(String),
}

pub struct Worker {
    jobs: SyncSender<Job>,
    events: Receiver<Event>,
}

impl Worker {
    pub fn spawn(paths: CavalryPaths, wake: impl Fn() + Send + 'static) -> Self {
        let (jobs, requests) = mpsc::sync_channel(4);
        let (results, events) = mpsc::channel();
        std::thread::Builder::new()
            .name("cavalrykit-io".into())
            .spawn(move || {
                let emit = |event| {
                    let sent = results.send(event).is_ok();
                    if sent {
                        wake();
                    }
                    sent
                };
                let mut running = None;
                let interval = Duration::from_secs(5);
                let mut next_probe = Instant::now();
                loop {
                    if Instant::now() >= next_probe {
                        let value = paths::is_cavalry_running();
                        if running != Some(value) && !emit(Event::Running(value)) {
                            break;
                        }
                        running = Some(value);
                        next_probe = Instant::now() + interval;
                    }
                    let job = match requests
                        .recv_timeout(next_probe.saturating_duration_since(Instant::now()))
                    {
                        Ok(job) => job,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                    // The temporary archive guard stays on this thread through copying and cleanup.
                    let event =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match job {
                            Job::SaveSettings(path, settings) => {
                                Event::SettingsSaved(settings.save(&path))
                            }
                            Job::Restart(path, allow_quit, cancel) => {
                                Event::Restarted(crate::restart::perform(&path, allow_quit, cancel))
                            }
                            Job::Discard(plan) => {
                                drop(plan);
                                Event::Discarded
                            }
                            Job::Scan => Event::Library(library::scan(&paths)),
                            Job::Plan(files) => Event::Planned(installer::plan(&files, &paths)),
                            Job::Install(mut plan, resolved) => {
                                let mut outcome =
                                    installer::execute(std::mem::take(&mut plan.units), resolved);
                                outcome.duplicates.append(&mut plan.duplicates);
                                outcome.failures.append(&mut plan.failures);
                                drop(plan);
                                Event::Installed(outcome, library::scan(&paths))
                            }
                            Job::Uninstall(target) => Event::Uninstalled(
                                library::uninstall(&paths, &target),
                                library::scan(&paths),
                            ),
                        }))
                        .unwrap_or_else(|_| {
                            Event::Failed("后台操作异常，未完成的操作请重新检查。".into())
                        });
                    if matches!(event, Event::Restarted(_)) {
                        next_probe = Instant::now();
                    }
                    if !emit(event) {
                        break;
                    }
                }
            })
            .expect("start CavalryKit I/O worker");
        Self { jobs, events }
    }

    pub fn submit(&self, job: Job) -> Result<(), String> {
        self.jobs
            .try_send(job)
            .map_err(|_| "后台任务不可用或队列已满，请稍后重试。".into())
    }

    pub fn try_recv(&self) -> Result<Event, TryRecvError> {
        self.events.try_recv()
    }
}
