use std::path::Path;

use anyhow::Result;

use crate::fs_access::Fs;

pub fn default_pick_dir() -> &'static str {
    "~/"
}

pub fn script_ext() -> &'static str {
    "bash"
}

pub fn decorate_program_name(base: &str) -> String {
    base.to_string()
}

#[allow(dead_code)]
pub fn set_script_permissions(path: &Path) -> Result<()> {
    set_script_permissions_with(&crate::fs_access::RealFs, path)
}

pub fn set_script_permissions_with(fs_access: &impl Fs, path: &Path) -> Result<()> {
    fs_access.set_executable(path)
}
