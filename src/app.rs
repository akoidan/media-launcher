use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::fs_access::{Fs, RealFs};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(value_parser = crate::media_scan::validate_root_dir)]
    pub root_dir: Option<PathBuf>,

    #[arg(long, value_enum)]
    pub player: Option<crate::players::PlayerKind>,
}

pub fn run(args: Args) -> Result<()> {
    run_with(&RealFs, args)
}

pub fn run_with(fs_access: &impl Fs, args: Args) -> Result<()> {
    let root_dir = match args.root_dir {
        Some(p) => fs_access.canonicalize(&p)?,
        None => crate::media_scan::pick_directory()?,
    };

    let player = crate::players::resolve_player_with(fs_access, args.player)?;

    let (structure, font_dir) = crate::media_scan::scan_dir_with(fs_access, &root_dir)?;

    for (episode, value) in structure {
        let open_cmd = match player.build_launch_command(&value, &font_dir) {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Skipping episode {episode}: {e}");
                continue;
            }
        };

        crate::media_scan::write_episode_script_with(fs_access, &root_dir, episode, &open_cmd)?;
    }

    Ok(())
}
