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
    /// 目标已存在且内容完全相同、无需打扰用户的重复导入
    pub duplicates: Vec<String>,
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
    /// 内容与已安装条目完全相同而跳过的重复导入
    pub duplicates: Vec<String>,
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
    let mut duplicates = Vec::new();
    let mut destinations: std::collections::HashMap<PathBuf, PathBuf> =
        std::collections::HashMap::new();
    units.retain(|u| {
        if u.dest.exists() {
            // 同名且内容完全相同 → 静默跳过；不同内容才交给用户做冲突三选。
            if payloads_equal(u.kind, &u.source, &u.dest) {
                duplicates.push(u.display.clone());
            } else {
                conflicts.push(u.clone());
            }
            return false;
        }
        match destinations.entry(u.dest.clone()) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(u.source.clone());
                true
            }
            std::collections::hash_map::Entry::Occupied(slot) => {
                if payloads_equal(u.kind, &u.source, slot.get()) {
                    duplicates.push(u.display.clone());
                } else {
                    conflicts.push(u.clone());
                }
                false
            }
        }
    });

    InstallPlan {
        units,
        conflicts,
        duplicates,
        failures,
        _temp: Some(TempGuard(temp)),
    }
}

/// 只有逐字节完全相同才自动跳过；读取失败时保留人工冲突确认。
fn payloads_equal(kind: UnitKind, left: &Path, right: &Path) -> bool {
    match kind {
        UnitKind::Script => fs::read(left)
            .ok()
            .zip(fs::read(right).ok())
            .is_some_and(|(a, b)| a == b),
        UnitKind::Plugin => dirs_equal(left, right, 0),
    }
}

fn dirs_equal(left: &Path, right: &Path, depth: usize) -> bool {
    if depth > 64 {
        return false;
    }
    let entries = |dir: &Path| fs::read_dir(dir).ok()?.collect::<Result<Vec<_>, _>>().ok();
    let (Some(mut left_entries), Some(mut right_entries)) = (entries(left), entries(right)) else {
        return false;
    };
    left_entries.sort_by_key(|entry| entry.file_name());
    right_entries.sort_by_key(|entry| entry.file_name());
    if left_entries.len() != right_entries.len() {
        return false;
    }
    for (left, right) in left_entries.iter().zip(&right_entries) {
        if left.file_name() != right.file_name() {
            return false;
        }
        let (Ok(left_meta), Ok(right_meta)) = (
            fs::symlink_metadata(left.path()),
            fs::symlink_metadata(right.path()),
        ) else {
            return false;
        };
        let left_type = left_meta.file_type();
        let right_type = right_meta.file_type();
        if left_type.is_symlink() || right_type.is_symlink() || left_type != right_type {
            return false;
        }
        if left_type.is_dir() {
            if !dirs_equal(&left.path(), &right.path(), depth + 1) {
                return false;
            }
        } else if left_type.is_file() {
            match (fs::read(left.path()), fs::read(right.path())) {
                (Ok(a), Ok(b)) if a == b => {}
                _ => return false,
            }
        } else {
            return false;
        }
    }
    true
}

/// 执行安装：普通条目直接复制，冲突条目按用户选择处理。
pub fn execute(units: Vec<PlannedUnit>, resolved: Vec<(PlannedUnit, ConflictChoice)>) -> Outcome {
    let mut outcome = Outcome::default();
    // Keep operations ordered: later keep-both decisions see earlier installed names.
    let actions = units
        .into_iter()
        .map(|u| (u, None))
        .chain(resolved.into_iter().map(|(u, c)| (u, Some(c))));
    for (mut unit, choice) in actions {
        if choice == Some(ConflictChoice::Skip) {
            outcome.skipped.push(unit.display);
            continue;
        }
        if choice == Some(ConflictChoice::KeepBoth) {
            unit.dest = unique_dest(&unit.dest);
            if let Some(name) = unit.dest.file_name() {
                unit.display = name.to_string_lossy().into_owned();
            }
        }
        match install_transaction(&unit, choice == Some(ConflictChoice::Overwrite)) {
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

/// Stage on the destination filesystem before touching the old item; rollback on failure.
fn install_transaction(unit: &PlannedUnit, overwrite: bool) -> Result<(), String> {
    let parent = unit.dest.parent().ok_or("目标没有父目录")?;
    fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    if unit.kind == UnitKind::Plugin {
        let source = unit
            .source
            .canonicalize()
            .map_err(|e| format!("插件源目录不可用: {e}"))?;
        let target_parent = parent
            .canonicalize()
            .map_err(|e| format!("目标目录不可用: {e}"))?;
        if target_parent.starts_with(&source) {
            return Err("目标位于插件源目录内部，已取消安装以避免递归复制".into());
        }
    }
    let staging = tempfile::Builder::new()
        .prefix(".cavalrykit-")
        .tempdir_in(parent)
        .map_err(|e| format!("创建暂存目录失败: {e}"))?;
    let staged = staging.path().join("payload");
    let mut copy = unit.clone();
    copy.dest = staged.clone();
    copy_unit(&copy)?;
    // Recheck immediately before committing, including dangling symlinks.
    let existing = fs::symlink_metadata(&unit.dest).ok();
    if existing
        .as_ref()
        .is_some_and(|m| m.file_type().is_symlink())
    {
        return Err("拒绝覆盖符号链接目标".into());
    }
    if existing.is_some() && !overwrite {
        return Err("目标已被其他操作创建，请重新导入并处理冲突".into());
    }
    let backup = staging.path().join("previous");
    if existing.is_some() {
        fs::rename(&unit.dest, &backup)
            .map_err(|e| format!("无法备份旧版本（可能正在使用）: {e}"))?;
    }
    if let Err(error) = fs::rename(&staged, &unit.dest) {
        if existing.is_some() {
            if let Err(restore) = fs::rename(&backup, &unit.dest) {
                let recovery = staging.keep();
                return Err(format!(
                    "安装失败: {error}；自动恢复失败: {restore}；旧文件保留在 {}",
                    recovery.join("previous").display()
                ));
            }
        }
        return Err(format!("安装失败，旧版本已保留: {error}"));
    }
    Ok(())
}

fn copy_unit(unit: &PlannedUnit) -> Result<(), String> {
    if fs::symlink_metadata(&unit.source)
        .map_err(|e| format!("源文件不可用: {e}"))?
        .file_type()
        .is_symlink()
    {
        return Err("不安装符号链接，请导入原始文件".into());
    }
    match unit.kind {
        UnitKind::Script => {
            if let Some(parent) = unit.dest.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            fs::copy(&unit.source, &unit.dest)
                .map(|_| ())
                .map_err(|e| format!("复制失败: {e}"))
        }
        UnitKind::Plugin => copy_dir_all(&unit.source, &unit.dest, 0),
    }
}

fn copy_dir_all(src: &Path, dst: &Path, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("插件目录嵌套超过 64 层，已取消安装".into());
    }
    fs::create_dir_all(dst).map_err(|e| format!("创建插件目录失败: {e}"))?;
    for entry in fs::read_dir(src).map_err(|e| format!("读取插件目录失败: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let meta = fs::symlink_metadata(&from).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            return Err("插件含符号链接，已取消安装并保留旧版本".into());
        }
        if meta.is_dir() {
            copy_dir_all(&from, &to, depth + 1)?;
        } else if meta.is_file() {
            fs::copy(&from, &to).map_err(|e| format!("复制 {} 失败: {e}", from.display()))?;
        } else {
            return Err("插件中含非普通文件，已取消安装".into());
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
    fn identical_reimport_is_a_quiet_duplicate_not_a_conflict() {
        let dir = tmpdir("dup");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        paths.ensure_dirs().unwrap();

        let src = dir.join("Tool.jsc");
        fs::write(&src, b"// same").unwrap();
        let first = plan(std::slice::from_ref(&src), &paths);
        assert_eq!(first.units.len(), 1);
        let out = execute(first.units, vec![]);
        assert_eq!(out.installed_scripts, vec!["Tool.jsc"]);

        // 同名同内容再次导入：不装、不问、不生成 "Tool 2.jsc"
        let again = plan(std::slice::from_ref(&src), &paths);
        assert!(again.units.is_empty());
        assert!(again.conflicts.is_empty());
        assert_eq!(again.duplicates, vec!["Tool.jsc"]);
        assert!(!paths.scripts.join("Tool 2.jsc").exists());

        // 同名不同内容仍是冲突
        fs::write(&src, b"// changed").unwrap();
        let changed = plan(std::slice::from_ref(&src), &paths);
        assert_eq!(changed.conflicts.len(), 1);
        assert!(changed.duplicates.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn identical_plugin_tree_is_a_duplicate_and_batch_twin_is_detected() {
        let dir = tmpdir("dup-plugin");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        paths.ensure_dirs().unwrap();

        let plugin = dir.join("Kit");
        fs::create_dir_all(plugin.join("sub")).unwrap();
        fs::write(plugin.join("definitions.json"), b"{}").unwrap();
        fs::write(plugin.join("sub/run.js"), b"1").unwrap();
        let first = plan(std::slice::from_ref(&plugin), &paths);
        let out = execute(first.units, vec![]);
        assert_eq!(out.installed_plugins, vec!["Kit"]);

        let again = plan(std::slice::from_ref(&plugin), &paths);
        assert_eq!(again.duplicates, vec!["Kit"]);
        assert!(again.conflicts.is_empty());

        // 同批两个来源内容相同、目标相同 → 第二个是重复项而非冲突
        let twin = dir.join("copy");
        fs::create_dir_all(twin.join("Kit/sub")).unwrap();
        fs::write(twin.join("Kit/definitions.json"), b"{}").unwrap();
        fs::write(twin.join("Kit/sub/run.js"), b"1").unwrap();
        let _ = fs::remove_dir_all(paths.plugins.join("Kit"));
        let batch = plan(&[plugin.clone(), twin.join("Kit")], &paths);
        assert_eq!(batch.units.len(), 1);
        assert_eq!(batch.duplicates, vec!["Kit"]);
        assert!(batch.conflicts.is_empty());

        // 插件树有一点不同 → 冲突
        fs::write(twin.join("Kit/sub/run.js"), b"2").unwrap();
        let batch = plan(&[plugin, twin.join("Kit")], &paths);
        assert_eq!(batch.conflicts.len(), 1);
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
        let plan1 = plan(std::slice::from_ref(&src), &paths);
        assert_eq!(plan1.conflicts.len(), 1);
        assert!(plan1.units.is_empty());
        let c = plan1.conflicts[0].clone();
        let out = execute(vec![], vec![(c, ConflictChoice::Overwrite)]);
        assert!(out.installed_scripts.contains(&"old.jsc".to_string()));
        assert_eq!(
            fs::read_to_string(paths.scripts.join("old.jsc")).unwrap(),
            "new"
        );

        // 冲突 → 跳过
        fs::write(&src, b"newer").unwrap();
        let plan2 = plan(std::slice::from_ref(&src), &paths);
        let c = plan2.conflicts[0].clone();
        let out2 = execute(plan2.units, vec![(c, ConflictChoice::Skip)]);
        assert_eq!(out2.skipped, vec!["old.jsc"]);
        assert_eq!(
            fs::read_to_string(paths.scripts.join("old.jsc")).unwrap(),
            "new"
        );

        // 冲突 → 保留两者
        let plan3 = plan(&[src], &paths);
        let c = plan3.conflicts[0].clone();
        let out3 = execute(plan3.units, vec![(c, ConflictChoice::KeepBoth)]);
        assert_eq!(out3.installed_scripts, vec!["old 2.jsc"]);
        assert_eq!(
            fs::read_to_string(paths.scripts.join("old.jsc")).unwrap(),
            "new"
        );
        assert_eq!(
            fs::read_to_string(paths.scripts.join("old 2.jsc")).unwrap(),
            "newer"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
