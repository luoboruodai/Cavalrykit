//! 已安装内容扫描与管理（列表 / 卸载 / 作者解析）。

use std::fs;
use std::path::{Path, PathBuf};

use crate::detector::is_script_file;
use crate::paths::CavalryPaths;

#[derive(Debug, Clone)]
pub struct ScriptEntry {
    pub name: String,
    /// 所在子目录（相对 Scripts 根），根目录下为 None
    pub group: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PluginEntry {
    pub name: String,
    pub author: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Default)]
pub struct Library {
    pub scripts: Vec<ScriptEntry>,
    pub plugins: Vec<PluginEntry>,
}

/// 扫描 Cavalry 的脚本目录与插件目录。
pub fn scan(paths: &CavalryPaths) -> Library {
    let mut lib = Library::default();
    if paths.scripts.is_dir() {
        walk_scripts(&paths.scripts, &paths.scripts, &mut lib.scripts);
    }
    if paths.plugins.is_dir() {
        walk_plugins(&paths.plugins, &mut lib.plugins);
    }
    lib.scripts.sort_by(|a, b| a.group.cmp(&b.group).then(a.name.cmp(&b.name)));
    lib.plugins.sort_by(|a, b| a.name.cmp(&b.name));
    lib
}

fn walk_scripts(dir: &Path, root: &Path, out: &mut Vec<ScriptEntry>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.filter_map(|e| e.ok()) {
        let p = entry.path();
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if crate::unzipper::is_junk_name(name) {
            continue;
        }
        let Ok(meta) = fs::symlink_metadata(&p) else { continue };
        if meta.is_dir() {
            walk_scripts(&p, root, out);
        } else if is_script_file(&p) {
            let group = p
                .parent()
                .and_then(|par| par.strip_prefix(root).ok())
                .filter(|rel| !rel.as_os_str().is_empty())
                .map(|rel| rel.to_string_lossy().into_owned());
            out.push(ScriptEntry {
                name: name.to_string(),
                group,
                path: p,
            });
        }
    }
}

/// 插件按「产品」列出：直接包含 definitions.json 的目录即一个产品，
/// 命中后不再深入（Cavalry 允许插件任意嵌套打包）。
fn walk_plugins(dir: &Path, out: &mut Vec<PluginEntry>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.filter_map(|e| e.ok()) {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        if crate::unzipper::is_junk_name(&entry.file_name().to_string_lossy()) {
            continue;
        }
        if p.join("definitions.json").is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            out.push(PluginEntry {
                name,
                author: plugin_author(&p),
                path: p,
            });
        } else {
            walk_plugins(&p, out);
        }
    }
}

/// 解析插件 definitions.json 中的 author 字段（容错：任何解析失败都返回 None）。
pub fn plugin_author(dir: &Path) -> Option<String> {
    let text = fs::read_to_string(dir.join("definitions.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("author")?.as_str().map(|s| s.to_string())
}

/// 卸载：安全校验目标必须位于 Cavalry 的 Scripts 或 Plugins 目录内，
/// 然后移入系统废纸篓 / 回收站（可恢复）。
pub fn uninstall(paths: &CavalryPaths, target: &Path) -> Result<(), String> {
    let t = target
        .canonicalize()
        .map_err(|e| format!("目标路径无效: {e}"))?;
    let inside_scripts = paths
        .scripts
        .canonicalize()
        .map(|s| t.starts_with(s))
        .unwrap_or(false);
    let inside_plugins = paths
        .plugins
        .canonicalize()
        .map(|p| t.starts_with(p))
        .unwrap_or(false);
    if !inside_scripts && !inside_plugins {
        return Err("拒绝删除 Cavalry 目录之外的路径".to_string());
    }
    trash::delete(target).map_err(|e| format!("移入废纸篓失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "cavalrykit-test-library-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn scan_lists_scripts_with_groups_and_plugins_with_authors() {
        let dir = tmpdir("scan");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        paths.ensure_dirs().unwrap();

        std::fs::write(paths.scripts.join("Renamer.jsc"), b"").unwrap();
        std::fs::create_dir_all(paths.scripts.join("deformers")).unwrap();
        std::fs::write(paths.scripts.join("deformers/bend.js"), b"").unwrap();
        std::fs::write(paths.scripts.join("notes.txt"), b"ignored").unwrap();

        let plug = paths.plugins.join("Field Kit");
        std::fs::create_dir_all(&plug).unwrap();
        std::fs::write(
            plug.join("definitions.json"),
            br#"{"author": "samMularczyk"}"#,
        )
        .unwrap();

        let lib = scan(&paths);
        assert_eq!(lib.scripts.len(), 2);
        let root_script = lib.scripts.iter().find(|s| s.name == "Renamer.jsc").unwrap();
        assert_eq!(root_script.group, None);
        let sub_script = lib.scripts.iter().find(|s| s.name == "bend.js").unwrap();
        assert_eq!(sub_script.group.as_deref(), Some("deformers"));

        assert_eq!(lib.plugins.len(), 1);
        assert_eq!(lib.plugins[0].name, "Field Kit");
        assert_eq!(lib.plugins[0].author.as_deref(), Some("samMularczyk"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn uninstall_rejects_paths_outside_cavalry_dirs() {
        let dir = tmpdir("guard");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        paths.ensure_dirs().unwrap();
        let outsider = dir.join("important.txt");
        std::fs::write(&outsider, b"do not delete").unwrap();
        let r = uninstall(&paths, &outsider);
        assert!(r.is_err());
        assert!(outsider.exists()); // 文件必须还在
        let _ = std::fs::remove_dir_all(&dir);
    }
}
