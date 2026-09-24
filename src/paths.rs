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
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq Cavalry.exe", "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Cavalry.exe"))
        .unwrap_or(false)
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
