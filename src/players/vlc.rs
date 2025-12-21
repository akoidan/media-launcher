use std::path::PathBuf;

use super::Player;

pub struct VlcPlayer {
}

impl Player for VlcPlayer {
    fn program_name(&self) -> &'static str {
        #[cfg(windows)]
        {
            return "vlc.exe";
        }

        #[cfg(not(windows))]
        {
            return "vlc";
        }
    }

    fn append_audio_args(&self, cmd: &mut String, audio: &[PathBuf]) {
        for a in audio {
            cmd.push_str(&format!(" --input-slave=\"{}\"", a.display()));
        }
    }

    fn append_subtitle_args(&self, cmd: &mut String, subtitles: &[PathBuf]) {
        for s in subtitles {
            cmd.push_str(&format!(" --sub-file=\"{}\"", s.display()));
        }
    }

    fn append_font_args(&self, _cmd: &mut String, _font_dir: &Option<PathBuf>) {}
}
