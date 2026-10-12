//! 双平台路径与系统命令适配（macOS / Windows 的全部差异集中于此模块）。

use std::path::{Path, PathBuf};
use std::process::Command;

/// Cavalry 的两个用户目录（脚本 / 第三方插件）。
#[derive(Debug, Clone)]
pub struct CavalryPaths {
    pub scripts: PathBuf,
    pub plugins: PathBuf,
}

impl CavalryPaths {
    /// 依据当前平台定位真实的 Cavalry 目录。
    ///
    /// macOS:   `~/Library/Application Support/Cavalry`
    /// Windows: `%APPDATA%\Cavalry`
    ///
    /// `dirs::config_dir()` 在两个平台上恰好对应上述位置。
    pub fn detect() -> Option<CavalryPaths> {
        let base = dirs::config_dir()?.join("Cavalry");
        Some(CavalryPaths::from_base(&base))
    }

    /// 由 Cavalry 根目录派生两个子目录（测试注入也走这里）。
    pub fn from_base(base: &Path) -> CavalryPaths {
        CavalryPaths {
            scripts: base.join("Scripts"),
            plugins: base.join("Third-Party").join("Plugins"),
        }
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.scripts)?;
        std::fs::create_dir_all(&self.plugins)
    }
}

#[cfg(target_os = "macos")]
pub fn is_cavalry_running() -> bool {
    Command::new("pgrep")
        .args(["-x", "Cavalry"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
pub fn is_cavalry_running() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    // Native enumeration: no tasklist subprocess, hidden console, or UI-thread wait.
    // SAFETY: the snapshot is checked and closed on every path; dwSize is initialized.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut more = Process32FirstW(snapshot, &mut entry);
        let mut found = false;
        while more != 0 {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..len]).eq_ignore_ascii_case("Cavalry.exe")
            {
                found = true;
                break;
            }
            more = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        found
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn is_cavalry_running() -> bool {
    false
}

/// Stable taskbar identity, independent of imported script paths or working directory.
pub fn setup_app_identity() {
    #[cfg(target_os = "windows")]
    {
        let id: Vec<u16> = "com.cavalrykit.app\0".encode_utf16().collect();
        // SAFETY: id is a valid, null-terminated UTF-16 string for the duration of the call.
        unsafe {
            windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
        }
    }
}

/// 在文件管理器中定位并选中该文件/文件夹。
pub fn reveal_in_file_manager(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg("-R").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn();
    }
}

/// 用文件管理器打开目录。
pub fn open_dir(dir: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(dir).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(dir).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_base_appends_subdirs() {
        // 用 Windows 风格路径做纯字符串层面的验证（任何平台都适用）
        let base = Path::new(r"C:\Users\test\AppData\Roaming\Cavalry");
        let p = CavalryPaths::from_base(base);
        assert!(p.scripts.ends_with("Scripts"));
        assert!(p.plugins.ends_with("Plugins"));
        assert!(p.plugins.to_string_lossy().contains("Third-Party"));
    }

    #[test]
    fn from_base_mac_style() {
        let base = Path::new("/Users/test/Library/Application Support/Cavalry");
        let p = CavalryPaths::from_base(base);
        assert_eq!(p.scripts, base.join("Scripts"));
        assert_eq!(p.plugins, base.join("Third-Party").join("Plugins"));
    }
}
