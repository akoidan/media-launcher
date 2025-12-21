use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use vfs::{MemoryFS, VfsFileType, VfsPath};

#[derive(Debug)]
pub struct VfsFs {
    root: VfsPath,
    pub writes: Mutex<Vec<(PathBuf, Vec<u8>)>>,
}

impl VfsFs {
    pub fn new() -> Self {
        Self {
            root: VfsPath::new(MemoryFS::new()),
            writes: Mutex::new(Vec::new()),
        }
    }

    fn norm(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    fn from_vfs_path(p: &VfsPath) -> PathBuf {
        PathBuf::from(p.as_str().trim_start_matches('/'))
    }

    fn vpath(&self, path: &Path) -> anyhow::Result<VfsPath> {
        Ok(self.root.join(Self::norm(path))?)
    }

    pub fn ensure_dir(&self, path: &Path) -> anyhow::Result<()> {
        self.vpath(path)?.create_dir_all()?;
        Ok(())
    }

    pub fn create_file(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            self.ensure_dir(parent)?;
        }
        let file = self.vpath(path)?;
        let mut w = file.create_file()?;
        w.write_all(b"")?;
        Ok(())
    }

    pub fn put_file(&self, path: &Path, contents: &[u8]) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            self.ensure_dir(parent)?;
        }
        let file = self.vpath(path)?;
        let mut w = file.create_file()?;
        w.write_all(contents)?;
        Ok(())
    }
}

impl Default for VfsFs {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::fs_access::Fs for VfsFs {
    fn read_dir_paths(&self, dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let iter = self.vpath(dir)?.read_dir()?;
        Ok(iter.map(|p| Self::from_vfs_path(&p)).collect())
    }

    fn is_dir(&self, path: &Path) -> bool {
        match self.vpath(path).and_then(|p| Ok(p.metadata()?)) {
            Ok(m) => m.file_type == VfsFileType::Directory,
            Err(_) => false,
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        match self.vpath(path).and_then(|p| Ok(p.metadata()?)) {
            Ok(m) => m.file_type == VfsFileType::File,
            Err(_) => false,
        }
    }

    fn write(&self, path: &Path, contents: &[u8]) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            self.ensure_dir(parent)?;
        }

        let file = self.vpath(path)?;
        let mut w = file.create_file()?;
        w.write_all(contents)?;

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
