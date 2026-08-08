use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::ValueEnum;

use crate::fs_access::Fs;
use crate::media_scan::EpisodeFiles;
use crate::os;

mod mpv;
mod vlc;

pub use mpv::MpvPlayer;
pub use vlc::VlcPlayer;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum PlayerKind {
    Mpv,
    Vlc,
}

impl PlayerKind {
    fn key(self) -> &'static str {
        match self {
            PlayerKind::Mpv => "mpv",
            PlayerKind::Vlc => "vlc",
        }
    }
}

#[allow(dead_code)]
pub fn resolve_player(requested: Option<PlayerKind>) -> Result<Box<dyn Player>> {
    resolve_player_with(&crate::fs_access::RealFs, requested)
}

pub fn resolve_player_with(
    fs_access: &impl Fs,
    requested: Option<PlayerKind>,
) -> Result<Box<dyn Player>> {
    let (kind, program_name) = match requested {
        Some(kind) => {
            let program_name = os::resolve_program_name_with(fs_access, kind.key())
                .ok_or_else(|| anyhow!("Player '{}' not found in PATH", kind.key()))?;
            (kind, program_name)
        }
        None => {
            let mpv = os::resolve_program_name_with(fs_access, PlayerKind::Mpv.key());
            let vlc = os::resolve_program_name_with(fs_access, PlayerKind::Vlc.key());

            match (mpv, vlc) {
                (Some(name), _) => (PlayerKind::Mpv, name),
                (None, Some(name)) => (PlayerKind::Vlc, name),
                (None, None) => {
                    return Err(anyhow!(
                        "No supported players found in PATH (tried '{}' and '{}')",
                        PlayerKind::Mpv.key(),
                        PlayerKind::Vlc.key()
                    ));
                }
            }
        }
    };

    Ok(match kind {
        PlayerKind::Mpv => Box::new(MpvPlayer { program_name }),
        PlayerKind::Vlc => Box::new(VlcPlayer { program_name }),
    })
}

pub trait Player {
    fn program_name(&self) -> &str;

    fn build_launch_command(
        &self,
        value: &EpisodeFiles,
        font_dir: &Option<PathBuf>,
    ) -> Result<String> {
        let video = value
            .video
            .as_ref()
            .ok_or_else(|| anyhow!("Main video file not found"))?;
        let mut cmd = format!("{} \"{}\"", self.program_name(), video.display());
        self.append_audio_args(&mut cmd, &value.audio);
        self.append_subtitle_args(&mut cmd, &value.subtitles);
        self.append_font_args(&mut cmd, font_dir);
        Ok(cmd)
    }

    fn append_audio_args(&self, cmd: &mut String, audio: &[PathBuf]);
    fn append_subtitle_args(&self, cmd: &mut String, subtitles: &[PathBuf]);
    fn append_font_args(&self, cmd: &mut String, font_dir: &Option<PathBuf>);
}
