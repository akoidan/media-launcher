use std::path::PathBuf;

use anyhow::{anyhow, Result};

use super::Player;
use crate::media_scan::EpisodeFiles;

pub struct PotPlayerPlayer {
    pub program_name: String,
}

impl Player for PotPlayerPlayer {
    fn program_name(&self) -> &str {
        &self.program_name
    }

    // PotPlayer needs /same before the video, with audio files listed as
    // bare extra content args (not per-file flags) for them to load as
    // selectable streams of the same item, so the default build order
    // (video, then flag-based audio/subtitle args) doesn't fit here.
    fn build_launch_command(
        &self,
        value: &EpisodeFiles,
        font_dir: &Option<PathBuf>,
    ) -> Result<String> {
        let video = value
            .video
            .as_ref()
            .ok_or_else(|| anyhow!("Main video file not found"))?;
        let mut cmd = format!("\"{}\" /same \"{}\"", self.program_name(), video.display());
        self.append_audio_args(&mut cmd, &value.audio);
        self.append_subtitle_args(&mut cmd, &value.subtitles);
        self.append_font_args(&mut cmd, font_dir);
        Ok(cmd)
    }

    fn append_audio_args(&self, cmd: &mut String, audio: &[PathBuf]) {
        for a in audio {
            cmd.push_str(&format!(" \"{}\"", a.display()));
        }
    }

    fn append_subtitle_args(&self, cmd: &mut String, subtitles: &[PathBuf]) {
        for s in subtitles {
            cmd.push_str(&format!(" /sub=\"{}\"", s.display()));
        }
    }

    fn append_font_args(&self, _cmd: &mut String, _font_dir: &Option<PathBuf>) {}
}
