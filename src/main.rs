use anyhow::Result;
use clap::Parser;

mod app;
mod fs_access;
mod media_scan;
mod os;
mod players;

fn main() -> Result<()> {
    app::run(app::Args::parse())
}
