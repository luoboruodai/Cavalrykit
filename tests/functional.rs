//! 功能测试（端到端）：以临时目录为 Cavalry 根，走完
//! 识别(detector) → 规划/执行(installer) → 扫描(library::scan) → 卸载(library::uninstall)
//! 全链路；覆盖各脚本类型、插件文件夹、脚本 zip、插件 zip、冲突策略与坏包处理。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use cavalrykit::installer::{self, ConflictChoice};
use cavalrykit::library;
use cavalrykit::paths::CavalryPaths;
use cavalrykit::unzipper::extract_zip;

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "cavalrykit-func-{tag}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
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

/// 规划 + 无冲突执行（测试快捷路径），返回 Outcome。
fn install(inputs: &[PathBuf], paths: &CavalryPaths) -> installer::Outcome {
    let plan = installer::plan(inputs, paths);
    assert!(
        plan.conflicts.is_empty(),
        "本用例不应有冲突: {:?}",
        plan.conflicts.iter().map(|c| &c.display).collect::<Vec<_>>()
    );
    assert!(
        plan.failures.is_empty(),
        "本用例不应有失败: {:?}",
        plan.failures
    );
    installer::execute(plan.units, vec![])
}

// ---------- 1. 脚本全类型(.jsc / .js / .jsx / 大写 .JSC)安装 → 扫描 → 卸载 ----------

#[test]
fn script_types_roundtrip() {
    let dir = tmpdir("scripts");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));

    let mut inputs = Vec::new();
    for name in ["a.jsc", "b.js", "c.jsx", "D.JSC"] {
        let p = dir.join(name);
        fs::write(&p, format!("// {name}")).unwrap();
        inputs.push(p);
    }
    let out = install(&inputs, &paths);
    assert_eq!(out.installed_scripts.len(), 4);
    assert!(!out.restart_hint, "纯脚本不应提示重启");

    // 扫描:四个都应在,根目录组为 None
    let lib = library::scan(&paths);
    assert_eq!(lib.scripts.len(), 4);
    for s in &lib.scripts {
        assert_eq!(s.group, None);
    }

    // 卸载:逐个进废纸篓,目录随之清空
    for s in &lib.scripts {
        library::uninstall(&paths, &s.path).unwrap();
    }
    let lib = library::scan(&paths);
    assert!(lib.scripts.is_empty());
    let _ = fs::remove_dir_all(&dir);
}

// ---------- 2. 脚本 zip:GitHub 包裹壳 / 顶层散装 ----------

#[test]
fn script_zip_wrapper_and_flat() {
    let dir = tmpdir("scriptzip");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));

    // GitHub 式:单一包裹目录 + 文档垃圾 + macOS 垃圾
    let z1 = dir.join("repo-main.zip");
    make_zip(
        &z1,
        &[
            ("repo-main/", b""),
            ("repo-main/deformers/bend.js", b"//bend"),
            ("repo-main/deformers/twist.jsc", b"//twist"),
            ("repo-main/README.md", b"doc"),
            ("__MACOSX/repo-main/._bend.js", b"junk"),
        ],
    );
    // 散装:顶层直接是脚本
    let z2 = dir.join("loose.zip");
    make_zip(&z2, &[("solo.jsc", b"//solo"), ("note.txt", b"n")]);

    let out = install(&[z1, z2], &paths);
    assert_eq!(out.installed_scripts.len(), 3);

    // 包裹壳被剥掉,子目录结构保留
    assert!(paths.scripts.join("deformers/bend.js").is_file());
    assert!(paths.scripts.join("deformers/twist.jsc").is_file());
    assert!(paths.scripts.join("solo.jsc").is_file());
    assert!(!paths.scripts.join("repo-main").exists(), "包裹壳不应带入");
    assert!(!paths.scripts.join("README.md").exists());
    assert!(!paths.scripts.join("note.txt").exists(), "非脚本文件不应安装");

    // 扫描:子目录脚本带组名
    let lib = library::scan(&paths);
    let bend = lib.scripts.iter().find(|s| s.name == "bend.js").unwrap();
    assert_eq!(bend.group.as_deref(), Some("deformers"));

    // 卸载子目录脚本
    library::uninstall(&paths, &bend.path).unwrap();
    assert!(!paths.scripts.join("deformers/bend.js").exists());
    let _ = fs::remove_dir_all(&dir);
}

// ---------- 3. 插件:文件夹直装 / 插件 zip / 深层嵌套 zip ----------

#[test]
fn plugin_folder_and_zip_roundtrip() {
    let dir = tmpdir("plugins");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));

    // 3a. 插件文件夹直接输入
    let folder = dir.join("Field Kit");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("definitions.json"), br#"{"author":"sam"}"#).unwrap();
    fs::write(folder.join("glow.sksl"), b"shader").unwrap();

    // 3b. 插件 zip(产品目录在顶层)
    let z1 = dir.join("noise-pack.zip");
    make_zip(
        &z1,
        &[
            ("Noise Pack/", b""),
            ("Noise Pack/definitions.json", br#"{"author":"zzz"}"#),
            ("Noise Pack/noise.sksl", b"s"),
            ("__MACOSX/Noise Pack/._definitions.json", b"junk"),
        ],
    );

    // 3c. 深层嵌套 zip(wrapper/wrapper2/产品目录)
    let z2 = dir.join("deep.zip");
    make_zip(
        &z2,
        &[
            ("Deep-1.0/", b""),
            ("Deep-1.0/dist/", b""),
            ("Deep-1.0/dist/Deep Kit/", b""),
            ("Deep-1.0/dist/Deep Kit/definitions.json", br#"{"author":"deep"}"#),
        ],
    );

    let out = install(&[folder, z1, z2], &paths);
    assert_eq!(out.installed_plugins.len(), 3);
    assert!(out.restart_hint, "装了插件必须提示重启 Cavalry");

    // 落点正确:Third-Party/Plugins/<产品目录>
    assert!(paths.plugins.join("Field Kit/definitions.json").is_file());
    assert!(paths.plugins.join("Field Kit/glow.sksl").is_file(), "插件资源要完整");
    assert!(paths.plugins.join("Noise Pack/noise.sksl").is_file());
    assert!(paths.plugins.join("Deep Kit/definitions.json").is_file());
    assert!(!paths.plugins.join("__MACOSX").exists());

    // 扫描:名称 + 作者解析
    let lib = library::scan(&paths);
    assert_eq!(lib.plugins.len(), 3);
    let fk = lib.plugins.iter().find(|p| p.name == "Field Kit").unwrap();
    assert_eq!(fk.author.as_deref(), Some("sam"));
    let dk = lib.plugins.iter().find(|p| p.name == "Deep Kit").unwrap();
    assert_eq!(dk.author.as_deref(), Some("deep"));

    // 卸载插件(整目录进废纸篓)
    library::uninstall(&paths, &fk.path).unwrap();
    assert!(!paths.plugins.join("Field Kit").exists());
    assert_eq!(library::scan(&paths).plugins.len(), 2);
    let _ = fs::remove_dir_all(&dir);
}

// ---------- 4. 冲突三策略端到端(脚本 + 插件目录) ----------

#[test]
fn conflicts_all_strategies() {
    let dir = tmpdir("conflicts");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
    paths.ensure_dirs().unwrap();

    // 已有旧脚本与旧插件
    fs::write(paths.scripts.join("tool.jsc"), b"old-script").unwrap();
    let old_plug = paths.plugins.join("Kit");
    fs::create_dir_all(&old_plug).unwrap();
    fs::write(old_plug.join("definitions.json"), br#"{"author":"old"}"#).unwrap();

    let new_script = dir.join("tool.jsc");
    fs::write(&new_script, b"new-script").unwrap();
    let new_plug = dir.join("Kit");
    fs::create_dir_all(&new_plug).unwrap();
    fs::write(new_plug.join("definitions.json"), br#"{"author":"new"}"#).unwrap();

    // 两个输入都撞冲突
    let plan = installer::plan(&[new_script.clone(), new_plug.clone()], &paths);
    assert_eq!(plan.conflicts.len(), 2);
    assert!(plan.units.is_empty());

    // 脚本 → 保留两者;插件 → 覆盖
    let resolved = vec![
        (plan.conflicts[0].clone(), ConflictChoice::KeepBoth),
        (plan.conflicts[1].clone(), ConflictChoice::Overwrite),
    ];
    let out = installer::execute(vec![], resolved);
    assert_eq!(out.installed_scripts, vec!["tool 2.jsc"]);
    assert_eq!(out.installed_plugins, vec!["Kit"]);

    assert_eq!(fs::read_to_string(paths.scripts.join("tool.jsc")).unwrap(), "old-script");
    assert_eq!(fs::read_to_string(paths.scripts.join("tool 2.jsc")).unwrap(), "new-script");
    assert!(paths.plugins.join("Kit/definitions.json").is_file());
    let lib = library::scan(&paths);
    assert_eq!(lib.plugins[0].author.as_deref(), Some("new"), "覆盖后作者应为新版");

    // 再撞一次选「跳过」:内容不变
    let plan2 = installer::plan(&[new_script], &paths);
    let out2 = installer::execute(plan2.units, vec![(plan2.conflicts[0].clone(), ConflictChoice::Skip)]);
    assert_eq!(out2.skipped, vec!["tool.jsc"]);
    assert_eq!(fs::read_to_string(paths.scripts.join("tool.jsc")).unwrap(), "old-script");
    let _ = fs::remove_dir_all(&dir);
}

// ---------- 5. 坏包 / 怪包:损坏 zip、空 zip、无内容 zip、zip-slip ----------

#[test]
fn bad_zips_are_reported_not_fatal() {
    let dir = tmpdir("badzips");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));

    // 损坏 zip(不是 zip 的字节)
    let corrupt = dir.join("corrupt.zip");
    fs::write(&corrupt, b"this is not a zip").unwrap();

    // 空 zip
    let empty = dir.join("empty.zip");
    make_zip(&empty, &[]);

    // 只有图片的 zip
    let photos = dir.join("photos.zip");
    make_zip(&photos, &[("img/cat.png", b"png")]);

    // zip-slip 恶意包
    let evil = dir.join("evil.zip");
    make_zip(&evil, &[("../evil.jsc", b"bad")]);

    let plan = installer::plan(
        &[corrupt.clone(), empty.clone(), photos.clone(), evil.clone()],
        &paths,
    );
    assert!(plan.units.is_empty(), "坏包不应产生任何安装条目");
    assert_eq!(plan.failures.len(), 4, "四个坏包都应有明确失败原因");
    for (name, reason) in &plan.failures {
        assert!(!reason.is_empty(), "{name} 的失败原因不能为空");
    }

    // zip-slip 不得写出任何内容到临时区之外
    assert!(!dir.join("evil.jsc").exists());
    // extract_zip 直接调用也应报错
    let out_dir = dir.join("evil-out");
    fs::create_dir_all(&out_dir).unwrap();
    assert!(extract_zip(&evil, &out_dir).is_err());
    let _ = fs::remove_dir_all(&dir);
}

// ---------- 6. 混合 zip:同时含脚本与插件目录 → 两者都装(插件目录内部除外) ----------

#[test]
fn mixed_zip_installs_both() {
    let dir = tmpdir("mixed");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));

    let z = dir.join("mixed.zip");
    make_zip(
        &z,
        &[
            ("bundle/", b""),
            ("bundle/tools/useful.jsc", b"//tool"),
            ("bundle/MyPlugin/", b""),
            ("bundle/MyPlugin/definitions.json", br#"{"author":"m"}"#),
            ("bundle/MyPlugin/helper.js", "//插件内部资源,不当脚本".as_bytes()),
        ],
    );
    let plan = installer::plan(std::slice::from_ref(&z), &paths);
    assert!(plan.failures.is_empty());
    assert_eq!(plan.units.len(), 2, "插件和脚本都应装上");

    let out = installer::execute(plan.units, vec![]);
    assert_eq!(out.installed_plugins, vec!["MyPlugin"]);
    assert_eq!(out.installed_scripts, vec!["tools/useful.jsc"]);
    // 插件目录内部的 .js 是插件资源,不得出现在脚本目录
    assert!(paths.plugins.join("MyPlugin/helper.js").is_file());
    assert!(!paths.scripts.join("MyPlugin/helper.js").exists());
    assert!(!paths.scripts.join("helper.js").exists());
    let _ = fs::remove_dir_all(&dir);
}

// ---------- 6b. zip 根直接是插件内容(definitions.json 在顶层) ----------

#[test]
fn plugin_zip_contents_at_root() {
    let dir = tmpdir("rootdef");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));

    // 用户把插件文件夹的「内容」直接压缩(而不是压缩文件夹本身)
    let z = dir.join("Root Kit.zip");
    make_zip(
        &z,
        &[
            ("definitions.json", br#"{"author":"root"}"#),
            ("shader.sksl", b"s"),
            ("helper.js", "//插件资源".as_bytes()),
        ],
    );
    let plan = installer::plan(std::slice::from_ref(&z), &paths);
    assert_eq!(plan.units.len(), 1);
    assert_eq!(plan.units[0].kind, installer::UnitKind::Plugin);
    // 以 zip 名(去扩展名)作为插件目录名
    assert_eq!(plan.units[0].display, "Root Kit");

    let out = installer::execute(plan.units, vec![]);
    assert_eq!(out.installed_plugins, vec!["Root Kit"]);
    assert!(paths.plugins.join("Root Kit/definitions.json").is_file());
    assert!(paths.plugins.join("Root Kit/shader.sksl").is_file());
    // 插件内的 .js 不得泄到脚本目录
    assert!(library::scan(&paths).scripts.is_empty());
    let lib = library::scan(&paths);
    assert_eq!(lib.plugins[0].author.as_deref(), Some("root"));
    let _ = fs::remove_dir_all(&dir);
}

// ---------- 7. 脚本文件夹输入(无 definitions.json → 按脚本集合) ----------

#[test]
fn folder_of_scripts_input() {
    let dir = tmpdir("folder");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));

    let folder = dir.join("我的脚本集");
    fs::create_dir_all(folder.join("子组")).unwrap();
    fs::write(folder.join("one.jsc"), b"//1").unwrap();
    fs::write(folder.join("子组/two.js"), b"//2").unwrap();
    fs::write(folder.join("说明.txt"), b"doc").unwrap();

    let out = install(std::slice::from_ref(&folder), &paths);
    assert_eq!(out.installed_scripts.len(), 2);
    assert!(paths.scripts.join("one.jsc").is_file());
    assert!(paths.scripts.join("子组/two.js").is_file());
    let lib = library::scan(&paths);
    let two = lib.scripts.iter().find(|s| s.name == "two.js").unwrap();
    assert_eq!(two.group.as_deref(), Some("子组"));
    let _ = fs::remove_dir_all(&dir);
}

// ---------- 8. 真实环境往返(手动运行:cargo test -- --ignored real_env) ----------
// 只增删独一无二名字的测试件,不动用户既有内容;卸载走系统废纸篓,可恢复。

#[test]
#[ignore = "写真实 ~/Library/Application Support/Cavalry,手动跑"]
fn real_env_roundtrip() {
    let paths = CavalryPaths::detect().expect("未检测到 Cavalry 目录");
    paths.ensure_dirs().unwrap();

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let tag = format!("CKTest{stamp}");

    // 8a. 真实安装一个脚本
    let stage = std::env::temp_dir().join(format!("cavalrykit-real-{stamp}"));
    fs::create_dir_all(&stage).unwrap();
    let script = stage.join(format!("{tag}.jsc"));
    fs::write(&script, "// CavalryKit 功能测试,可安全删除").unwrap();

    // 8b. 真实安装一个插件 zip
    let z = stage.join(format!("{tag}.zip"));
    let plug_dir = format!("{tag} Kit/");
    let def = format!("{tag} Kit/definitions.json");
    let def_body: &[u8] = br#"{"author":"cavalrykit-test"}"#;
    make_zip(&z, &[(&plug_dir, b""), (&def, def_body)]);

    let before = library::scan(&paths);
    let out = install(&[script.clone(), z.clone()], &paths);
    assert_eq!(out.installed_scripts, vec![format!("{tag}.jsc")]);
    assert_eq!(out.installed_plugins, vec![format!("{tag} Kit")]);

    let mid = library::scan(&paths);
    assert_eq!(mid.scripts.len(), before.scripts.len() + 1);
    assert_eq!(mid.plugins.len(), before.plugins.len() + 1);
    let plug = mid.plugins.iter().find(|p| p.name == format!("{tag} Kit")).unwrap();
    assert_eq!(plug.author.as_deref(), Some("cavalrykit-test"));

    // 8c. 真实卸载(进废纸篓)
    let s = mid.scripts.iter().find(|s| s.name == format!("{tag}.jsc")).unwrap();
    library::uninstall(&paths, &s.path).unwrap();
    library::uninstall(&paths, &plug.path).unwrap();

    let after = library::scan(&paths);
    assert_eq!(after.scripts.len(), before.scripts.len());
    assert_eq!(after.plugins.len(), before.plugins.len());

    let _ = fs::remove_dir_all(&stage);
    println!("真实环境往返通过:安装 → 扫描 → 卸载(废纸篓)均正常");
}

// ==================== 多压缩格式兼容(zip 之外:tar / tar.gz / tgz / tar.bz2 / tbz2 / 7z) ====================

fn make_tar_like(path: &Path, entries: &[(&str, &[u8])], enc: &str) {
    let mut buf = Vec::new();
    {
        let mut b = tar::Builder::new(&mut buf);
        for (name, data) in entries {
            if name.ends_with('/') {
                let mut h = tar::Header::new_gnu();
                h.set_entry_type(tar::EntryType::Directory);
                h.set_size(0);
                h.set_cksum();
                b.append_data(&mut h, name.trim_end_matches('/'), &[][..]).unwrap();
            } else {
                let mut h = tar::Header::new_gnu();
                h.set_size(data.len() as u64);
                h.set_cksum();
                b.append_data(&mut h, name, &data[..]).unwrap();
            }
        }
        b.finish().unwrap();
    }
    let out: Vec<u8> = match enc {
        "gz" => {
            let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            e.write_all(&buf).unwrap();
            e.finish().unwrap()
        }
        "bz2" => {
            let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::default());
            e.write_all(&buf).unwrap();
            e.finish().unwrap()
        }
        _ => buf,
    };
    fs::write(path, out).unwrap();
}

fn make_7z(path: &Path, entries: &[(&str, &[u8])]) {
    let mut w = sevenz_rust::SevenZWriter::create(path).unwrap();
    for (name, data) in entries {
        let dir = name.ends_with('/');
        let mut e = sevenz_rust::SevenZArchiveEntry::default();
        e.name = name.trim_end_matches('/').to_string();
        e.is_directory = dir;
        e.has_stream = !dir;
        if dir {
            w.push_archive_entry::<&[u8]>(e, None).unwrap();
        } else {
            w.push_archive_entry(e, Some(&data[..])).unwrap();
        }
    }
    w.finish().unwrap();
}

/// 同一棵「包裹壳 + 脚本 + 文档 + macOS 垃圾」逻辑树,逐格式验证
fn assert_script_archive(dir: &Path, archive: &Path, tag: &str) {
    let paths = CavalryPaths::from_base(&dir.join(format!("Cavalry-{tag}")));
    let out = install(std::slice::from_ref(&archive.to_path_buf()), &paths);
    assert_eq!(out.installed_scripts.len(), 2, "{tag} 应装 2 个脚本");
    assert!(paths.scripts.join("kit/a.jsc").is_file(), "{tag}: 包裹壳未剥或结构丢失");
    assert!(paths.scripts.join("b.js").is_file(), "{tag}");
    assert!(!paths.scripts.join("wrap").exists(), "{tag}: 包裹壳不应带入");
    assert!(!paths.scripts.join("README.md").exists(), "{tag}: 文档不应安装");
    // macOS 垃圾被过滤
    assert!(!paths.scripts.join("kit/._a.jsc").exists(), "{tag}: 垃圾泄漏");
    assert!(!paths.scripts.join("__MACOSX").exists(), "{tag}: 垃圾泄漏");
}

/// 同一棵插件树,逐格式验证(名称 + 作者 + 资源完整)
fn assert_plugin_archive(dir: &Path, archive: &Path, tag: &str) {
    let paths = CavalryPaths::from_base(&dir.join(format!("Cavalry-{tag}")));
    let out = install(std::slice::from_ref(&archive.to_path_buf()), &paths);
    assert_eq!(out.installed_plugins, vec!["Prod".to_string()], "{tag}");
    assert!(out.restart_hint, "{tag}");
    assert!(paths.plugins.join("Prod/definitions.json").is_file(), "{tag}");
    assert!(paths.plugins.join("Prod/res.sksl").is_file(), "{tag}");
    let lib = library::scan(&paths);
    assert_eq!(lib.plugins[0].author.as_deref(), Some("fmt"), "{tag}");
    // 卸载闭环
    library::uninstall(&paths, &lib.plugins[0].path).unwrap();
    assert!(library::scan(&paths).plugins.is_empty(), "{tag}");
}

const SCRIPT_TREE: &[(&str, &[u8])] = &[
    ("wrap/", b""),
    ("wrap/kit/", b""),
    ("wrap/kit/a.jsc", b"//a"),
    ("wrap/b.js", b"//b"),
    ("wrap/README.md", b"doc"),
    ("__MACOSX/wrap/kit/._a.jsc", b"junk"),
];

const PLUGIN_TREE: &[(&str, &[u8])] = &[
    ("Prod/", b""),
    ("Prod/definitions.json", br#"{"author":"fmt"}"#),
    ("Prod/res.sksl", b"shader"),
];

#[test]
fn compat_tar_family_and_7z() {
    let dir = tmpdir("formats");
    let cases: [(&str, &str); 6] = [
        ("tar", ""), ("tar.gz", "gz"), ("tgz", "gz"),
        ("tar.bz2", "bz2"), ("tbz2", "bz2"), ("7z", "7z"),
    ];
    for (ext, enc) in cases {
        // 脚本包
        let s = dir.join(format!("scripts-{ext}.{ext}"));
        if enc == "7z" { make_7z(&s, SCRIPT_TREE) } else { make_tar_like(&s, SCRIPT_TREE, enc) }
        assert_script_archive(&dir, &s, &format!("scripts-{ext}"));
        // 插件包
        let p = dir.join(format!("plugin.{ext}"));
        if enc == "7z" { make_7z(&p, PLUGIN_TREE) } else { make_tar_like(&p, PLUGIN_TREE, enc) }
        assert_plugin_archive(&dir, &p, &format!("plugin-{ext}"));
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn compat_7z_root_definitions_named_by_archive() {
    // 7z 包根直接是插件内容 → 以包名命名(与 zip 行为一致)
    let dir = tmpdir("7zroot");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
    let z = dir.join("Seven Kit.7z");
    make_7z(&z, &[("definitions.json", br#"{"author":"s7"}"#), ("res.sksl", b"s")]);
    let out = install(std::slice::from_ref(&z), &paths);
    assert_eq!(out.installed_plugins, vec!["Seven Kit".to_string()]);
    assert!(paths.plugins.join("Seven Kit/definitions.json").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unsupported_formats_get_clear_errors() {
    let dir = tmpdir("unsupported");
    let paths = CavalryPaths::from_base(&dir.join("Cavalry"));
    paths.ensure_dirs().unwrap();
    for name in ["x.rar", "x.gz", "x.xz", "x.iso", "x.txt"] {
        let p = dir.join(name);
        fs::write(&p, b"nope").unwrap();
        let plan = installer::plan(std::slice::from_ref(&p), &paths);
        assert!(plan.units.is_empty(), "{name} 不应产生安装条目");
        assert_eq!(plan.failures.len(), 1, "{name} 应有明确失败原因");
        assert!(!plan.failures[0].1.is_empty());
    }
    // 伪装成 zip 的坏内容
    let fake = dir.join("fake.tar.gz");
    fs::write(&fake, b"not really gzip").unwrap();
    let plan = installer::plan(std::slice::from_ref(&fake), &paths);
    assert_eq!(plan.failures.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}
