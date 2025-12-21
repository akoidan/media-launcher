use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde_json::Value;

use crate::fs_access::Fs;

#[derive(Debug, Default)]
pub struct MockFs {
    pub dirs: BTreeSet<PathBuf>,
    pub files: BTreeSet<PathBuf>,
    pub dir_entries: HashMap<PathBuf, Vec<PathBuf>>,
    pub file_contents: HashMap<PathBuf, Vec<u8>>,
    pub writes: Mutex<Vec<(PathBuf, Vec<u8>)>>,
}

impl MockFs {
    fn push_unique(list: &mut Vec<PathBuf>, item: PathBuf) {
        if !list.contains(&item) {
            list.push(item);
        }
    }

    pub fn add_dir(&mut self, dir: PathBuf) {
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

    pub fn add_file(&mut self, file: PathBuf) {
        if let Some(parent) = file.parent() {
            let parent = parent.to_path_buf();
            self.add_dir(parent.clone());
            let list = self.dir_entries.entry(parent).or_default();
            Self::push_unique(list, file.clone());
        }
        self.files.insert(file);
    }

    pub fn put_input_file(&mut self, file: PathBuf, contents: &[u8]) {
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

pub fn populate_tree(fs: &mut MockFs, base: &Path, node: &Value) {
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
