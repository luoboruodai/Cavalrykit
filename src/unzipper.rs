//! 压缩包解压:zip / tar / tar.gz / tar.bz2 / 7z。
//! 全部格式统一:过滤 macOS 元数据垃圾,防护 zip-slip 路径穿越。

use std::fs::File;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

/// 支持的压缩格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    SevenZ,
}

/// 按文件名后缀识别压缩格式(双后缀 .tar.gz 等优先匹配)。
pub fn archive_kind(path: &Path) -> Option<ArchiveKind> {
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    if name.ends_with(".zip") {
        Some(ArchiveKind::Zip)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Some(ArchiveKind::TarGz)
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        Some(ArchiveKind::TarBz2)
    } else if name.ends_with(".tar") {
        Some(ArchiveKind::Tar)
    } else if name.ends_with(".7z") {
        Some(ArchiveKind::SevenZ)
    } else {
        None
    }
}

/// 压缩包名去掉整个压缩后缀(pack.tar.gz → pack),用于插件命名。
pub fn archive_stem(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let lower = name.to_ascii_lowercase();
    for suf in [
        ".tar.gz", ".tar.bz2", ".tgz", ".tbz2", ".zip", ".tar", ".7z",
    ] {
        if lower.ends_with(suf) {
            return Some(name[..name.len() - suf.len()].to_string());
        }
    }
    None
}

/// 按格式分发解压。
pub fn extract_archive(archive_path: &Path, dest: &Path) -> Result<(), String> {
    match archive_kind(archive_path) {
        Some(ArchiveKind::Zip) => extract_zip(archive_path, dest),
        Some(ArchiveKind::Tar) => {
            let f = File::open(archive_path).map_err(|e| format!("无法打开压缩包: {e}"))?;
            extract_tar(f, dest)
        }
        Some(ArchiveKind::TarGz) => {
            let f = File::open(archive_path).map_err(|e| format!("无法打开压缩包: {e}"))?;
            extract_tar(flate2::read::GzDecoder::new(f), dest)
        }
        Some(ArchiveKind::TarBz2) => {
            let f = File::open(archive_path).map_err(|e| format!("无法打开压缩包: {e}"))?;
            extract_tar(bzip2::read::BzDecoder::new(f), dest)
        }
        Some(ArchiveKind::SevenZ) => extract_7z(archive_path, dest),
        None => Err("不支持的压缩格式(支持 zip / 7z / tar / tar.gz / tar.bz2)".to_string()),
    }
}

/// tar 家族解压:同一套垃圾过滤与路径净化。
fn extract_tar<R: Read>(reader: R, dest: &Path) -> Result<(), String> {
    let mut archive = tar::Archive::new(reader);
    let entries = archive
        .entries()
        .map_err(|e| format!("不是有效的 tar 包: {e}"))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("读取 tar 条目失败: {e}"))?;
        let raw = entry
            .path()
            .map_err(|e| format!("读取 tar 条目名失败: {e}"))?
            .into_owned();
        let out = match filtered_out_path(&raw, dest)? {
            Some(p) => p,
            None => continue, // 垃圾条目
        };
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&out).map_err(|e| format!("创建目录失败: {e}"))?;
        } else {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            let mut out_file =
                File::create(&out).map_err(|e| format!("写入文件 {} 失败: {e}", out.display()))?;
            io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("解压 {} 失败: {e}", out.display()))?;
        }
    }
    Ok(())
}

/// 7z 解压:同一套垃圾过滤与路径净化。
fn extract_7z(path: &Path, dest: &Path) -> Result<(), String> {
    let mut reader = sevenz_rust::SevenZReader::open(path, sevenz_rust::Password::empty())
        .map_err(|e| format!("不是有效的 7z 文件: {e}"))?;
    let mut failed: Option<String> = None;
    let r = reader.for_each_entries(|entry, content| {
        if failed.is_some() {
            return Ok(false);
        }
        let raw = PathBuf::from(entry.name());
        let out = match filtered_out_path(&raw, dest) {
            Ok(Some(p)) => p,
            Ok(None) => return Ok(true), // 垃圾条目,跳过
            Err(msg) => {
                failed = Some(msg);
                return Ok(false);
            }
        };
        let step: Result<(), String> = (|| {
            if entry.is_directory() {
                std::fs::create_dir_all(&out).map_err(|e| format!("创建目录失败: {e}"))?;
            } else {
                if let Some(parent) = out.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
                }
                let mut out_file = File::create(&out)
                    .map_err(|e| format!("写入文件 {} 失败: {e}", out.display()))?;
                io::copy(content, &mut out_file)
                    .map_err(|e| format!("解压 {} 失败: {e}", out.display()))?;
            }
            Ok(())
        })();
        if let Err(msg) = step {
            failed = Some(msg);
            return Ok(false);
        }
        Ok(true)
    });
    if let Some(msg) = failed {
        return Err(msg);
    }
    r.map_err(|e| format!("解压 7z 失败: {e}"))?;
    Ok(())
}

/// 垃圾过滤 + 路径净化 → 目标落点;垃圾条目返回 Ok(None),危险路径返回 Err。
fn filtered_out_path(raw: &Path, dest: &Path) -> Result<Option<PathBuf>, String> {
    // 任一路径组件是垃圾 → 整条跳过
    if raw
        .components()
        .filter_map(|c| match c {
            Component::Normal(p) => p.to_str(),
            _ => None,
        })
        .any(is_junk_name)
    {
        return Ok(None);
    }
    let Some(safe_rel) = sanitize(raw) else {
        return Err(format!("压缩包包含不安全路径: {}", raw.display()));
    };
    Ok(Some(dest.join(safe_rel)))
}

/// 判断文件名/目录名是否为安装时应忽略的垃圾（macOS 压缩包常见）。
pub fn is_junk_name(name: &str) -> bool {
    name == "__MACOSX" || name == ".DS_Store" || name.starts_with("._")
}

/// 净化 zip 条目路径：拒绝绝对路径与 `..` 穿越，仅保留正常组件。
fn sanitize(rel: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for c in rel.components() {
        match c {
            Component::Normal(p) => out.push(p),
            _ => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

/// 解压 zip 到 dest 目录。已存在的同名文件会被覆盖。
pub fn extract_zip(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| format!("无法打开压缩包: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("不是有效的 zip 文件: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取压缩包条目失败: {e}"))?;

        let raw_name = entry.name().to_owned();
        let out_path = match filtered_out_path(Path::new(&raw_name), dest)? {
            Some(p) => p,
            None => continue, // 垃圾条目
        };

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| format!("创建目录失败: {e}"))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            let mut out_file = File::create(&out_path)
                .map_err(|e| format!("写入文件 {} 失败: {e}", out_path.display()))?;
            io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("解压 {} 失败: {e}", out_path.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn filters_junk_and_extracts_normal() {
        let dir = std::env::temp_dir().join("cavalrykit_test_unzip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let zip_path = dir.join("t.zip");
        {
            let f = File::create(&zip_path).unwrap();
            let mut w = zip::ZipWriter::new(f);
            let opt = zip::write::SimpleFileOptions::default();
            for (name, data) in [
                ("good/script.js", &b"hello"[..]),
                ("__MACOSX/._good", &b"junk"[..]),
                ("good/._script.js", &b"junk"[..]),
                ("good/.DS_Store", &b"junk"[..]),
            ] {
                w.start_file(name, opt).unwrap();
                w.write_all(data).unwrap();
            }
            w.finish().unwrap();
        }
        let out = dir.join("out");
        extract_zip(&zip_path, &out).unwrap();
        assert!(out.join("good/script.js").is_file());
        assert!(!out.join("__MACOSX").exists());
        assert!(!out.join("good/._script.js").exists());
        assert!(!out.join("good/.DS_Store").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_path_traversal() {
        let dir = std::env::temp_dir().join("cavalrykit_test_unzip_slip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let zip_path = dir.join("t.zip");
        {
            let f = File::create(&zip_path).unwrap();
            let mut w = zip::ZipWriter::new(f);
            let opt = zip::write::SimpleFileOptions::default();
            w.start_file("../evil.txt", opt).unwrap();
            w.write_all(b"bad").unwrap();
            w.finish().unwrap();
        }
        let out = dir.join("out");
        let r = extract_zip(&zip_path, &out);
        assert!(r.is_err());
        assert!(!dir.join("evil.txt").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
