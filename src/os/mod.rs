#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod linux;

#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
pub use linux::*;
