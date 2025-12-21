use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::ValueEnum;

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
    fn base_name(self) -> &'static str {
        match self {
            PlayerKind::Mpv => "mpv",
            PlayerKind::Vlc => "vlc",
        }
    }

    fn decorated_name(self) -> String {
        os::decorate_program_name(self.base_name())
    }

    fn is_available(self) -> bool {
        os::is_program_in_path(self.base_name())
    }
}

pub fn resolve_player(requested: Option<PlayerKind>) -> Result<Box<dyn Player>> {
    let kind = match requested {
        Some(kind) => {
            if !kind.is_available() {
                return Err(anyhow!("Player '{}' not found in PATH", kind.decorated_name()));
            }
            kind
        }
        None => {
            let mpv_available = PlayerKind::Mpv.is_available();
            let vlc_available = PlayerKind::Vlc.is_available();

            match (mpv_available, vlc_available) {
                (true, _) => PlayerKind::Mpv,
                (false, true) => PlayerKind::Vlc,
                (false, false) => {
                    return Err(anyhow!(
                        "No supported players found in PATH (tried '{}' and '{}')",
                        PlayerKind::Mpv.decorated_name(),
                        PlayerKind::Vlc.decorated_name()
                    ));
                }
            }
        }
    };

    Ok(match kind {
        PlayerKind::Mpv => Box::new(MpvPlayer {}),
        PlayerKind::Vlc => Box::new(VlcPlayer {}),
    })
}


pub trait Player {
    fn program_name(&self) -> String;

    fn build_launch_command(
        &self,
        value: &EpisodeFiles,
        font_dir: &Option<PathBuf>,
    ) -> Option<String> {
        let video = value.video.as_ref()?;
        let mut cmd = format!("{} \"{}\"", self.program_name(), video.display());
        self.append_audio_args(&mut cmd, &value.audio);
        self.append_subtitle_args(&mut cmd, &value.subtitles);
        self.append_font_args(&mut cmd, font_dir);
        Some(cmd)
    }

    fn append_audio_args(&self, cmd: &mut String, audio: &[PathBuf]);
    fn append_subtitle_args(&self, cmd: &mut String, subtitles: &[PathBuf]);
    fn append_font_args(&self, cmd: &mut String, font_dir: &Option<PathBuf>);
}
