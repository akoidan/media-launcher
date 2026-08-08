use std::path::Path;

use anyhow::Result;

use super::PlayerLocation;
use crate::fs_access::Fs;

pub fn default_pick_dir() -> &'static str {
    "~/"
}

// Finder runs .command files in Terminal.app on double-click with no setup;
// .bash/.sh have no default handler and often open in a text editor instead.
pub fn script_ext() -> &'static str {
    "command"
}

pub fn script_header() -> &'static str {
    "#!/bin/bash\n"
}

// GUI apps like VLC/mpv ship as .app bundles, not PATH binaries. Symlinking
// the bundled binary into PATH breaks VLC specifically (NSBundle resolves
// relative to the symlink, not the bundle), so a plain PATH install (e.g.
// Homebrew) is tried first, falling back to the bundled binary otherwise.
pub fn player_locations(player: &str) -> Vec<PlayerLocation> {
    match player {
        "vlc" => vec![
            PlayerLocation::Path("vlc".into()),
            PlayerLocation::Absolute("/Applications/VLC.app/Contents/MacOS/VLC".into()),
        ],
        "mpv" => vec![
            PlayerLocation::Path("mpv".into()),
            PlayerLocation::Absolute("/Applications/mpv.app/Contents/MacOS/mpv".into()),
        ],
        _ => vec![PlayerLocation::Path(player.into())],
    }
}

#[allow(dead_code)]
pub fn set_script_permissions(path: &Path) -> Result<()> {
    set_script_permissions_with(&crate::fs_access::RealFs, path)
}

pub fn set_script_permissions_with(fs_access: &impl Fs, path: &Path) -> Result<()> {
    fs_access.set_executable(path)
}
