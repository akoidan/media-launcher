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

use common::{build_mock_fs, MockFs};

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
    #[serde(default)]
    #[allow(dead_code)]
    expected_macos_writes: std::collections::BTreeMap<String, String>,
}

fn parse_fixture_input(fixture_path: &Path, fixture_json: &str) -> FixtureInput {
    serde_json::from_str(fixture_json).unwrap_or_else(|e| {
        panic!(
            "Failed to parse fixture JSON {}: {e}",
            fixture_path.to_string_lossy()
        )
    })
}

fn resolve_root(fixture_path: &Path, input: &FixtureInput) -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(&input.root.windows)
    }

    #[cfg(not(windows))]
    {
        let fixture_stem = fixture_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        PathBuf::from(&input.root.linux).join(&fixture_stem)
    }
}

fn expected_writes(input: &FixtureInput) -> &std::collections::BTreeMap<String, String> {
    #[cfg(windows)]
    {
        &input.expected_windows_writes
    }

    #[cfg(target_os = "macos")]
    {
        &input.expected_macos_writes
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        &input.expected_unix_writes
    }
}

struct PathGuard {
    old_path: Option<std::ffi::OsString>,
}

impl PathGuard {
    fn set(paths: &[PathBuf]) -> Self {
        let old_path = env::var_os("PATH");
        env::set_var("PATH", set_test_path(paths));
        Self { old_path }
    }
}

impl Drop for PathGuard {
    fn drop(&mut self) {
        if let Some(p) = self.old_path.take() {
            env::set_var("PATH", p);
        } else {
            env::remove_var("PATH");
        }
    }
}

fn resolve_player_kind(input: &FixtureInput) -> players::PlayerKind {
    match input.player.as_str() {
        "mpv" => players::PlayerKind::Mpv,
        "vlc" => players::PlayerKind::Vlc,
        other => panic!("Unknown player kind in fixture: {other}"),
    }
}

fn collect_writes(fs: &MockFs) -> std::collections::BTreeMap<String, String> {
    let writes = fs.writes.lock().unwrap();
    writes
        .iter()
        .map(|(p, bytes)| {
            (
                p.to_string_lossy().to_string(),
                String::from_utf8_lossy(bytes).to_string(),
            )
        })
        .collect()
}

fn assert_fixture_writes(
    fixture_path: &Path,
    expected: &std::collections::BTreeMap<String, String>,
    actual: &std::collections::BTreeMap<String, String>,
) {
    let expected_json = serde_json::to_string_pretty(expected).unwrap();
    let actual_json = serde_json::to_string_pretty(actual).unwrap();

    pretty_assertions::assert_eq!(
        format!(
            "Fixture mismatch: {}\n{expected_json}",
            fixture_path.display()
        ),
        format!(
            "Fixture mismatch: {}\n{actual_json}",
            fixture_path.display()
        ),
    );
}

fn run_fixture(fixture_path: &Path, fixture_json: &str) {
    let input = parse_fixture_input(fixture_path, fixture_json);
    let root = resolve_root(fixture_path, &input);
    let expected = expected_writes(&input);
    let fs = build_mock_fs(
        &root,
        &input.files,
        &input.path_dirs,
        &input.binaries,
        os::path_program_name,
    );

    let path_dirs = input
        .path_dirs
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let _path_guard = PathGuard::set(&path_dirs);

    let args = app::Args {
        root_dir: Some(root),
        player: Some(resolve_player_kind(&input)),
    };

    app::run_with(&fs, args).unwrap_or_else(|e| {
        panic!(
            "Fixture run failed for {}: {e}",
            fixture_path.to_string_lossy()
        )
    });

    let actual = collect_writes(&fs);
    assert_fixture_writes(fixture_path, expected, &actual);
}

fn run_fixture_file(rel_path: &str) {
    let _guard = env_lock().lock().unwrap();

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    let json = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {e}", fixture_path.display()));
    run_fixture(&fixture_path, &json);
}

include!(concat!(env!("OUT_DIR"), "/generated_fixture_tests.rs"));
