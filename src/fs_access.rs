use std::path::{Path, PathBuf};

use anyhow::Result;

pub trait Fs {
    fn read_dir_paths(&self, dir: &Path) -> Result<Vec<PathBuf>>;
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn write(&self, path: &Path, contents: &[u8]) -> Result<()>;
    fn canonicalize(&self, path: &Path) -> Result<PathBuf>;
    #[allow(dead_code)]
    fn set_executable(&self, path: &Path) -> Result<()>;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RealFs;

impl Fs for RealFs {
    fn read_dir_paths(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            out.push(entry?.path());
        }
        Ok(out)
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn write(&self, path: &Path, contents: &[u8]) -> Result<()> {
        std::fs::write(path, contents)?;
        Ok(())
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        // std::fs::canonicalize returns \\?\ verbatim paths on Windows, which
        // external players like VLC choke on. Strip the prefix back to a
        // regular drive/UNC path since these are ordinary local paths.
        let canonical = std::fs::canonicalize(path)?;
        let s = canonical.to_string_lossy();
        Ok(if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            PathBuf::from(format!(r"\\{rest}"))
        } else if let Some(rest) = s.strip_prefix(r"\\?\") {
            PathBuf::from(rest)
        } else {
            canonical
        })
    }

    fn set_executable(&self, path: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perm = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(path, perm)?;
            Ok(())
        }

        #[cfg(not(unix))]
        {
            let _ = path;
            Ok(())
        }
    }
}
