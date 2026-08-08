#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(any(windows, target_os = "macos")))]
mod linux;

use std::env;
use std::path::Path;

use crate::fs_access::{Fs, RealFs};

#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(any(windows, target_os = "macos")))]
pub use linux::*;

/// One place to look for a player's executable, in the order each OS module
/// wants them tried.
pub enum PlayerLocation {
    /// Search every directory on $PATH for this executable name.
    Path(String),
    /// Check this exact absolute path (e.g. a known GUI app bundle location).
    /// Unused on Linux, which only ever looks on PATH.
    #[allow(dead_code)]
    Absolute(String),
}

#[allow(dead_code)]
pub fn resolve_program_name(player: &str) -> Option<String> {
    resolve_program_name_with(&RealFs, player)
}

/// Resolves a player to the name/path that should be used to launch it: the
/// bare PATH name if found there, otherwise the first hardcoded install
/// location (per player_locations) that actually exists.
pub fn resolve_program_name_with(fs_access: &impl Fs, player: &str) -> Option<String> {
    for location in player_locations(player) {
        match location {
            PlayerLocation::Path(name) => {
                if is_on_path_with(fs_access, &name) {
                    return Some(name);
                }
            }
            PlayerLocation::Absolute(path) => {
                if fs_access.is_file(Path::new(&path)) {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn is_on_path_with(fs_access: &impl Fs, name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|dir| fs_access.is_file(&dir.join(name)))
}

/// The bare PATH-style executable name for a player on this OS (e.g.
/// "vlc.exe" on Windows). Used by tests to place a mock binary on PATH.
#[allow(dead_code)]
pub fn path_program_name(player: &str) -> String {
    player_locations(player)
        .into_iter()
        .find_map(|loc| match loc {
            PlayerLocation::Path(name) => Some(name),
            PlayerLocation::Absolute(_) => None,
        })
        .expect("every player defines a PATH location")
}
