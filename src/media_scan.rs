use std::{
    collections::BTreeMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use rfd::FileDialog;

use crate::fs_access::{Fs, RealFs};
use crate::os;

static RE_E: OnceLock<Regex> = OnceLock::new();
static RE_2D: OnceLock<Regex> = OnceLock::new();

fn re_e() -> &'static Regex {
    RE_E.get_or_init(|| Regex::new(r"E(\d\d)").expect("Invalid RE_E regex"))
}

fn re_2d() -> &'static Regex {
    RE_2D.get_or_init(|| Regex::new(r"\d\d").expect("Invalid RE_2D regex"))
}

#[derive(Debug, Default)]
pub struct EpisodeFiles {
    pub audio: Vec<PathBuf>,
    pub subtitles: Vec<PathBuf>,
    pub video: Option<PathBuf>,
}

pub fn pick_directory() -> Result<PathBuf> {
    let path = FileDialog::new()
        .set_directory(os::default_pick_dir())
        .pick_folder()
        .ok_or_else(|| anyhow!("No directory selected"))?;

    Ok(path)
}

pub fn validate_root_dir(s: &str) -> std::result::Result<PathBuf, String> {
    if s == "pick" {
        return pick_directory().map_err(|e| e.to_string());
    }

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
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mkv" | "ass" | "mka" | "ttf"
    )
}

fn extract_episode_number(file_name: &str) -> Option<u32> {
    if let Some(caps) = re_e().captures(file_name) {
        return caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok());
    }

    re_2d()
        .find(file_name)
        .and_then(|m| m.as_str().parse::<u32>().ok())
}

fn handle_media_file(path: PathBuf, structure: &mut BTreeMap<u32, EpisodeFiles>) -> Result<()> {
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

    let Some(episode) = extract_episode_number(file_name) else {
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

pub fn scan_dir(dir_path: &Path) -> Result<(BTreeMap<u32, EpisodeFiles>, Option<PathBuf>)> {
    scan_dir_with(&RealFs, dir_path)
}

pub fn scan_dir_with(
    fs_access: &impl Fs,
    dir_path: &Path,
) -> Result<(BTreeMap<u32, EpisodeFiles>, Option<PathBuf>)> {
    let mut structure: BTreeMap<u32, EpisodeFiles> = BTreeMap::new();
    let mut font_dir: Option<PathBuf> = None;

    read_dir_recursive(fs_access, dir_path, &mut structure, &mut font_dir)?;
    Ok((structure, font_dir))
}

pub fn write_episode_script(root_dir: &Path, episode: u32, open_cmd: &str) -> Result<()> {
    write_episode_script_with(&RealFs, root_dir, episode, open_cmd)
}

pub fn write_episode_script_with(
    fs_access: &impl Fs,
    root_dir: &Path,
    episode: u32,
    open_cmd: &str,
) -> Result<()> {
    let script_path = root_dir.join(format!("{:02}.{}", episode, os::script_ext()));
    fs_access.write(&script_path, open_cmd.as_bytes())?;
    os::set_script_permissions_with(fs_access, &script_path)?;
    println!("{}", script_path.display());
    Ok(())
}

fn read_dir_recursive(
    fs_access: &impl Fs,
    dir_path: &Path,
    structure: &mut BTreeMap<u32, EpisodeFiles>,
    font_dir: &mut Option<PathBuf>,
) -> Result<()> {
    let entries = fs_access
        .read_dir_paths(dir_path)
        .with_context(|| format!("Failed to read directory {}", dir_path.display()))?;

    for entry in entries {
        let path = entry;
        if fs_access.is_dir(&path) {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_ascii_lowercase());
            let name = name.as_deref().unwrap_or("");
            if name.contains("font") || name.contains("шрифты") {
                *font_dir = Some(path);
            } else {
                read_dir_recursive(fs_access, &path, structure, font_dir)?;
            }
            continue;
        }

        if fs_access.is_file(&path) {
            handle_media_file(path, structure)?;
        }
    }

    Ok(())
}
