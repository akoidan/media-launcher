use std::path::Path;

use anyhow::Result;

use crate::fs_access::Fs;

pub fn default_pick_dir() -> &'static str {
    "C:\\\\"
}

pub fn script_ext() -> &'static str {
    "cmd"
}

pub fn decorate_program_name(base: &str) -> String {
    format!("{base}.exe")
}

pub fn set_script_permissions(_path: &Path) -> Result<()> {
    set_script_permissions_with(&crate::fs_access::RealFs, _path)
}

pub fn set_script_permissions_with(_fs_access: &impl Fs, _path: &Path) -> Result<()> {
    Ok(())
}
