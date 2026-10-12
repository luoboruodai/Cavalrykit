//! Explicit, cancellable, graceful restart. Never kill a process or bypass save prompts.
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningApp {
    pub pid: u32,
    pub path: PathBuf,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartError {
    NotFound,
    Multiple,
    InvalidPath,
    QuitDeclined,
    Timeout,
    Cancelled,
    System(String),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartResult {
    LaunchRequested,
    AlreadyRunning,
}

pub trait Backend {
    fn running(&mut self) -> Result<Vec<RunningApp>, RestartError>;
    fn locate(&mut self, configured: &str) -> Result<PathBuf, RestartError>;
    fn validate(&mut self, path: &Path) -> Result<(), RestartError>;
    fn request_quit(&mut self, app: &RunningApp) -> Result<(), RestartError>;
    fn launch(&mut self, path: &Path) -> Result<(), RestartError>;
}

/// Public for deterministic safety tests; tests use a fake backend and never touch Cavalry.
pub fn run_with<B: Backend>(
    backend: &mut B,
    configured: &str,
    allow_quit: bool,
    cancel: &AtomicBool,
    max_polls: usize,
    mut wait: impl FnMut(),
) -> Result<RestartResult, RestartError> {
    if cancel.load(Ordering::SeqCst) {
        return Err(RestartError::Cancelled);
    }
    let apps = backend.running()?;
    if !allow_quit && !apps.is_empty() {
        return Ok(RestartResult::AlreadyRunning);
    }
    if apps.len() > 1 {
        return Err(RestartError::Multiple);
    }
    let path = if let Some(app) = apps.first() {
        app.path.clone()
    } else {
        backend.locate(configured)?
    };
    // Verify the launch target before asking a running application to quit.
    backend.validate(&path)?;
    if let Some(app) = apps.first() {
        if cancel.load(Ordering::SeqCst) {
            return Err(RestartError::Cancelled);
        }
        backend.request_quit(app)?;
        let mut exited = false;
        for _ in 0..max_polls {
            if cancel.load(Ordering::SeqCst) {
                return Err(RestartError::Cancelled);
            }
            let current = backend.running()?;
            if !current.iter().any(|p| p.pid == app.pid) {
                exited = true;
                break;
            }
            wait();
        }
        if !exited {
            return Err(RestartError::Timeout);
        }
    }
    if cancel.load(Ordering::SeqCst) {
        return Err(RestartError::Cancelled);
    }
    // Someone may have restarted Cavalry manually while we were waiting. Never launch twice.
    if !backend.running()?.is_empty() {
        return Ok(RestartResult::AlreadyRunning);
    }
    backend.launch(&path)?;
    Ok(RestartResult::LaunchRequested)
}

pub fn perform(
    configured: &str,
    allow_quit: bool,
    cancel: Arc<AtomicBool>,
) -> Result<RestartResult, RestartError> {
    run_with(&mut Native, configured, allow_quit, &cancel, 120, || {
        std::thread::sleep(Duration::from_millis(500))
    })
}
struct Native;

#[cfg(target_os = "macos")]
impl Backend for Native {
    fn running(&mut self) -> Result<Vec<RunningApp>, RestartError> {
        use objc2_app_kit::NSRunningApplication;
        use objc2_foundation::NSString;
        let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(
            &NSString::from_str("com.scenegroup.cavalry"),
        );
        let mut found = Vec::new();
        for app in apps.iter() {
            if app.isTerminated() {
                continue;
            }
            let path = app
                .bundleURL()
                .and_then(|u| u.path())
                .map(|p| PathBuf::from(p.to_string()))
                .ok_or(RestartError::InvalidPath)?;
            found.push(RunningApp {
                pid: app.processIdentifier() as u32,
                path,
            });
        }
        Ok(found)
    }
    fn locate(&mut self, configured: &str) -> Result<PathBuf, RestartError> {
        if !configured.trim().is_empty() {
            return Ok(configured.into());
        }
        let mut paths = vec![PathBuf::from("/Applications/Cavalry.app")];
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Applications/Cavalry.app"));
        }
        paths
            .into_iter()
            .find(|p| p.is_dir())
            .ok_or(RestartError::NotFound)
    }
    fn validate(&mut self, path: &Path) -> Result<(), RestartError> {
        if !path.is_dir() || path.extension().and_then(|s| s.to_str()) != Some("app") {
            return Err(RestartError::InvalidPath);
        }
        let out = std::process::Command::new("/usr/bin/plutil")
            .args(["-extract", "CFBundleIdentifier", "raw", "-o", "-"])
            .arg(path.join("Contents/Info.plist"))
            .output()
            .map_err(|e| RestartError::System(e.to_string()))?;
        if !out.status.success()
            || String::from_utf8_lossy(&out.stdout).trim() != "com.scenegroup.cavalry"
        {
            return Err(RestartError::InvalidPath);
        }
        Ok(())
    }
    fn request_quit(&mut self, target: &RunningApp) -> Result<(), RestartError> {
        use objc2_app_kit::NSRunningApplication;
        use objc2_foundation::NSString;
        let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(
            &NSString::from_str("com.scenegroup.cavalry"),
        );
        for app in apps.iter() {
            let path = app
                .bundleURL()
                .and_then(|u| u.path())
                .map(|p| PathBuf::from(p.to_string()));
            if app.processIdentifier() as u32 == target.pid && path.as_ref() == Some(&target.path) {
                return if app.terminate() {
                    Ok(())
                } else {
                    Err(RestartError::QuitDeclined)
                };
            }
        }
        Err(RestartError::QuitDeclined)
    }
    fn launch(&mut self, path: &Path) -> Result<(), RestartError> {
        let status = std::process::Command::new("/usr/bin/open")
            .arg("-a")
            .arg(path)
            .status()
            .map_err(|e| RestartError::System(e.to_string()))?;
        if status.success() {
            Ok(())
        } else {
            Err(RestartError::System("Launch request rejected".into()))
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, BOOL, HWND, INVALID_HANDLE_VALUE, LPARAM},
        System::{Diagnostics::ToolHelp::*, Threading::*},
        UI::WindowsAndMessaging::*,
    };
    fn process_path(pid: u32) -> Result<PathBuf, RestartError> {
        // SAFETY: checked process handle and sized UTF-16 buffer; handle closed on every path.
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return Err(RestartError::System(
                    "Cannot inspect Cavalry process".into(),
                ));
            }
            let mut chars = vec![0u16; 32768];
            let mut len = chars.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, 0, chars.as_mut_ptr(), &mut len);
            CloseHandle(handle);
            if ok == 0 {
                return Err(RestartError::System("Cannot locate running Cavalry".into()));
            }
            use std::os::windows::ffi::OsStringExt;
            Ok(std::ffi::OsString::from_wide(&chars[..len as usize]).into())
        }
    }
    impl Backend for Native {
        fn running(&mut self) -> Result<Vec<RunningApp>, RestartError> {
            unsafe {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
                if snapshot == INVALID_HANDLE_VALUE {
                    return Err(RestartError::System("Cannot enumerate processes".into()));
                }
                let mut entry: PROCESSENTRY32W = std::mem::zeroed();
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                let mut more = Process32FirstW(snapshot, &mut entry);
                let mut pids = Vec::new();
                while more != 0 {
                    let len = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    if String::from_utf16_lossy(&entry.szExeFile[..len])
                        .eq_ignore_ascii_case("Cavalry.exe")
                    {
                        pids.push(entry.th32ProcessID);
                    }
                    more = Process32NextW(snapshot, &mut entry);
                }
                CloseHandle(snapshot);
                pids.into_iter()
                    .map(|pid| process_path(pid).map(|path| RunningApp { pid, path }))
                    .collect()
            }
        }
        fn locate(&mut self, configured: &str) -> Result<PathBuf, RestartError> {
            if !configured.trim().is_empty() {
                return Ok(configured.into());
            }
            let mut candidates = Vec::new();
            for variable in ["ProgramFiles", "ProgramW6432"] {
                if let Some(base) = std::env::var_os(variable) {
                    candidates.push(PathBuf::from(base).join("Cavalry/Cavalry.exe"));
                }
            }
            if let Some(base) = std::env::var_os("LOCALAPPDATA") {
                candidates.push(PathBuf::from(base).join("Programs/Cavalry/Cavalry.exe"));
            }
            candidates
                .into_iter()
                .find(|p| p.is_file())
                .ok_or(RestartError::NotFound)
        }
        fn validate(&mut self, path: &Path) -> Result<(), RestartError> {
            if path.is_file()
                && path
                    .file_name()
                    .is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case("Cavalry.exe"))
            {
                Ok(())
            } else {
                Err(RestartError::InvalidPath)
            }
        }
        fn request_quit(&mut self, app: &RunningApp) -> Result<(), RestartError> {
            if process_path(app.pid)? != app.path {
                return Err(RestartError::QuitDeclined);
            }
            struct Query {
                pid: u32,
                posted: bool,
            }
            unsafe extern "system" fn visit(hwnd: HWND, param: LPARAM) -> BOOL {
                let query = unsafe { &mut *(param as *mut Query) };
                let mut pid = 0;
                unsafe {
                    GetWindowThreadProcessId(hwnd, &mut pid);
                }
                if pid == query.pid
                    && unsafe { IsWindowVisible(hwnd) } != 0
                    && unsafe { GetWindow(hwnd, GW_OWNER) }.is_null()
                {
                    query.posted |= unsafe { PostMessageW(hwnd, WM_CLOSE, 0, 0) } != 0;
                }
                1
            }
            let mut query = Query {
                pid: app.pid,
                posted: false,
            };
            // WM_CLOSE gives Cavalry ownership of save/cancel prompts. No termination API is used.
            unsafe {
                EnumWindows(Some(visit), &mut query as *mut Query as LPARAM);
            }
            if query.posted {
                Ok(())
            } else {
                Err(RestartError::QuitDeclined)
            }
        }
        fn launch(&mut self, path: &Path) -> Result<(), RestartError> {
            std::process::Command::new(path)
                .spawn()
                .map(|_| ())
                .map_err(|e| RestartError::System(e.to_string()))
        }
    }
}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl Backend for Native {
    fn running(&mut self) -> Result<Vec<RunningApp>, RestartError> {
        Err(RestartError::NotFound)
    }
    fn locate(&mut self, _: &str) -> Result<PathBuf, RestartError> {
        Err(RestartError::NotFound)
    }
    fn validate(&mut self, _: &Path) -> Result<(), RestartError> {
        Err(RestartError::NotFound)
    }
    fn request_quit(&mut self, _: &RunningApp) -> Result<(), RestartError> {
        Err(RestartError::NotFound)
    }
    fn launch(&mut self, _: &Path) -> Result<(), RestartError> {
        Err(RestartError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    struct Fake {
        states: VecDeque<Vec<RunningApp>>,
        quit: usize,
        launch: Vec<PathBuf>,
        valid: bool,
        refuse: bool,
    }
    impl Backend for Fake {
        fn running(&mut self) -> Result<Vec<RunningApp>, RestartError> {
            Ok(if self.states.len() > 1 {
                self.states.pop_front().unwrap()
            } else {
                self.states.front().cloned().unwrap_or_default()
            })
        }
        fn locate(&mut self, p: &str) -> Result<PathBuf, RestartError> {
            Ok(p.into())
        }
        fn validate(&mut self, _: &Path) -> Result<(), RestartError> {
            if self.valid {
                Ok(())
            } else {
                Err(RestartError::InvalidPath)
            }
        }
        fn request_quit(&mut self, _: &RunningApp) -> Result<(), RestartError> {
            self.quit += 1;
            if self.refuse {
                Err(RestartError::QuitDeclined)
            } else {
                Ok(())
            }
        }
        fn launch(&mut self, p: &Path) -> Result<(), RestartError> {
            self.launch.push(p.into());
            Ok(())
        }
    }
    fn app() -> RunningApp {
        RunningApp {
            pid: 7,
            path: "original/Cavalry".into(),
        }
    }
    fn fake(states: Vec<Vec<RunningApp>>) -> Fake {
        Fake {
            states: states.into(),
            quit: 0,
            launch: vec![],
            valid: true,
            refuse: false,
        }
    }
    #[test]
    fn unsaved_or_cancelled_quit_never_forces_or_relaunches() {
        let mut b = fake(vec![vec![app()]]);
        assert_eq!(
            run_with(&mut b, "different", true, &AtomicBool::new(false), 3, || {}),
            Err(RestartError::Timeout)
        );
        assert_eq!(b.quit, 1);
        assert!(b.launch.is_empty());
    }
    #[test]
    fn reopens_original_instance_not_another_configured_version() {
        let mut b = fake(vec![vec![app()], vec![]]);
        assert_eq!(
            run_with(&mut b, "different", true, &AtomicBool::new(false), 3, || {}),
            Ok(RestartResult::LaunchRequested)
        );
        assert_eq!(b.launch, vec![app().path]);
    }
    #[test]
    fn multiple_instances_and_bad_paths_do_not_send_quit() {
        let mut b = fake(vec![vec![app(), app()]]);
        assert_eq!(
            run_with(&mut b, "", true, &AtomicBool::new(false), 2, || {}),
            Err(RestartError::Multiple)
        );
        assert_eq!(b.quit, 0);
        let mut b = fake(vec![vec![app()]]);
        b.valid = false;
        assert_eq!(
            run_with(&mut b, "", true, &AtomicBool::new(false), 2, || {}),
            Err(RestartError::InvalidPath)
        );
        assert_eq!(b.quit, 0);
    }
    #[test]
    fn cancellation_and_external_relaunch_prevent_duplicate_launch() {
        let cancel = AtomicBool::new(false);
        let mut b = fake(vec![vec![app()]]);
        assert_eq!(
            run_with(&mut b, "", true, &cancel, 3, || cancel
                .store(true, Ordering::SeqCst)),
            Err(RestartError::Cancelled)
        );
        assert!(b.launch.is_empty());
        let other = RunningApp { pid: 8, ..app() };
        let mut b = fake(vec![vec![app()], vec![other]]);
        assert_eq!(
            run_with(&mut b, "", true, &AtomicBool::new(false), 3, || {}),
            Ok(RestartResult::AlreadyRunning)
        );
        assert!(b.launch.is_empty());
    }
    #[test]
    fn launch_only_intent_never_quits_a_newly_running_instance() {
        let mut b = fake(vec![vec![app()]]);
        assert_eq!(
            run_with(&mut b, "", false, &AtomicBool::new(false), 2, || {}),
            Ok(RestartResult::AlreadyRunning)
        );
        assert_eq!(b.quit, 0);
        assert!(b.launch.is_empty());
    }
}
