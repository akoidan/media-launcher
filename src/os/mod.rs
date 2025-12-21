#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod linux;

use std::env;

#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
pub use linux::*;

pub fn is_program_in_path(base: &str) -> bool {
    let program = decorate_program_name(base);

    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    for dir in env::split_paths(&path) {
        if dir.join(&program).is_file() {
            return true;
        }
    }

    false
}
