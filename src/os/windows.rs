use std::path::Path;

use anyhow::Result;

use super::PlayerLocation;
use crate::fs_access::Fs;

pub fn default_pick_dir() -> &'static str {
    "C:\\\\"
}

pub fn script_ext() -> &'static str {
    "cmd"
}

pub fn script_header() -> &'static str {
    ""
}

pub fn player_locations(player: &str) -> Vec<PlayerLocation> {
    match player {
        "vlc" => vec![
            PlayerLocation::Path("vlc.exe".into()),
            PlayerLocation::Absolute(r"C:\Program Files\VideoLAN\VLC\vlc.exe".into()),
            PlayerLocation::Absolute(r"C:\Program Files (x86)\VideoLAN\VLC\vlc.exe".into()),
        ],
        "mpv" => vec![
            PlayerLocation::Path("mpv.exe".into()),
            PlayerLocation::Absolute(r"C:\Program Files\mpv\mpv.exe".into()),
        ],
        _ => vec![PlayerLocation::Path(player.into())],
    }
}

#[allow(dead_code)]
pub fn set_script_permissions(_path: &Path) -> Result<()> {
    set_script_permissions_with(&crate::fs_access::RealFs, _path)
}

pub fn set_script_permissions_with(_fs_access: &impl Fs, _path: &Path) -> Result<()> {
    Ok(())
}
