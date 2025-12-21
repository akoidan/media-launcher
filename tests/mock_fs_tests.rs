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

use std::{
    collections::{BTreeSet, HashMap},
    env,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use fs_access::Fs;

#[derive(Debug)]
struct MockFs {
    dirs: BTreeSet<PathBuf>,
    files: BTreeSet<PathBuf>,
    dir_entries: HashMap<PathBuf, Vec<PathBuf>>,
    writes: Mutex<HashMap<PathBuf, Vec<u8>>>,
}

impl Default for MockFs {
    fn default() -> Self {
        Self {
            dirs: BTreeSet::new(),
            files: BTreeSet::new(),
            dir_entries: HashMap::new(),
            writes: Mutex::new(HashMap::new()),
        }
    }
}

impl MockFs {
    fn add_dir(&mut self, dir: impl Into<PathBuf>) {
        let dir = dir.into();
        self.dirs.insert(dir.clone());
        self.dir_entries.entry(dir).or_default();
    }

    fn add_file(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        if let Some(parent) = path.parent() {
            let parent = parent.to_path_buf();
            self.add_dir(parent.clone());
            self.dir_entries.entry(parent).or_default().push(path.clone());
        }
        self.files.insert(path);
    }

    fn add_child_dir(&mut self, parent: impl Into<PathBuf>, child: impl Into<PathBuf>) {
        let parent = parent.into();
        let child = child.into();
        self.add_dir(parent.clone());
        self.add_dir(child.clone());
        self.dir_entries.entry(parent).or_default().push(child);
    }
}

impl Fs for MockFs {
    fn read_dir_paths(&self, dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
        Ok(self
            .dir_entries
            .get(dir)
            .cloned()
            .unwrap_or_default())
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains(path)
    }

    fn write(&self, path: &Path, contents: &[u8]) -> anyhow::Result<()> {
        let mut writes = self.writes.lock().unwrap();
        writes.insert(path.to_path_buf(), contents.to_vec());
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

#[test]
fn scan_dir_with_mock_fs_solo_leveling_tv2_groups_episodes() {
    let mut fs = MockFs::default();

    let root = PathBuf::from("root");
    let rus = root.join("RUS Sound");
    let sub = root.join("SUB");

    fs.add_dir(root.clone());
    fs.add_child_dir(root.clone(), rus.clone());
    fs.add_child_dir(root.clone(), sub.clone());

    for ep in 13..=25 {
        let hash = format!("HASH{ep:02}");

        let mkv = root.join(format!(
            "[SubsPlease] Solo Leveling - {ep:02} (1080p) [{hash}].mkv"
        ));
        fs.add_file(mkv);

        let mka = rus.join(format!(
            "[SubsPlease] Solo Leveling - {ep:02} (1080p) [{hash}].mka"
        ));
        fs.add_file(mka);

        let ass = sub.join(format!(
            "[SubsPlease] Solo Leveling - {ep:02} (1080p) [{hash}].ass"
        ));
        fs.add_file(ass);
    }

    let (structure, font_dir) = media_scan::scan_dir_with(&fs, &root).unwrap();

    assert_eq!(font_dir, None);
    assert_eq!(structure.len(), 13);

    for ep in 13..=25 {
        let entry = structure.get(&ep).unwrap();
        assert!(entry.video.is_some());
        assert_eq!(entry.audio.len(), 1);
        assert_eq!(entry.subtitles.len(), 1);
    }
}

#[test]
fn is_program_in_path_with_mock_fs_uses_fs_for_existence() {
    let _guard = env_lock().lock().unwrap();

    let mut fs = MockFs::default();

    let bin1 = PathBuf::from("bin1");
    let bin2 = PathBuf::from("bin2");
    fs.add_dir(bin1.clone());
    fs.add_dir(bin2.clone());

    // Create a "mpv" file in bin2 that matches whatever decorate_program_name does on this OS.
    let decorated = os::decorate_program_name("mpv");
    fs.add_file(bin2.join(decorated));

    let old_path = env::var_os("PATH");
    env::set_var("PATH", set_test_path(&[bin1.clone(), bin2.clone()]));

    let ok = os::is_program_in_path_with(&fs, "mpv");

    if let Some(p) = old_path {
        env::set_var("PATH", p);
    } else {
        env::remove_var("PATH");
    }

    assert!(ok);
}

#[test]
fn app_run_with_mock_fs_writes_scripts() {
    let _guard = env_lock().lock().unwrap();

    let mut fs = MockFs::default();

    let root = PathBuf::from("root");
    let rus = root.join("RUS Sound");
    let sub = root.join("SUB");

    fs.add_dir(root.clone());
    fs.add_child_dir(root.clone(), rus.clone());
    fs.add_child_dir(root.clone(), sub.clone());

    for ep in 13..=14 {
        let hash = format!("HASH{ep:02}");

        fs.add_file(root.join(format!(
            "[SubsPlease] Solo Leveling - {ep:02} (1080p) [{hash}].mkv"
        )));
        fs.add_file(rus.join(format!(
            "[SubsPlease] Solo Leveling - {ep:02} (1080p) [{hash}].mka"
        )));
        fs.add_file(sub.join(format!(
            "[SubsPlease] Solo Leveling - {ep:02} (1080p) [{hash}].ass"
        )));
    }

    let bin = PathBuf::from("bin");
    fs.add_dir(bin.clone());
    fs.add_file(bin.join(os::decorate_program_name("mpv")));

    let old_path = env::var_os("PATH");
    env::set_var("PATH", set_test_path(&[bin.clone()]));

    let args = app::Args {
        root_dir: Some(root.clone()),
        player: Some(players::PlayerKind::Mpv),
    };

    let result = app::run_with(&fs, args);

    if let Some(p) = old_path {
        env::set_var("PATH", p);
    } else {
        env::remove_var("PATH");
    }

    result.unwrap();

    let ext = os::script_ext();
    let writes = fs.writes.lock().unwrap();
    assert!(writes.contains_key(&root.join(format!("13.{ext}"))));
    assert!(writes.contains_key(&root.join(format!("14.{ext}"))));
}
