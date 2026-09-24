// Windows 下不弹控制台黑窗(GUI 子系统)
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() -> eframe::Result<()> {
    cavalrykit::run()
}
