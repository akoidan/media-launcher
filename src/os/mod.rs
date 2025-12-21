#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod linux;

use std::env;

use crate::fs_access::{Fs, RealFs};

#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
pub use linux::*;

pub fn is_program_in_path(base: &str) -> bool {
    is_program_in_path_with(&RealFs, base)
}

pub fn is_program_in_path_with(fs_access: &impl Fs, base: &str) -> bool {
    let program = decorate_program_name(base);

    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    for dir in env::split_paths(&path) {
        if fs_access.is_file(&dir.join(&program)) {
            return true;
        }
    }

    false
}
