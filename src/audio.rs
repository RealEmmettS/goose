//! Audio backend — plays the goose's sound requests via `rodio`.
//!
//! The engine emits platform-free [`Sound`] requests; this maps them to the bundled clips
//! and plays them fire-and-forget. The original honk/bite/mud/pat sounds are embedded for
//! personal-use self-distribution. Honest degradation: if there is no output device the
//! whole backend is a silent no-op, and individual decode/playback failures are ignored.

use honk_engine::Sound;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
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
pub struct Audio {
    // Held only to keep the device open; never touched directly.
    _stream: OutputStream,
    handle: OutputStreamHandle,
    counter: usize,
}

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
            Sound::Honk => HONKS[self.next() % HONKS.len()],
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
