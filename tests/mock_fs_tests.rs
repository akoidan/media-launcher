#[path = "../src/fs_access.rs"]
mod fs_access;

#[path = "../src/os/mod.rs"]
mod os;

#[path = "../src/media_scan.rs"]
mod media_scan;

#[path = "../src/players/mod.rs"]
mod players;

#[path = "../src/app.rs"]
mod app;

mod test_vfs;

use serde::Deserialize;
use serde_json::Value;

use std::{
    env,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use fs_access::Fs;

use test_vfs::VfsFs;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn set_test_path(paths: &[PathBuf]) -> std::ffi::OsString {
    env::join_paths(paths).unwrap()
}

#[derive(Debug, Deserialize)]
struct FixtureInput {
    root: String,
    player: String,
    path_dirs: Vec<String>,
    binaries: Vec<String>,
    files: Value,
    #[serde(default)]
    expected_windows_writes: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    expected_unix_writes: std::collections::BTreeMap<String, String>,
}

fn norm_path_for_compare(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn populate_tree(fs: &VfsFs, base: &Path, node: &Value) {
    match node {
        Value::String(contents) => {
            fs.put_file(base, contents.as_bytes()).unwrap();
        }
        Value::Object(map) => {
            fs.ensure_dir(base).unwrap();
            for (name, child) in map {
                populate_tree(fs, &base.join(name), child);
            }
        }
        _ => {
            panic!("Invalid fixture node for {}", base.display());
        }
    }
}

#[test]
fn golden_fixture_solo_leveling() {
    let _guard = env_lock().lock().unwrap();

    let input: FixtureInput = serde_json::from_str(include_str!(
        "fixtures/solo-leveling.json"
    ))
    .unwrap();

    let expected = {
        #[cfg(windows)]
        {
            &input.expected_windows_writes
        }

        #[cfg(not(windows))]
        {
            &input.expected_unix_writes
        }
    };

    let fs = VfsFs::default();
    let root = PathBuf::from(&input.root);

    fs.ensure_dir(&root).unwrap();

    populate_tree(&fs, &root, &input.files);

    for d in &input.path_dirs {
        fs.ensure_dir(&PathBuf::from(d)).unwrap();
    }

    for bin in &input.binaries {
        let decorated = os::decorate_program_name(bin);
        fs.create_file(&PathBuf::from("bin").join(decorated)).unwrap();
    }

    let old_path = env::var_os("PATH");
    let path_dirs = input.path_dirs.iter().map(PathBuf::from).collect::<Vec<_>>();
    env::set_var("PATH", set_test_path(&path_dirs));

    let player = match input.player.as_str() {
        "mpv" => players::PlayerKind::Mpv,
        "vlc" => players::PlayerKind::Vlc,
        other => panic!("Unknown player kind in fixture: {other}"),
    };

    let args = app::Args {
        root_dir: Some(root.clone()),
        player: Some(player),
    };

    let result = app::run_with(&fs, args);

    if let Some(p) = old_path {
        env::set_var("PATH", p);
    } else {
        env::remove_var("PATH");
    }

    result.unwrap();

    let writes = fs.writes.lock().unwrap();
    let actual: std::collections::BTreeMap<String, String> = writes
        .iter()
        .map(|(p, bytes)| {
            (
                norm_path_for_compare(p),
                String::from_utf8_lossy(bytes).to_string(),
            )
        })
        .collect();

    assert_eq!(*expected, actual);
}
