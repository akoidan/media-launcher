use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use regex::Regex;

mod players;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Root directory to scan (episode folder)
    #[arg(value_parser = validate_root_dir)]
    root_dir: PathBuf,

    #[arg(long, value_enum, default_value_t = players::PlayerKind::Mpv)]
    player: players::PlayerKind,
}

#[derive(Debug, Default)]
struct EpisodeFiles {
    audio: Vec<PathBuf>,
    subtitles: Vec<PathBuf>,
    video: Option<PathBuf>,
}

fn validate_root_dir(s: &str) -> std::result::Result<PathBuf, String> {
    let p = PathBuf::from(s);
    if !p.exists() {
        return Err(format!("Path does not exist: {s}"));
    }
    if !p.is_dir() {
        return Err(format!("Not a directory: {s}"));
    }
    Ok(p)
}

fn is_script_file(path: &Path) -> bool {
    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");
    matches!(ext.to_ascii_lowercase().as_str(), "cmd" | "bash")
}

fn is_supported_file(path: &Path) -> bool {
    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");
    matches!(ext.to_ascii_lowercase().as_str(), "mkv" | "ass" | "mka" | "ttf")
}

fn extract_episode_number(file_name: &str, re_e: &Regex, re_2d: &Regex) -> Option<u32> {
    if let Some(caps) = re_e.captures(file_name) {
        return caps
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok());
    }

    re_2d
        .find(file_name)
        .and_then(|m| m.as_str().parse::<u32>().ok())
}

fn handle_media_file(
    path: PathBuf,
    structure: &mut BTreeMap<u32, EpisodeFiles>,
    re_e: &Regex,
    re_2d: &Regex,
) -> Result<()> {
    if is_script_file(&path) {
        return Ok(());
    }

    if !is_supported_file(&path) {
        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("<unknown>");
        eprintln!("Unkown file {name}");
        return Ok(());
    }

    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("Cannot read file name for {}", path.display()))?;

    let Some(episode) = extract_episode_number(file_name, re_e, re_2d) else {
        eprintln!("file {file_name} doesn't have epoisode #");
        return Ok(());
    };

    let slot = structure.entry(episode).or_default();

    match path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "mkv" => slot.video = Some(path),
        "ass" => slot.subtitles.push(path),
        "mka" => slot.audio.push(path),
        _ => {
            // ignore (e.g. ttf)
        }
    }

    Ok(())
}

fn read_dir_recursive(
    dir_path: &Path,
    structure: &mut BTreeMap<u32, EpisodeFiles>,
    font_dir: &mut Option<PathBuf>,
    re_e: &Regex,
    re_2d: &Regex,
) -> Result<()> {
    let entries = fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read directory {}", dir_path.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy().to_ascii_lowercase();
            if name.contains("font") || name.contains("шрифты") {
                *font_dir = Some(path);
            } else {
                read_dir_recursive(&path, structure, font_dir, re_e, re_2d)?;
            }
            continue;
        }

        if file_type.is_file() {
            handle_media_file(path, structure, re_e, re_2d)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let root_dir = fs::canonicalize(&args.root_dir)
        .with_context(|| format!("Failed to resolve {}", args.root_dir.display()))?;

    let script_ext = if cfg!(windows) { "cmd" } else { "bash" };

    let player = players::create_player(args.player);

    let re_e = Regex::new(r"E(\d\d)")?;
    let re_2d = Regex::new(r"\d\d")?;

    let mut structure: BTreeMap<u32, EpisodeFiles> = BTreeMap::new();
    let mut font_dir: Option<PathBuf> = None;

    read_dir_recursive(&root_dir, &mut structure, &mut font_dir, &re_e, &re_2d)?;

    for (episode, value) in structure {
        let Some(open_cmd) = player.build_launch_command(&value, &font_dir) else {
            continue;
        };

        let script_path = root_dir.join(format!("{:02}.{}", episode, script_ext));
        fs::write(&script_path, open_cmd)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perm = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&script_path, perm)?;
        }
    }

    Ok(())
}
