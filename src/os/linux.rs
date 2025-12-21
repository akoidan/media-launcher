use std::{fs, path::Path};

use anyhow::Result;

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
    use std::os::unix::fs::PermissionsExt;

    let perm = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, perm)?;
    Ok(())
}
