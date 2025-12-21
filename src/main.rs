use std::{collections::BTreeMap, fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

mod media_scan;
mod os;
mod players;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Root directory to scan (episode folder)
    #[arg(value_parser = media_scan::validate_root_dir)]
    root_dir: Option<PathBuf>,

    #[arg(long, value_enum)]
    player: Option<players::PlayerKind>,
}


fn main() -> Result<()> {
    let args = Args::parse();

    let selected_root_dir = match args.root_dir {
        Some(p) => p,
        None => media_scan::pick_directory()?,
    };

    let root_dir = fs::canonicalize(&selected_root_dir)
        .with_context(|| format!("Failed to resolve {}", selected_root_dir.display()))?;

    let player = players::resolve_player(args.player)?;

    let mut structure: BTreeMap<u32, media_scan::EpisodeFiles> = BTreeMap::new();
    let mut font_dir: Option<PathBuf> = None;

    media_scan::read_dir_recursive(&root_dir, &mut structure, &mut font_dir)?;

    for (episode, value) in structure {
        let Some(open_cmd) = player.build_launch_command(&value, &font_dir) else {
            continue;
        };

        let script_path = root_dir.join(format!("{:02}.{}", episode, os::script_ext()));
        fs::write(&script_path, open_cmd)?;

        os::set_script_permissions(&script_path)?;

        println!("{}", script_path.display());
    }

    Ok(())
}
