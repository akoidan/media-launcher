use std::path::Path;

use anyhow::Result;

use super::PlayerLocation;
use crate::fs_access::Fs;

pub fn default_pick_dir() -> &'static str {
    "~/"
}

pub fn script_ext() -> &'static str {
    "bash"
}

pub fn script_header() -> &'static str {
    "#!/bin/bash\n"
}

// Distro package managers put binaries straight on PATH, so that's the only
// location worth checking.
pub fn player_locations(player: &str) -> Vec<PlayerLocation> {
    vec![PlayerLocation::Path(player.into())]
}

#[allow(dead_code)]
pub fn set_script_permissions(path: &Path) -> Result<()> {
    set_script_permissions_with(&crate::fs_access::RealFs, path)
}

pub fn set_script_permissions_with(fs_access: &impl Fs, path: &Path) -> Result<()> {
    fs_access.set_executable(path)
}
