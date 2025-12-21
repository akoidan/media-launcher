use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use rfd::FileDialog;

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

pub fn read_dir_recursive(
    dir_path: &Path,
    structure: &mut BTreeMap<u32, EpisodeFiles>,
    font_dir: &mut Option<PathBuf>,
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
                read_dir_recursive(&path, structure, font_dir)?;
            }
            continue;
        }

        if file_type.is_file() {
            handle_media_file(path, structure)?;
        }
    }

    Ok(())
}
