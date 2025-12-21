use std::path::PathBuf;

use super::Player;
use crate::os;

pub struct VlcPlayer {}

impl Player for VlcPlayer {
    fn program_name(&self) -> String {
        os::decorate_program_name("vlc")
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
