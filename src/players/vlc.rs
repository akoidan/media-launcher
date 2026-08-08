use std::path::PathBuf;

use super::Player;

pub struct VlcPlayer {
    pub program_name: String,
}

impl Player for VlcPlayer {
    fn program_name(&self) -> &str {
        &self.program_name
    }

    fn append_audio_args(&self, cmd: &mut String, audio: &[PathBuf]) {
        // VLC's --input-slave is a single-value option: repeating the flag
        // overwrites the previous value instead of appending, so all slaves
        // must be combined into one value separated by '#'.
        if !audio.is_empty() {
            let joined = audio
                .iter()
                .map(|a| a.display().to_string())
                .collect::<Vec<_>>()
                .join("#");
            cmd.push_str(&format!(" --input-slave=\"{joined}\""));
        }
    }

    fn append_subtitle_args(&self, cmd: &mut String, subtitles: &[PathBuf]) {
        for s in subtitles {
            cmd.push_str(&format!(" --sub-file=\"{}\"", s.display()));
        }
    }

    fn append_font_args(&self, _cmd: &mut String, _font_dir: &Option<PathBuf>) {}
}
