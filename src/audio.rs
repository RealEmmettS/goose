//! Audio backend — plays the goose's sound requests via `rodio`.
//!
//! The engine emits platform-free [`Sound`] requests; this maps them to the bundled clips
//! and plays them fire-and-forget. The original honk/bite/mud/pat sounds are embedded for
//! personal-use self-distribution. Honest degradation: if there is no output device the
//! whole backend is a silent no-op, and individual decode/playback failures are ignored.

use honk_engine::{HonkTone, Sound};
#[cfg(windows)]
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
#[cfg(windows)]
use std::io::Cursor;

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
#[cfg(windows)]
pub struct Audio {
    // Held only to keep the device open; never touched directly.
    _stream: OutputStream,
    handle: OutputStreamHandle,
    counter: usize,
}

#[cfg(windows)]
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

    /// Play `sound` fire-and-forget (honks/pats rotate through their variants).
    pub fn play(&mut self, sound: Sound) {
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
        if let Ok(sink) = Sink::try_new(&self.handle) {
            if let Ok(decoder) = Decoder::new(Cursor::new(bytes)) {
                sink.append(decoder);
                sink.detach(); // play to completion in the background
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub struct Audio {
    dir: std::path::PathBuf,
    counter: usize,
}

#[cfg(target_os = "macos")]
impl Audio {
    /// Open the macOS command-line sound backend. Returns `None` if `afplay` is missing or the
    /// embedded clips cannot be staged to a private temp directory.
    pub fn new() -> Option<Self> {
        let afplay = std::path::Path::new("/usr/bin/afplay");
        if !afplay.exists() {
            return None;
        }
        let dir = std::env::temp_dir().join(format!("honk300-audio-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok()?;
        for (name, bytes) in sound_files() {
            std::fs::write(dir.join(name), bytes).ok()?;
        }
        Some(Self { dir, counter: 0 })
    }

    fn next(&mut self) -> usize {
        self.counter = self.counter.wrapping_add(1);
        self.counter
    }

    pub fn play(&mut self, sound: Sound) {
        let name = match sound {
            Sound::Honk(tone) => match tone {
                HonkTone::Normal => format!("honk{}.mp3", self.next() % HONKS.len()),
                HonkTone::High => format!("honk{}.mp3", (self.next() + 1) % HONKS.len()),
                HonkTone::Low => {
                    format!("honk{}.mp3", (self.next() + HONKS.len() - 1) % HONKS.len())
                }
            },
            Sound::Bite => "bite.mp3".into(),
            Sound::MudSquish => "mud.mp3".into(),
            Sound::Pat => format!("pat{}.wav", self.next() % PATS.len()),
        };
        let _ = std::process::Command::new("/usr/bin/afplay")
            .arg(self.dir.join(name))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

#[cfg(target_os = "macos")]
impl Drop for Audio {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(target_os = "macos")]
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
