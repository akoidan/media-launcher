use std::{fs, path::PathBuf};

use anyhow::Result;
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

    let root_dir = match args.root_dir {
        Some(p) => fs::canonicalize(p)?,
        None => media_scan::pick_directory()?,
    };

    let player = players::resolve_player(args.player)?;

    let (structure, font_dir) = media_scan::scan_dir(&root_dir)?;

    for (episode, value) in structure {
        let open_cmd = match player.build_launch_command(&value, &font_dir) {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Skipping episode {episode}: {e}");
                continue;
            }
        };

        let script_path = root_dir.join(format!("{:02}.{}", episode, os::script_ext()));
        fs::write(&script_path, open_cmd)?;

        os::set_script_permissions(&script_path)?;

        println!("{}", script_path.display());
    }

    Ok(())
}
