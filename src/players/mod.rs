use std::path::PathBuf;

use crate::EpisodeFiles;

use clap::ValueEnum;

mod mpv;
mod vlc;

pub use mpv::MpvPlayer;
pub use vlc::VlcPlayer;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum PlayerKind {
    Mpv,
    Vlc,
}

pub fn create_player(kind: PlayerKind) -> Box<dyn Player> {
    match kind {
        PlayerKind::Mpv => Box::new(MpvPlayer {}),
        PlayerKind::Vlc => Box::new(VlcPlayer {}),
    }
}

pub trait Player {
    fn program_name(&self) -> &'static str;

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
