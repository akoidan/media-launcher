use std::path::PathBuf;

use super::Player;

pub struct MpvPlayer {
    pub program_name: String,
}

impl Player for MpvPlayer {
    fn program_name(&self) -> &str {
        &self.program_name
    }

    fn append_audio_args(&self, cmd: &mut String, audio: &[PathBuf]) {
        for a in audio {
            cmd.push_str(&format!(" --audio-file=\"{}\"", a.display()));
        }
    }

    fn append_subtitle_args(&self, cmd: &mut String, subtitles: &[PathBuf]) {
        for s in subtitles {
            cmd.push_str(&format!(" --sub-file=\"{}\"", s.display()));
        }
    }

    fn append_font_args(&self, cmd: &mut String, font_dir: &Option<PathBuf>) {
        if let Some(font_dir) = font_dir {
            cmd.push_str(&format!(" --sub-fonts-dir=\"{}\"", font_dir.display()));
        }
    }
}
