//! 输入识别：单个脚本文件 / 插件文件夹 / 脚本或插件压缩包。

use std::fs;
use std::path::{Path, PathBuf};

use crate::installer::{PlannedUnit, UnitKind};
use crate::paths::CavalryPaths;
use crate::unzipper::{self, is_junk_name};

const SCRIPT_EXTS: [&str; 3] = ["jsc", "js", "jsx"];

/// 是否属于 Cavalry 脚本文件（按扩展名，大小写不敏感）。
pub fn is_script_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SCRIPT_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// 识别一个用户输入（文件或文件夹），返回待安装条目。
///
/// zip 会先解压到 `temp` 目录下（条目源路径位于临时目录中，
/// 由调用方通过 `InstallPlan._temp` 守卫保证其存活期）。
pub fn inspect(
    input: &Path,
    temp: &Path,
    paths: &CavalryPaths,
) -> Result<Vec<PlannedUnit>, String> {
    if !input.exists() {
        return Err("文件不存在".to_string());
    }
    let meta = fs::metadata(input).map_err(|e| format!("无法读取文件信息: {e}"))?;

    if meta.is_dir() {
        return inspect_dir(input, paths);
    }

    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "jsc" | "js" | "jsx" => {
            let file_name = file_name_of(input)?;
            Ok(vec![PlannedUnit {
                kind: UnitKind::Script,
                display: file_name.clone(),
                source: input.to_path_buf(),
                dest: paths.scripts.join(&file_name),
            }])
        }
        _ if unzipper::archive_kind(input).is_some() => inspect_archive(input, temp, paths),
        _ => Err(format!("不支持的文件类型: .{ext}")),
    }
}

/// 文件夹输入：含 definitions.json → 插件；否则作为脚本集合收集其内部脚本。
fn inspect_dir(dir: &Path, paths: &CavalryPaths) -> Result<Vec<PlannedUnit>, String> {
    if dir.join("definitions.json").is_file() {
        let name = file_name_of(dir)?;
        return Ok(vec![PlannedUnit {
            kind: UnitKind::Plugin,
            display: name.clone(),
            source: dir.to_path_buf(),
            dest: paths.plugins.join(&name),
        }]);
    }

    let mut found = Vec::new();
    collect_scripts_under(dir, dir, &[], paths, &mut found);
    if found.is_empty() {
        Err("文件夹中没有可安装的脚本".to_string())
    } else {
        Ok(found)
    }
}

/// 压缩包输入(zip / tar / tar.gz / tar.bz2 / 7z):插件优先(包根直接是产品内容 →
/// 以包名命名;否则递归寻找含 definitions.json 的产品目录);
/// 同包脚本一并收集,但跳过插件目录内部。
fn inspect_archive(
    archive_path: &Path,
    temp: &Path,
    paths: &CavalryPaths,
) -> Result<Vec<PlannedUnit>, String> {
    let stem = unzipper::archive_stem(archive_path).unwrap_or_else(|| "archive".into());
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let root = temp.join(format!("{stem}-{stamp}"));
    fs::create_dir_all(&root).map_err(|e| format!("创建临时目录失败: {e}"))?;
    unzipper::extract_archive(archive_path, &root)?;

    let mut units: Vec<PlannedUnit> = Vec::new();

    // 1a) zip 根直接含 definitions.json(连内容一起压缩的包)→ 整个根即插件,以 zip 名命名
    if root.join("definitions.json").is_file() {
        units.push(PlannedUnit {
            kind: UnitKind::Plugin,
            display: stem.clone(),
            source: root.clone(),
            dest: paths.plugins.join(&stem),
        });
        return Ok(units);
    }

    // 1b) 递归找插件产品目录(命中后不再深入其内部)
    let mut plugin_dirs: Vec<PathBuf> = Vec::new();
    find_plugin_dirs(&root, &mut plugin_dirs);
    for d in &plugin_dirs {
        let name = file_name_of(d)?;
        units.push(PlannedUnit {
            kind: UnitKind::Plugin,
            display: name.clone(),
            source: d.clone(),
            dest: paths.plugins.join(&name),
        });
    }

    // 2) 脚本集合:剥壳后收集;插件目录内部的文件是插件资源,不当脚本
    let eff = effective_root(&root);
    collect_scripts_under(&eff, &eff, &plugin_dirs, paths, &mut units);
    if units.is_empty() {
        Err("压缩包中没有可识别的脚本或插件".to_string())
    } else {
        Ok(units)
    }
}

/// 若根目录下恰好只剩一个子目录，则视为包裹壳并进入其中。
fn effective_root(root: &Path) -> PathBuf {
    let entries: Vec<PathBuf> = fs::read_dir(root)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| !is_junk_name(&p.file_name().and_then(|n| n.to_str()).unwrap_or("")))
                .collect()
        })
        .unwrap_or_default();
    if entries.len() == 1 && entries[0].is_dir() {
        entries[0].clone()
    } else {
        root.to_path_buf()
    }
}

/// 递归收集目录下的脚本文件（相对 base 的路径保留为子目录结构）。
/// `exclude` 内的目录（插件产品目录）整体跳过——其内部文件是插件资源。
fn collect_scripts_under(
    base: &Path,
    dir: &Path,
    exclude: &[PathBuf],
    paths: &CavalryPaths,
    out: &mut Vec<PlannedUnit>,
) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.filter_map(|e| e.ok()) {
        let p = entry.path();
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if is_junk_name(name) {
            continue;
        }
        let Ok(meta) = fs::symlink_metadata(&p) else { continue };
        if meta.is_dir() {
            if exclude.iter().any(|ex| p.starts_with(ex)) {
                continue;
            }
            collect_scripts_under(base, &p, exclude, paths, out);
        } else if is_script_file(&p) {
            let rel = p.strip_prefix(base).unwrap_or(&p).to_path_buf();
            let display = rel.to_string_lossy().into_owned();
            out.push(PlannedUnit {
                kind: UnitKind::Script,
                display,
                source: p.clone(),
                dest: paths.scripts.join(&rel),
            });
        }
    }
}

/// 递归寻找插件产品目录（含 definitions.json），命中后不再深入其内部。
fn find_plugin_dirs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.filter_map(|e| e.ok()) {
        let p = entry.path();
        let Ok(meta) = fs::symlink_metadata(&p) else { continue };
        if !meta.is_dir() {
            continue;
        }
        if is_junk_name(&entry.file_name().to_string_lossy()) {
            continue;
        }
        if p.join("definitions.json").is_file() {
            out.push(p);
        } else {
            find_plugin_dirs(&p, out);
        }
    }
}

fn file_name_of(path: &Path) -> Result<String, String> {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| format!("路径没有文件名: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "cavalrykit-test-detector-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn make_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let f = fs::File::create(path).unwrap();
        let mut w = zip::ZipWriter::new(f);
        let opt = zip::write::SimpleFileOptions::default();
        for (name, data) in entries {
            if name.ends_with('/') {
                w.add_directory(*name, opt).unwrap();
            } else {
                w.start_file(*name, opt).unwrap();
                w.write_all(data).unwrap();
            }
        }
        w.finish().unwrap();
    }

    #[test]
    fn single_script_file() {
        let dir = tmpdir("single");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        let s = dir.join("Hello.jsc");
        fs::write(&s, b"//").unwrap();
        let units = inspect(&s, &dir.join("tmp"), &paths).unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].kind, UnitKind::Script);
        assert_eq!(units[0].dest, paths.scripts.join("Hello.jsc"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn plugin_folder_with_definitions() {
        let dir = tmpdir("plugdir");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        let plug = dir.join("Field Kit");
        fs::create_dir_all(&plug).unwrap();
        fs::write(plug.join("definitions.json"), b"{}").unwrap();
        let units = inspect(&plug, &dir.join("tmp"), &paths).unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].kind, UnitKind::Plugin);
        assert_eq!(units[0].dest, paths.plugins.join("Field Kit"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn github_source_zip_strips_wrapper_and_skips_docs() {
        let dir = tmpdir("ghzip");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        let z = dir.join("repo-main.zip");
        make_zip(
            &z,
            &[
                ("repo-main/", b""),
                ("repo-main/basic/a.js", b"//a"),
                ("repo-main/basic/b.js", b"//b"),
                ("repo-main/README.md", b"readme"),
                ("repo-main/LICENSE", b"license"),
                ("__MACOSX/repo-main/._a.js", b"junk"),
            ],
        );
        let tmp = dir.join("tmp");
        fs::create_dir_all(&tmp).unwrap();
        let units = inspect(&z, &tmp, &paths).unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].dest, paths.scripts.join("basic/a.js"));
        assert_eq!(units[1].dest, paths.scripts.join("basic/b.js"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn plugin_zip_finds_product_dir() {
        let dir = tmpdir("plugzip");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        let z = dir.join("fieldkit.zip");
        make_zip(
            &z,
            &[
                ("Field Kit/", b""),
                ("Field Kit/definitions.json", b"{}"),
                ("Field Kit/glow.sksl", b"shader"),
                ("__MACOSX/._Field Kit", b"junk"),
            ],
        );
        let tmp = dir.join("tmp");
        fs::create_dir_all(&tmp).unwrap();
        let units = inspect(&z, &tmp, &paths).unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].kind, UnitKind::Plugin);
        assert_eq!(units[0].display, "Field Kit");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unrecognized_zip_errors() {
        let dir = tmpdir("badzip");
        let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
        let z = dir.join("photos.zip");
        make_zip(&z, &[ ("img/cat.png", b"png") ]);
        let tmp = dir.join("tmp");
        fs::create_dir_all(&tmp).unwrap();
        let r = inspect(&z, &tmp, &paths);
        assert!(r.is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
