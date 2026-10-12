//! 0.1.1 safety, background-work and scan regressions; synthetic files only.
use cavalrykit::{
    installer::{self, ConflictChoice, PlannedUnit, UnitKind},
    library,
    paths::CavalryPaths,
    worker::{Event, Job, Worker},
};
use std::{
    fs,
    path::Path,
    sync::mpsc::TryRecvError,
    time::{Duration, Instant},
};

fn script(source: &Path, dest: &Path) -> PlannedUnit {
    PlannedUnit {
        kind: UnitKind::Script,
        display: "test.js".into(),
        source: source.into(),
        dest: dest.into(),
    }
}

#[test]
fn failed_overwrite_preserves_old_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let dest = tmp.path().join("test.js");
    fs::write(&dest, "old version").unwrap();
    let result = installer::execute(
        vec![],
        vec![(
            script(&tmp.path().join("missing.js"), &dest),
            ConflictChoice::Overwrite,
        )],
    );
    assert_eq!(result.failures.len(), 1);
    assert_eq!(fs::read_to_string(&dest).unwrap(), "old version");
    assert_eq!(
        fs::read_dir(tmp.path()).unwrap().count(),
        1,
        "no leftover staging"
    );
}

#[test]
fn self_import_overwrite_does_not_delete_source() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("test.js");
    fs::write(&file, "keep me").unwrap();
    let result = installer::execute(
        vec![],
        vec![(script(&file, &file), ConflictChoice::Overwrite)],
    );
    assert!(result.failures.is_empty());
    assert_eq!(fs::read_to_string(file).unwrap(), "keep me");
}

#[test]
fn target_created_after_planning_is_not_silently_overwritten() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("new.js");
    let dst = tmp.path().join("old.js");
    fs::write(&src, "incoming").unwrap();
    fs::write(&dst, "new external change").unwrap();
    let result = installer::execute(vec![script(&src, &dst)], vec![]);
    assert_eq!(result.failures.len(), 1);
    assert_eq!(fs::read_to_string(dst).unwrap(), "new external change");
}

#[test]
fn duplicate_destinations_in_one_batch_are_conflicts() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a");
    let b = tmp.path().join("b");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    fs::write(a.join("same.js"), "first").unwrap();
    fs::write(b.join("same.js"), "second").unwrap();
    let paths = CavalryPaths::from_base(&tmp.path().join("Cavalry"));
    let mut plan = installer::plan(&[a.join("same.js"), b.join("same.js")], &paths);
    assert_eq!(plan.units.len(), 1);
    assert_eq!(plan.conflicts.len(), 1);
    let resolved = plan
        .conflicts
        .drain(..)
        .map(|u| (u, ConflictChoice::KeepBoth))
        .collect();
    let outcome = installer::execute(std::mem::take(&mut plan.units), resolved);
    assert!(outcome.failures.is_empty());
    assert_eq!(
        fs::read_to_string(paths.scripts.join("same.js")).unwrap(),
        "first"
    );
    assert_eq!(
        fs::read_to_string(paths.scripts.join("same 2.js")).unwrap(),
        "second"
    );
}

#[test]
fn keep_both_multiple_choices_reserve_distinct_names() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("new.js");
    let dst = tmp.path().join("old.js");
    fs::write(&src, "incoming").unwrap();
    fs::write(&dst, "old").unwrap();
    let result = installer::execute(
        vec![],
        vec![
            (script(&src, &dst), ConflictChoice::KeepBoth),
            (script(&src, &dst), ConflictChoice::KeepBoth),
        ],
    );
    assert_eq!(result.installed_scripts.len(), 2);
    assert!(tmp.path().join("old 2.js").exists());
    assert!(tmp.path().join("old 3.js").exists());
}

#[test]
fn uninstall_cannot_remove_library_roots() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = CavalryPaths::from_base(tmp.path());
    paths.ensure_dirs().unwrap();
    assert!(library::uninstall(&paths, &paths.scripts).is_err());
    assert!(library::uninstall(&paths, &paths.plugins).is_err());
    assert!(paths.scripts.exists() && paths.plugins.exists());
}

#[test]
fn background_archive_plan_keeps_temporary_sources_until_install() {
    use std::io::Write;
    let tmp = tempfile::tempdir().unwrap();
    let paths = CavalryPaths::from_base(&tmp.path().join("Cavalry"));
    let zip = tmp.path().join("scripts.zip");
    let mut archive = zip::ZipWriter::new(fs::File::create(&zip).unwrap());
    archive
        .start_file("test.js", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(b"synthetic test").unwrap();
    archive.finish().unwrap();
    let worker = Worker::spawn(paths.clone(), || {});
    worker.submit(Job::Plan(vec![zip])).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut temporary = None;
    loop {
        assert!(Instant::now() < deadline, "worker timed out");
        match worker.try_recv() {
            Ok(Event::Planned(plan)) => {
                assert_eq!(plan.units.len(), 1);
                assert!(plan.units[0].source.exists());
                temporary = Some(plan._temp.as_ref().unwrap().0.clone());
                worker.submit(Job::Install(plan, vec![])).unwrap();
            }
            Ok(Event::Installed(outcome, lib)) => {
                assert!(outcome.failures.is_empty());
                assert_eq!(lib.scripts.len(), 1);
                assert_eq!(
                    fs::read_to_string(paths.scripts.join("test.js")).unwrap(),
                    "synthetic test"
                );
                assert!(!temporary.unwrap().exists());
                break;
            }
            Ok(Event::Failed(e)) => panic!("{e}"),
            Ok(_) | Err(TryRecvError::Empty) => std::thread::sleep(Duration::from_millis(2)),
            Err(TryRecvError::Disconnected) => panic!("worker disconnected"),
        }
    }
}

#[cfg(unix)]
#[test]
fn scans_do_not_follow_symbolic_link_cycles_or_external_content() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let paths = CavalryPaths::from_base(tmp.path());
    paths.ensure_dirs().unwrap();
    fs::write(paths.scripts.join("test.js"), "").unwrap();
    symlink(&paths.scripts, paths.scripts.join("loop")).unwrap();
    symlink(&paths.plugins, paths.plugins.join("loop")).unwrap();
    let lib = library::scan(&paths);
    assert_eq!(lib.scripts.len(), 1);
    assert!(lib.plugins.is_empty());
}

#[cfg(unix)]
#[test]
fn failed_plugin_staging_preserves_entire_old_plugin() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("new");
    let dst = tmp.path().join("old");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&dst).unwrap();
    fs::write(dst.join("keep.txt"), "old data").unwrap();
    fs::write(src.join("definitions.json"), "{}").unwrap();
    symlink(tmp.path().join("missing"), src.join("link")).unwrap();
    let unit = PlannedUnit {
        kind: UnitKind::Plugin,
        display: "plugin".into(),
        source: src,
        dest: dst.clone(),
    };
    let result = installer::execute(vec![], vec![(unit, ConflictChoice::Overwrite)]);
    assert_eq!(result.failures.len(), 1);
    assert_eq!(
        fs::read_to_string(dst.join("keep.txt")).unwrap(),
        "old data"
    );
    assert!(!dst.join("definitions.json").exists());
}

#[test]
fn embedded_window_icon_matches_platform_asset() {
    #[cfg(target_os = "macos")]
    let bytes = include_bytes!("../assets/icon-macos-window.png").as_slice();
    #[cfg(not(target_os = "macos"))]
    let bytes = include_bytes!("../assets/icon.ico").as_slice();
    let ico = image::load_from_memory(bytes).unwrap().to_rgba8();
    let window = cavalrykit::ui::icon::window_icon();
    assert_eq!((window.width, window.height), ico.dimensions());
    assert_eq!(window.rgba, ico.into_raw());
}

#[test]
fn nested_plugin_destination_is_rejected_before_recursive_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("plugin");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("definitions.json"), "{}").unwrap();
    let dest = source.join("nested/copy");
    let result = installer::execute(
        vec![PlannedUnit {
            kind: UnitKind::Plugin,
            display: "plugin".into(),
            source: source.clone(),
            dest: dest.clone(),
        }],
        vec![],
    );
    assert_eq!(result.failures.len(), 1);
    assert!(!dest.exists());
    assert_eq!(
        fs::read_to_string(source.join("definitions.json")).unwrap(),
        "{}"
    );
}

#[test]
fn foreign_platform_traversal_and_drive_paths_are_rejected() {
    use std::io::Write;
    for (index, name) in [
        "../escape.js",
        r"..\escape.js",
        r"folder\..\escape.js",
        r"C:\escape.js",
        r"\\server\share\escape.js",
        "/absolute.js",
        "file.js:stream",
    ]
    .iter()
    .enumerate()
    {
        let tmp = tempfile::tempdir().unwrap();
        let zip = tmp.path().join(format!("{index}.zip"));
        let mut writer = zip::ZipWriter::new(fs::File::create(&zip).unwrap());
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"synthetic").unwrap();
        writer.finish().unwrap();
        assert!(
            cavalrykit::unzipper::extract_zip(&zip, &tmp.path().join("out")).is_err(),
            "accepted {name}"
        );
    }
}
