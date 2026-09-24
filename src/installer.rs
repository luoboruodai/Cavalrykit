//! 安装引擎：规划（识别输入 + 收集冲突）与执行（复制 + 冲突三策略）。

use std::fs;
use std::path::{Path, PathBuf};

use crate::detector;
use crate::paths::CavalryPaths;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    Script,
    Plugin,
}

impl UnitKind {
    pub fn label(self) -> &'static str {
        match self {
            UnitKind::Script => "脚本",
            UnitKind::Plugin => "插件",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlannedUnit {
    pub kind: UnitKind,
    pub display: String,
    pub source: PathBuf,
    pub dest: PathBuf,
}

/// 解压产物守卫：离开作用域时自动清理临时目录。
pub struct TempGuard(pub PathBuf);

impl Drop for TempGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub struct InstallPlan {
    /// 无冲突、可直接安装的条目
    pub units: Vec<PlannedUnit>,
    /// 目标已存在、需要用户选择处理的冲突
    pub conflicts: Vec<PlannedUnit>,
    /// (输入名, 失败原因)
    pub failures: Vec<(String, String)>,
    /// (下划线前缀：仅用于持有守卫，不主动读取)
    pub _temp: Option<TempGuard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictChoice {
    Overwrite,
    Skip,
    KeepBoth,
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub installed_scripts: Vec<String>,
    pub installed_plugins: Vec<String>,
    pub skipped: Vec<String>,
    pub failures: Vec<(String, String)>,
    /// 本次安装了插件 → 需要重启 Cavalry
    pub restart_hint: bool,
}

/// 规划安装：识别每个输入、映射目标路径、分离出冲突项。
pub fn plan(inputs: &[PathBuf], paths: &CavalryPaths) -> InstallPlan {
    let _ = paths.ensure_dirs();

    // 唯一临时目录:pid + 纳秒 + 原子序号三重防碰撞——并行/快速连续规划时
    // 若两次拿到同一目录,先结束的 TempGuard 会把另一个 plan 的解压产物删掉
    static PLAN_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = PLAN_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp = std::env::temp_dir().join(format!(
        "cavalrykit-plan-{}-{nanos}-{seq}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&temp);

    let mut units: Vec<PlannedUnit> = Vec::new();
    let mut failures: Vec<(String, String)> = Vec::new();

    for input in inputs {
        match detector::inspect(input, &temp, paths) {
            Ok(found) => units.extend(found),
            Err(e) => failures.push((input.display().to_string(), e)),
        }
    }

    let mut conflicts = Vec::new();
    units.retain(|u| {
        if u.dest.exists() {
            conflicts.push(u.clone());
            false
        } else {
            true
        }
    });

    InstallPlan {
        units,
        conflicts,
        failures,
        _temp: Some(TempGuard(temp)),
    }
}

/// 执行安装：普通条目直接复制，冲突条目按用户选择处理。
pub fn execute(units: Vec<PlannedUnit>, resolved: Vec<(PlannedUnit, ConflictChoice)>) -> Outcome {
    let mut outcome = Outcome::default();
    let mut to_copy: Vec<PlannedUnit> = units;

    for (unit, choice) in resolved {
        match choice {
            ConflictChoice::Overwrite => {
                let _ = remove_path(&unit.dest);
                to_copy.push(unit);
            }
            ConflictChoice::Skip => outcome.skipped.push(unit.display),
            ConflictChoice::KeepBoth => {
                let mut u = unit;
                u.dest = unique_dest(&u.dest);
                if let Some(new_name) = u.dest.file_name().map(|n| n.to_string_lossy().into_owned())
                {
                    u.display = new_name;
                }
                to_copy.push(u);
            }
        }
    }

    for unit in to_copy {
        match copy_unit(&unit) {
            Ok(()) => match unit.kind {
                UnitKind::Script => outcome.installed_scripts.push(unit.display),
                UnitKind::Plugin => {
                    outcome.installed_plugins.push(unit.display);
                    outcome.restart_hint = true;
                }
            },
            Err(e) => outcome.failures.push((unit.display, e)),
        }
    }

    outcome
}

fn remove_path(p: &Path) -> std::io::Result<()> {
    let meta = fs::symlink_metadata(p)?;
    if meta.is_dir() {
        fs::remove_dir_all(p)
    } else {
        fs::remove_file(p)
    }
}

fn copy_unit(unit: &PlannedUnit) -> Result<(), String> {
    match unit.kind {
        UnitKind::Script => {
            if let Some(parent) = unit.dest.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            fs::copy(&unit.source, &unit.dest)
                .map(|_| ())
                .map_err(|e| format!("复制失败: {e}"))
        }
        UnitKind::Plugin => copy_dir_all(&unit.source, &unit.dest),
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("创建插件目录失败: {e}"))?;
    for entry in
        fs::read_dir(src).map_err(|e| format!("读取插件目录失败: {e}"))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let meta = fs::symlink_metadata(&from).map_err(|e| e.to_string())?;
        if meta.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| format!("复制 {} 失败: {e}", from.display()))?;
        }
    }
    Ok(())
}

/// 生成不冲突的目标路径：`名称 2.jsc`、`Field Kit 2/` 这样的形式。
pub fn unique_dest(dest: &Path) -> PathBuf {
    let parent = dest.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = dest
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "未命名".into());
    let ext = dest.extension().map(|e| e.to_string_lossy().into_owned());

    for n in 2..100 {
        let name = match &ext {
            Some(e) if !e.is_empty() => format!("{stem} {n}.{e}"),
            _ => format!("{stem} {n}"),
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!("{stem}-{}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "cavalrykit-test-installer-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn unique_dest_appends_counter() {
        let dir = tmpdir("unique");
        let dest = dir.join("Renamer.jsc");
        fs::write(&dest, b"x").unwrap();
        let u = unique_dest(&dest);
        assert_eq!(u.file_name().unwrap(), "Renamer 2.jsc");
        fs::write(&u, b"x").unwrap();
        let u2 = unique_dest(&dest);
        assert_eq!(u2.file_name().unwrap(), "Renamer 3.jsc");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn execute_copies_scripts_and_flags_restart_for_plugins() {
        let dir = tmpdir("exec");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        paths.ensure_dirs().unwrap();

        let src_script = dir.join("a.jsc");
        fs::write(&src_script, b"// script").unwrap();
        let src_plugin = dir.join("MyPlugin");
        fs::create_dir_all(&src_plugin).unwrap();
        fs::write(src_plugin.join("definitions.json"), b"{}").unwrap();

        let plan = plan(&[src_script, src_plugin.clone()], &paths);
        assert!(plan.conflicts.is_empty());
        assert!(plan.failures.is_empty());
        assert_eq!(plan.units.len(), 2);

        let outcome = execute(plan.units, vec![]);
        assert_eq!(outcome.installed_scripts, vec!["a.jsc"]);
        assert_eq!(outcome.installed_plugins, vec!["MyPlugin"]);
        assert!(outcome.restart_hint);
        assert!(paths.scripts.join("a.jsc").is_file());
        assert!(paths.plugins.join("MyPlugin/definitions.json").is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn conflict_strategies_overwrite_skip_keepboth() {
        let dir = tmpdir("conflict");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        paths.ensure_dirs().unwrap();

        // 已存在的旧脚本
        fs::write(paths.scripts.join("old.jsc"), b"old").unwrap();

        let src = dir.join("old.jsc");
        fs::write(&src, b"new").unwrap();

        // 冲突 → 覆盖
        let plan1 = plan(&[src.clone()], &paths);
        assert_eq!(plan1.conflicts.len(), 1);
        assert!(plan1.units.is_empty());
        let c = plan1.conflicts[0].clone();
        let out = execute(vec![], vec![(c, ConflictChoice::Overwrite)]);
        assert!(out.installed_scripts.contains(&"old.jsc".to_string()));
        assert_eq!(fs::read_to_string(paths.scripts.join("old.jsc")).unwrap(), "new");

        // 冲突 → 跳过
        let plan2 = plan(&[src.clone()], &paths);
        let c = plan2.conflicts[0].clone();
        let out2 = execute(plan2.units, vec![(c, ConflictChoice::Skip)]);
        assert_eq!(out2.skipped, vec!["old.jsc"]);
        assert_eq!(fs::read_to_string(paths.scripts.join("old.jsc")).unwrap(), "new");

        // 冲突 → 保留两者
        let plan3 = plan(&[src], &paths);
        let c = plan3.conflicts[0].clone();
        let out3 = execute(plan3.units, vec![(c, ConflictChoice::KeepBoth)]);
        assert_eq!(out3.installed_scripts, vec!["old 2.jsc"]);
        assert_eq!(fs::read_to_string(paths.scripts.join("old.jsc")).unwrap(), "new");
        assert_eq!(
            fs::read_to_string(paths.scripts.join("old 2.jsc")).unwrap(),
            "new"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
