use std::path::Path;

use anyhow::Result;

use crate::fs_access::{Fs, RealFs};

pub fn default_pick_dir() -> &'static str {
    "~/"
}

pub fn script_ext() -> &'static str {
    "bash"
}

pub fn decorate_program_name(base: &str) -> String {
    base.to_string()
}

pub fn set_script_permissions(path: &Path) -> Result<()> {
    RealFs.set_executable(path)
}
