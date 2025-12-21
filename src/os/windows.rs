use std::path::Path;

use anyhow::Result;

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
    Ok(())
}
