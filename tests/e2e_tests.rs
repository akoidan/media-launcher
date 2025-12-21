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

use serde::Deserialize;
use serde_json::Value;

use std::{
    collections::{BTreeSet, HashMap},
    env,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use fs_access::Fs;

#[derive(Debug, Default)]
struct MockFs {
    dirs: BTreeSet<PathBuf>,
    files: BTreeSet<PathBuf>,
    dir_entries: HashMap<PathBuf, Vec<PathBuf>>,
    file_contents: HashMap<PathBuf, Vec<u8>>,
    writes: Mutex<Vec<(PathBuf, Vec<u8>)>>,
}

impl MockFs {
    fn push_unique(list: &mut Vec<PathBuf>, item: PathBuf) {
        if !list.contains(&item) {
            list.push(item);
        }
    }

    fn add_dir(&mut self, dir: PathBuf) {
        if let Some(parent) = dir.parent() {
            let parent = parent.to_path_buf();
            self.dirs.insert(parent.clone());
            self.dir_entries.entry(parent.clone()).or_default();
            let list = self.dir_entries.entry(parent).or_default();
            Self::push_unique(list, dir.clone());
        }

        self.dirs.insert(dir.clone());
        self.dir_entries.entry(dir).or_default();
    }

    fn add_file(&mut self, file: PathBuf) {
        if let Some(parent) = file.parent() {
            let parent = parent.to_path_buf();
            self.add_dir(parent.clone());
            let list = self.dir_entries.entry(parent).or_default();
            Self::push_unique(list, file.clone());
        }
        self.files.insert(file);
    }

    fn put_input_file(&mut self, file: PathBuf, contents: &[u8]) {
        self.add_file(file.clone());
        self.file_contents.insert(file, contents.to_vec());
    }
}

impl Fs for MockFs {
    fn read_dir_paths(&self, dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut v = self.dir_entries.get(dir).cloned().unwrap_or_default();
        v.sort();
        Ok(v)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains(path)
    }

    fn write(&self, path: &Path, contents: &[u8]) -> anyhow::Result<()> {
        // Writes are the program outputs (scripts). Record them.
        let mut writes = self.writes.lock().unwrap();
        writes.push((path.to_path_buf(), contents.to_vec()));
        Ok(())
    }

    fn canonicalize(&self, path: &Path) -> anyhow::Result<PathBuf> {
        Ok(path.to_path_buf())
    }

    fn set_executable(&self, path: &Path) -> anyhow::Result<()> {
        let _ = path;
        Ok(())
    }
}

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

fn populate_tree(fs: &mut MockFs, base: &Path, node: &Value) {
    match node {
        Value::String(contents) => {
            fs.put_input_file(base.to_path_buf(), contents.as_bytes());
        }
        Value::Object(map) => {
            fs.add_dir(base.to_path_buf());
            for (name, child) in map {
                populate_tree(fs, &base.join(name), child);
            }
        }
        _ => {
            panic!("Invalid fixture node for {}", base.display());
        }
    }
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

    if *expected != actual {
        let mut missing = Vec::new();
        let mut extra = Vec::new();
        let mut mismatched = Vec::new();

        for (k, v_expected) in expected {
            match actual.get(k) {
                None => missing.push(k.clone()),
                Some(v_actual) => {
                    if v_actual != v_expected {
                        mismatched.push(k.clone());
                    }
                }
            }
        }

        for k in actual.keys() {
            if !expected.contains_key(k) {
                extra.push(k.clone());
            }
        }

        let mut msg = String::new();
        use std::fmt::Write as _;
        let _ = writeln!(&mut msg, "Fixture mismatch: {}", fixture_path.display());
        let _ = writeln!(&mut msg, "Expected keys: {}", expected.len());
        let _ = writeln!(&mut msg, "Actual keys: {}", actual.len());
        let _ = writeln!(&mut msg, "Missing keys: {}", missing.len());
        let _ = writeln!(&mut msg, "Extra keys: {}", extra.len());
        let _ = writeln!(&mut msg, "Mismatched contents: {}", mismatched.len());

        for k in missing.iter().take(20) {
            let _ = writeln!(&mut msg, "missing: {k}");
        }
        for k in extra.iter().take(20) {
            let _ = writeln!(&mut msg, "extra: {k}");
        }
        for k in mismatched.iter().take(10) {
            let v_expected = expected.get(k).unwrap();
            let v_actual = actual.get(k).unwrap();
            let _ = writeln!(&mut msg, "mismatch: {k}");
            let _ = writeln!(&mut msg, "--- expected ---\n{v_expected}");
            let _ = writeln!(&mut msg, "--- actual ---\n{v_actual}");
        }

        panic!("{msg}");
    }
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
