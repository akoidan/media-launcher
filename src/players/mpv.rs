use std::path::PathBuf;

use super::Player;

pub struct MpvPlayer {
}

impl Player for MpvPlayer {
    fn program_name(&self) -> &'static str {
        #[cfg(windows)]
        {
            return "mpv.exe";
        }

        #[cfg(not(windows))]
        {
            return "mpv";
        }
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
