#[path = "../src/fs_access.rs"]
mod fs_access;

#[path = "../src/os/mod.rs"]
mod os;

#[path = "../src/media_scan.rs"]
mod media_scan;

#[path = "../src/players/mod.rs"]
mod players;

#[path = "../src/app.rs"]
#[allow(dead_code)]
mod app;

mod common;

use serde::Deserialize;
use serde_json::Value;

use std::{
    env,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use common::{populate_tree, MockFs};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn set_test_path(paths: &[PathBuf]) -> std::ffi::OsString {
    env::join_paths(paths).unwrap()
}

#[derive(Debug, Deserialize)]
struct RootPaths {
    #[allow(dead_code)]
    windows: String,
    #[allow(dead_code)]
    linux: String,
}

#[derive(Debug, Deserialize)]
struct FixtureInput {
    root: RootPaths,
    player: String,
    path_dirs: Vec<String>,
    binaries: Vec<String>,
    files: Value,
    #[serde(default)]
    #[allow(dead_code)]
    expected_windows_writes: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    #[allow(dead_code)]
    expected_unix_writes: std::collections::BTreeMap<String, String>,
}

fn run_fixture(fixture_path: &Path, fixture_json: &str) {
    let input: FixtureInput = serde_json::from_str(fixture_json).unwrap_or_else(|e| {
        panic!(
            "Failed to parse fixture JSON {}: {e}",
            fixture_path.to_string_lossy()
        )
    });

    #[cfg(windows)]
    let windows_root = PathBuf::from(&input.root.windows);

    #[cfg(windows)]
    let root = windows_root;

    #[cfg(not(windows))]
    let root = {
        let fixture_stem = fixture_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        PathBuf::from(&input.root.linux).join(&fixture_stem)
    };

    #[cfg(windows)]
    let expected = &input.expected_windows_writes;

    #[cfg(not(windows))]
    let expected = &input.expected_unix_writes;

    let mut fs = MockFs::default();

    fs.add_dir(root.clone());

    populate_tree(&mut fs, &root, &input.files);

    for d in &input.path_dirs {
        fs.add_dir(PathBuf::from(d));
    }

    for bin in &input.binaries {
        let decorated = os::decorate_program_name(bin);
        fs.add_file(PathBuf::from("bin").join(decorated));
    }

    let old_path = env::var_os("PATH");
    let path_dirs = input
        .path_dirs
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
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

    result.unwrap_or_else(|e| {
        panic!(
            "Fixture run failed for {}: {e}",
            fixture_path.to_string_lossy()
        )
    });

    let writes = fs.writes.lock().unwrap();
    let actual: std::collections::BTreeMap<String, String> = writes
        .iter()
        .map(|(p, bytes)| {
            (
                p.to_string_lossy().to_string(),
                String::from_utf8_lossy(bytes).to_string(),
            )
        })
        .collect();

    let expected_json = serde_json::to_string_pretty(expected).unwrap();
    let actual_json = serde_json::to_string_pretty(&actual).unwrap();

    pretty_assertions::assert_eq!(
        format!("Fixture mismatch: {}\n{expected_json}", fixture_path.display()),
        format!("Fixture mismatch: {}\n{actual_json}", fixture_path.display()),
    );
}

#[test]
fn golden_fixtures() {
    let _guard = env_lock().lock().unwrap();

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let mut fixtures = std::fs::read_dir(&fixtures_dir)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to read fixtures dir {}: {e}",
                fixtures_dir.display()
            )
        })
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect::<Vec<_>>();

    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "No fixtures found in {}",
        fixtures_dir.display()
    );

    for fixture_path in fixtures {
        let json = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {e}", fixture_path.display()));
        run_fixture(&fixture_path, &json);
    }
}
