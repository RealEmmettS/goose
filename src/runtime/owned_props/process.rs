use super::*;
use crate::assets::AssetCatalog;
use base64::Engine;
use honk_engine::collect_window::{collect_note_size, fit_collect_image};
use honk_engine::{CollectWindowCommand, CollectWindowPayload};
use std::fs::File;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command as Process, Stdio};
use std::time::{Duration, Instant};

pub(crate) struct Controller {
    child: Child,
    input: Option<ChildStdin>,
    output: ChildStdout,
    errors: ChildStderr,
    verified: Vec<File>,
    registry: Registry,
    outbox: Outbox,
    incoming: Vec<u8>,
    diagnostics: VecDeque<u8>,
    started: Instant,
    last_progress: Instant,
}

impl Controller {
    pub(crate) fn start(positioning: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let current = std::env::current_exe()?;
        let settings = current.with_file_name("honk300-settings");
        let verified = crate::install::verify_settings_companion(&current, &settings)?;
        let program = crate::install::companions::verified_settings_program(&verified)?;
        let mut child = Process::new(program)
            .arg0(&settings)
            .arg("--owned-props")
            .env("GDK_BACKEND", if positioning { "x11" } else { "wayland" })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        // Piped standard handles are guaranteed by Command. Once constructed,
        // every setup failure drops this owner and kills/reaps its exact child.
        let input = child.stdin.take().expect("piped input");
        let output = child.stdout.take().expect("piped output");
        let errors = child.stderr.take().expect("piped errors");
        let this = Self {
            child,
            input: Some(input),
            output,
            errors,
            verified,
            registry: Registry::new(positioning),
            outbox: Outbox::default(),
            incoming: Vec::with_capacity(MAX_EVENT_BYTES * 2),
            diagnostics: VecDeque::with_capacity(4096),
            started: Instant::now(),
            last_progress: Instant::now(),
        };
        nonblocking(this.input.as_ref().expect("retained input"))?;
        nonblocking(&this.output)?;
        nonblocking(&this.errors)?;
        Ok(this)
    }

    pub(crate) fn ready(&self) -> bool {
        self.registry.ready
    }
    pub(crate) fn has_capacity(&self) -> bool {
        self.registry.has_capacity()
    }
    pub(crate) fn snapshot(&mut self) -> Option<CollectWindowSnapshot> {
        self.registry.snapshot()
    }

    pub(crate) fn poll(&mut self) -> io::Result<()> {
        self.drain_diagnostics()?;
        if let Some(status) = self.child.try_wait()? {
            return Err(io::Error::other(format!(
                "native prop host exited ({status}): {}",
                self.diagnostic_text()
            )));
        }
        self.read_events()?;
        if !self.ready() && self.started.elapsed() > Duration::from_secs(10) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native prop host did not become ready",
            ));
        }
        if self.ready() {
            self.verified.clear();
        }
        let before = self.outbox.remaining();
        self.outbox
            .flush(self.input.as_mut().expect("retained input"))?;
        if before == 0 || self.outbox.remaining() < before {
            self.last_progress = Instant::now();
        } else if self.last_progress.elapsed() > Duration::from_secs(5) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native prop host stopped reading commands",
            ));
        }
        Ok(())
    }

    fn read_events(&mut self) -> io::Result<()> {
        let mut events = 0;
        loop {
            while let Some(end) = self.incoming.iter().position(|byte| *byte == b'\n') {
                self.registry.accept(&self.incoming[..end])?;
                self.incoming.drain(..=end);
                events += 1;
                if events == 64 {
                    return Ok(());
                }
            }
            if self.incoming.len() >= MAX_EVENT_BYTES {
                return Err(invalid("unterminated or oversized owned prop response"));
            }
            let mut bytes = [0u8; MAX_EVENT_BYTES];
            match self.output.read(&mut bytes) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "native prop host closed its output",
                    ))
                }
                Ok(count) => self.incoming.extend_from_slice(&bytes[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
    }

    fn drain_diagnostics(&mut self) -> io::Result<()> {
        let mut bytes = [0u8; 1024];
        for _ in 0..4 {
            match self.errors.read(&mut bytes) {
                Ok(0) => break,
                Ok(count) => {
                    for byte in &bytes[..count] {
                        if self.diagnostics.len() == 4096 {
                            self.diagnostics.pop_front();
                        }
                        self.diagnostics.push_back(*byte);
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn diagnostic_text(&self) -> String {
        // Keep diagnostics printable; a malformed companion cannot inject
        // terminal control sequences into the runtime's retained error log.
        String::from_utf8_lossy(&self.diagnostics.iter().copied().collect::<Vec<_>>())
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .collect()
    }

    pub(crate) fn apply(
        &mut self,
        command: CollectWindowCommand,
        assets: &AssetCatalog,
        bounds: Rect,
    ) -> io::Result<()> {
        if !self.ready() {
            return Err(invalid("owned prop command before readiness"));
        }
        let wire = match command {
            CollectWindowCommand::Spawn { request, payload } => {
                return self.spawn(request, payload, assets, bounds)
            }
            CollectWindowCommand::Move { id, top_left } => {
                if !self.registry.expected_positioning {
                    return Err(invalid("global prop movement is unavailable"));
                }
                if !self.registry.entries.contains_key(&id.0) {
                    return Ok(());
                }
                let (x, y) = coordinates(top_left)?;
                Command::Move { id: id.0, x, y }
            }
            CollectWindowCommand::SetPassthrough { id, passthrough } => {
                if !self.registry.entries.contains_key(&id.0) {
                    return Ok(());
                }
                Command::Passthrough {
                    id: id.0,
                    passthrough,
                }
            }
            CollectWindowCommand::Focus { id } => {
                if !self.registry.entries.contains_key(&id.0) {
                    return Ok(());
                }
                Command::Focus { id: id.0 }
            }
            CollectWindowCommand::TypeNote { id, note_index } => {
                if !self.registry.entries.contains_key(&id.0) {
                    return Ok(());
                }
                let text = assets
                    .note_text(note_index)
                    .ok_or_else(|| invalid("missing owned note asset"))?;
                if text.len() > 16 * 1024 || text.contains('\0') {
                    return Err(invalid("invalid owned note text"));
                }
                Command::Text { id: id.0, text }
            }
            CollectWindowCommand::Close { id } => {
                if !self.registry.entries.contains_key(&id.0) {
                    return Ok(());
                }
                Command::Close { id: id.0 }
            }
        };
        self.outbox.enqueue(wire)
    }

    fn spawn(
        &mut self,
        request: CollectWindowRequestId,
        payload: CollectWindowPayload,
        assets: &AssetCatalog,
        bounds: Rect,
    ) -> io::Result<()> {
        if self.registry.contains_request(request) || !self.has_capacity() {
            return Ok(());
        }
        match payload {
            CollectWindowPayload::Note { .. } => {
                let size = collect_note_size(bounds);
                let (width, height) = (size.x as u32, size.y as u32);
                let (x, y) = spawn_position(bounds, width, height)?;
                if let Some(id) =
                    self.registry
                        .reserve(request, CollectWindowKind::Note, width, height)?
                {
                    self.outbox.enqueue(Command::Note {
                        id,
                        x,
                        y,
                        width,
                        height,
                        title: "A note from your goose",
                    })?;
                }
            }
            CollectWindowPayload::Meme { index } => {
                let meme = assets
                    .meme(index)
                    .ok_or_else(|| invalid("missing owned picture asset"))?;
                let image = fit_collect_image(&meme.pixmap, bounds)
                    .ok_or_else(|| invalid("cannot fit owned picture"))?;
                let max_width = (bounds.width() * 0.48).floor().clamp(1.0, 900.0) as u32;
                let max_height = (bounds.height() * 0.48).floor().clamp(1.0, 700.0) as u32;
                // The entire image is fitted again inside the native content
                // area; the header and any blank margin share the same ceiling.
                let width = image.width().max(180).min(max_width);
                let height = image.height().saturating_add(40).max(100).min(max_height);
                let (x, y) = spawn_position(bounds, width, height)?;
                let pixels = base64::engine::general_purpose::STANDARD.encode(image.data());
                let mut title_end = meme.title.len().min(256);
                while !meme.title.is_char_boundary(title_end) {
                    title_end -= 1;
                }
                let title = &meme.title[..title_end];
                if let Some(id) =
                    self.registry
                        .reserve(request, CollectWindowKind::Meme, width, height)?
                {
                    self.outbox.enqueue(Command::Image {
                        id,
                        x,
                        y,
                        width,
                        height,
                        pixel_width: image.width(),
                        pixel_height: image.height(),
                        title,
                        pixels: &pixels,
                    })?;
                }
            }
        }
        Ok(())
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        self.input.take();
        // The unreaped Child handle remains authoritative. Never discover or
        // signal a process by name/PID after relinquishing that ownership.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn_position(bounds: Rect, width: u32, height: u32) -> io::Result<(i32, i32)> {
    coordinates(Vec2::new(
        (bounds.min.x + 40.0)
            .min(bounds.max.x - width as f32)
            .max(bounds.min.x),
        (bounds.min.y + 80.0)
            .min(bounds.max.y - height as f32)
            .max(bounds.min.y),
    ))
}

fn nonblocking(handle: &impl AsRawFd) -> io::Result<()> {
    // SAFETY: the borrowed live pipe retains its descriptor for both fcntl
    // calls. F_GETFL/F_SETFL do not transfer ownership or access memory.
    let flags = unsafe { libc::fcntl(handle.as_raw_fd(), libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(handle.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
