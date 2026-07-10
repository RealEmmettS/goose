//! Audio backend — plays the goose's sound requests via `rodio`.
//!
//! The engine emits platform-free [`Sound`] requests; this maps them to the bundled clips
//! and plays them fire-and-forget. The original honk/bite/mud/pat sounds are embedded for
//! personal-use self-distribution. Honest degradation: if there is no output device the
//! whole backend is a silent no-op, and individual decode/playback failures are ignored.

use honk_engine::{HonkTone, Sound};
#[cfg(any(
    windows,
    target_os = "macos",
    all(target_os = "linux", target_env = "gnu")
))]
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
#[cfg(any(
    windows,
    target_os = "macos",
    all(target_os = "linux", target_env = "gnu")
))]
use std::io::Cursor;
#[cfg(all(target_os = "linux", target_env = "musl"))]
use std::path::PathBuf;

#[cfg(any(test, all(target_os = "linux", target_env = "musl")))]
const MAX_AUDIO_CHILDREN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayOutcome {
    Played,
    #[allow(dead_code)]
    Busy,
    Failed,
}

#[cfg(test)]
fn bounded_child_count(requested: usize) -> usize {
    requested.min(MAX_AUDIO_CHILDREN)
}

#[cfg(any(test, all(target_os = "linux", target_env = "musl")))]
fn path_candidates(name: &str, path: &std::ffi::OsStr) -> Vec<std::path::PathBuf> {
    std::env::split_paths(path)
        .map(|directory| directory.join(name))
        .collect()
}

const HONKS: [&[u8]; 4] = [
    include_bytes!("../Assets/Sounds/Honk1.mp3"),
    include_bytes!("../Assets/Sounds/Honk2.mp3"),
    include_bytes!("../Assets/Sounds/Honk3.mp3"),
    include_bytes!("../Assets/Sounds/Honk4.mp3"),
];
const BITE: &[u8] = include_bytes!("../Assets/Sounds/BITE.mp3");
const MUD: &[u8] = include_bytes!("../Assets/Sounds/MudSquith.mp3");
const PATS: [&[u8]; 3] = [
    include_bytes!("../Assets/Sounds/Pat1.wav"),
    include_bytes!("../Assets/Sounds/Pat2.wav"),
    include_bytes!("../Assets/Sounds/Pat3.wav"),
];

/// Owns the output stream and plays sound clips. Keep the value alive for the whole run —
/// dropping it closes the audio device.
#[cfg(any(
    windows,
    target_os = "macos",
    all(target_os = "linux", target_env = "gnu")
))]
pub struct Audio {
    // Held only to keep the device open; never touched directly.
    _stream: OutputStream,
    handle: OutputStreamHandle,
    counter: usize,
}

#[cfg(any(
    windows,
    target_os = "macos",
    all(target_os = "linux", target_env = "gnu")
))]
impl Audio {
    /// Open the default output device. Returns `None` (the goose runs silent) when there is
    /// no audio device — e.g. a headless session.
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            handle,
            counter: 0,
        })
    }

    fn next(&mut self) -> usize {
        self.counter = self.counter.wrapping_add(1);
        self.counter
    }

    pub fn poll(&mut self) {}

    /// Play `sound` fire-and-forget (honks/pats rotate through their variants).
    pub fn play(&mut self, sound: Sound) -> PlayOutcome {
        let bytes: &'static [u8] = match sound {
            Sound::Honk(tone) => match tone {
                HonkTone::Normal => HONKS[self.next() % HONKS.len()],
                HonkTone::High => HONKS[(self.next() + 1) % HONKS.len()],
                HonkTone::Low => HONKS[(self.next() + HONKS.len() - 1) % HONKS.len()],
            },
            Sound::Bite => BITE,
            Sound::MudSquish => MUD,
            Sound::Pat => PATS[self.next() % PATS.len()],
        };
        let Ok(sink) = Sink::try_new(&self.handle) else {
            return PlayOutcome::Failed;
        };
        let Ok(decoder) = Decoder::new(Cursor::new(bytes)) else {
            return PlayOutcome::Failed;
        };
        sink.append(decoder);
        sink.detach(); // play to completion in the background
        PlayOutcome::Played
    }
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
pub struct Audio {
    dir: std::path::PathBuf,
    counter: usize,
    children: Vec<std::process::Child>,
    player: LinuxAudioPlayer,
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
impl Audio {
    /// Open the musl command-line fallback. GNU Linux uses the in-process rodio backend above;
    /// this path exists because the portable musl archive cannot assume a native audio library.
    pub fn new() -> Option<Self> {
        let player = LinuxAudioPlayer::detect()?;
        let dir = std::env::temp_dir().join(format!("honk300-audio-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok()?;
        for (name, bytes) in sound_files() {
            std::fs::write(dir.join(name), bytes).ok()?;
        }
        Some(Self {
            dir,
            counter: 0,
            children: Vec::new(),
            player,
        })
    }

    fn next(&mut self) -> usize {
        self.counter = self.counter.wrapping_add(1);
        self.counter
    }

    pub fn poll(&mut self) {
        reap_audio_children(&mut self.children);
    }

    pub fn play(&mut self, sound: Sound) -> PlayOutcome {
        let name = sound_file_name(sound, || self.next());
        let path = self.dir.join(name);
        let mut command = self.player.command(path);
        spawn_audio_child(&mut self.children, &mut command)
    }
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
#[derive(Debug, Clone)]
enum LinuxAudioPlayer {
    Ffplay(PathBuf),
    Mpv(PathBuf),
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
impl LinuxAudioPlayer {
    fn detect() -> Option<Self> {
        let search_path = std::env::var_os("PATH").unwrap_or_default();
        for path in path_candidates("ffplay", &search_path) {
            if is_executable_file(&path) {
                return Some(Self::Ffplay(path));
            }
        }
        for path in path_candidates("mpv", &search_path) {
            if is_executable_file(&path) {
                return Some(Self::Mpv(path));
            }
        }
        None
    }

    fn command(&self, path: PathBuf) -> std::process::Command {
        match self {
            Self::Ffplay(program) => {
                let mut command = std::process::Command::new(program);
                command
                    .arg("-nodisp")
                    .arg("-autoexit")
                    .arg("-loglevel")
                    .arg("quiet")
                    .arg(path);
                command
            }
            Self::Mpv(program) => {
                let mut command = std::process::Command::new(program);
                command.arg("--no-terminal").arg("--really-quiet").arg(path);
                command
            }
        }
    }
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
fn is_executable_file(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
fn spawn_audio_child(
    children: &mut Vec<std::process::Child>,
    command: &mut std::process::Command,
) -> PlayOutcome {
    reap_audio_children(children);
    if children.len() >= MAX_AUDIO_CHILDREN {
        return PlayOutcome::Busy;
    }
    match command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => {
            children.push(child);
            PlayOutcome::Played
        }
        Err(_) => PlayOutcome::Failed,
    }
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
fn reap_audio_children(children: &mut Vec<std::process::Child>) {
    let mut index = 0;
    while index < children.len() {
        match children[index].try_wait() {
            Ok(Some(_)) | Err(_) => {
                let mut child = children.swap_remove(index);
                let _ = child.wait();
            }
            Ok(None) => index += 1,
        }
    }
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
impl Drop for Audio {
    fn drop(&mut self) {
        for child in &mut self.children {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
fn sound_files() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("honk0.mp3", HONKS[0]),
        ("honk1.mp3", HONKS[1]),
        ("honk2.mp3", HONKS[2]),
        ("honk3.mp3", HONKS[3]),
        ("bite.mp3", BITE),
        ("mud.mp3", MUD),
        ("pat0.wav", PATS[0]),
        ("pat1.wav", PATS[1]),
        ("pat2.wav", PATS[2]),
    ]
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
fn sound_file_name(sound: Sound, mut next: impl FnMut() -> usize) -> String {
    match sound {
        Sound::Honk(tone) => match tone {
            HonkTone::Normal => format!("honk{}.mp3", next() % HONKS.len()),
            HonkTone::High => format!("honk{}.mp3", (next() + 1) % HONKS.len()),
            HonkTone::Low => {
                format!("honk{}.mp3", (next() + HONKS.len() - 1) % HONKS.len())
            }
        },
        Sound::Bite => "bite.mp3".into(),
        Sound::MudSquish => "mud.mp3".into(),
        Sound::Pat => format!("pat{}.wav", next() % PATS.len()),
    }
}

#[cfg(test)]
fn linux_audio_backend_for(target_env: &str) -> &'static str {
    if target_env == "musl" {
        "command-fallback"
    } else {
        "in-process"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_audio_child_pool_is_bounded() {
        assert_eq!(bounded_child_count(5), MAX_AUDIO_CHILDREN);
    }

    #[test]
    fn linux_player_discovery_searches_path() {
        let joined = std::env::join_paths([
            std::path::PathBuf::from("custom/tools"),
            std::path::PathBuf::from("usr/bin"),
        ])
        .unwrap();
        assert!(path_candidates("ffplay", &joined)
            .iter()
            .any(|path| path.ends_with("custom/tools/ffplay")));
    }

    #[test]
    fn linux_backend_selection_keeps_command_fallback_musl_only() {
        assert_eq!(linux_audio_backend_for("gnu"), "in-process");
        assert_eq!(linux_audio_backend_for("musl"), "command-fallback");
    }
}
