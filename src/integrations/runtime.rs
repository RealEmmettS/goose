use super::{directory, installed, Error, SCRIPT};
use honk_control::{CapabilityStatus, WaylandStatus};
use honk_engine::{ForeignWindowId, ForeignWindowSnapshot, Rect, Vec2};
use honk_platform_linux::kwin::{Bridge, Frame};
use honk_platform_linux::portal::Session;
use std::{
    collections::HashMap,
    io,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(crate) struct KwinRuntime {
    bridge: Option<Bridge>,
    consent: Option<installed::Consent>,
    identities: HashMap<String, ForeignWindowId>,
    next_id: u64,
    last_consent_check: Option<Instant>,
    failed: bool,
    observed: bool,
    warned_frame_loss: bool,
    warned_move_rejection: bool,
    pointer: Option<Session>,
    pointer_failure: Option<CapabilityStatus>,
}

impl KwinRuntime {
    pub(crate) fn request_pointer(&mut self) -> io::Result<()> {
        let frame = self.poll();
        if cfg!(target_env = "musl") || frame.is_none_or(|frame| !frame.stacking_order) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Pointer permission requires a live KDE 6 companion and the GNU Linux build",
            ));
        }
        if self.pointer.is_some() {
            return Ok(());
        }
        match Session::request() {
            Ok(session) => {
                self.pointer = Some(session);
                self.pointer_failure = None;
                Ok(())
            }
            Err(error) => {
                self.pointer_failure = Some(CapabilityStatus::Failed);
                Err(error)
            }
        }
    }

    pub(crate) fn cancel_pointer(&mut self) {
        self.pointer = None;
        self.pointer_failure = None;
    }

    pub(crate) fn pointer_status(&self) -> CapabilityStatus {
        if cfg!(target_env = "musl")
            || self
                .bridge
                .as_ref()
                .and_then(Bridge::snapshot)
                .is_none_or(|frame| !frame.stacking_order)
        {
            CapabilityStatus::Unsupported
        } else if let Some(session) = &self.pointer {
            if session.ready() {
                CapabilityStatus::Supported
            } else {
                CapabilityStatus::Unprobed
            }
        } else {
            self.pointer_failure.unwrap_or(CapabilityStatus::Denied)
        }
    }

    pub(crate) fn warp_pointer(&mut self, target: Vec2) -> io::Result<()> {
        let bridge = self.bridge.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "The KDE companion is disconnected",
            )
        })?;
        let session = self.pointer.as_mut().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "No pointer permission is active",
            )
        })?;
        session.warp(bridge, [target.x as f64, target.y as f64])
    }

    pub(crate) fn move_owned_prop(&mut self, pid: u32, id: u64, size: [f64; 2], target: Vec2) {
        let Some(bridge) = self.bridge.as_ref() else {
            return;
        };
        let Some(frame) = bridge.snapshot() else {
            return;
        };
        let Some(window) = frame.owned_prop(pid, id, size) else {
            return;
        };
        let [left, top, area_width, area_height] = window.area;
        let [x, y, width, height] = window.geometry;
        if width > area_width
            || height > area_height
            || !target.x.is_finite()
            || !target.y.is_finite()
        {
            return;
        }
        let dx = (target.x as f64).clamp(left, left + area_width - width) - x;
        let dy = (target.y as f64).clamp(top, top + area_height - height) - y;
        let distance = dx.hypot(dy);
        let scale = if distance > 23.0 {
            23.0 / distance
        } else {
            1.0
        };
        // The same live identity/geometry and script-side bounds checks apply.
        // A rejected or superseded move is never retried with stale authority.
        if let Err(error) = bridge.queue_move(window, [x + dx * scale, y + dy * scale]) {
            if !self.warned_move_rejection {
                self.warned_move_rejection = true;
                eprintln!("honk300: KDE owned-window movement was refused ({error})");
            }
        }
    }

    pub(crate) fn start() -> Self {
        let mut runtime = Self::default();
        match directory().and_then(|path| installed::read(&path).map_err(Into::into)) {
            Ok(None) => {}
            Ok(Some(_)) => {
                if let Err(error) = runtime.enable() {
                    eprintln!("honk300: optional KDE integration unavailable ({error})");
                }
            }
            Err(error) => {
                runtime.failed = true;
                eprintln!("honk300: KDE setup cannot be read ({error})");
            }
        }
        runtime
    }

    pub(crate) fn enable(&mut self) -> Result<(), Error> {
        let desired = (|| -> Result<installed::Consent, Error> {
            let consent = installed::read(&directory()?)?
                .ok_or("Use KDE setup before enabling its integration")?;
            if !consent.current() {
                return Err("Repeat KDE setup for this installed update".into());
            }
            Ok(consent)
        })();
        let consent = match desired {
            Ok(consent) => consent,
            Err(error) => {
                self.disable();
                self.failed = true;
                return Err(error);
            }
        };
        // Repeating setup for the same healthy companion is idempotent. Keep
        // its live portal session, in-flight consent and target identities.
        // A changed grant or expired bridge still takes the full revoke path.
        if self.consent.as_ref() == Some(&consent)
            && self.bridge.as_ref().and_then(Bridge::snapshot).is_some()
        {
            self.last_consent_check = Some(Instant::now());
            return Ok(());
        }
        self.disable();
        self.failed = true;
        let mut bridge = Bridge::connect()?;
        bridge.retire_owned_script(&consent.name())?;
        bridge.load_script(&consent.name(), SCRIPT)?;
        self.bridge = Some(bridge);
        self.consent = Some(consent);
        self.last_consent_check = Some(Instant::now());
        self.failed = false;
        Ok(())
    }

    pub(crate) fn disable(&mut self) {
        self.cancel_pointer();
        if let Some(bridge) = self.bridge.take() {
            bridge.stop();
        }
        self.consent = None;
        self.identities.clear();
        self.failed = false;
        self.observed = false;
        self.last_consent_check = None;
        self.warned_frame_loss = false;
        self.warned_move_rejection = false;
    }

    pub(crate) fn poll(&mut self) -> Option<Frame> {
        self.bridge.as_ref()?;
        if self
            .last_consent_check
            .is_none_or(|time| time.elapsed() >= Duration::from_millis(100))
        {
            self.last_consent_check = Some(Instant::now());
            let valid = directory()
                .ok()
                .and_then(|path| installed::read(&path).ok())
                .flatten();
            if valid != self.consent {
                self.disable();
                return None;
            }
        }
        let frame = self.bridge.as_ref()?.snapshot();
        self.observed |= frame.is_some();
        if frame.is_none() {
            self.cancel_pointer();
            self.identities.clear();
            if self.observed && !self.warned_frame_loss {
                self.warned_frame_loss = true;
                eprintln!("honk300: KDE observations expired; optional actions are cancelled until fresh observations return");
            }
        }
        if let Some(session) = &mut self.pointer {
            if let Err(error) = session.poll() {
                self.pointer = None;
                self.pointer_failure = Some(if error.kind() == io::ErrorKind::PermissionDenied {
                    CapabilityStatus::Denied
                } else {
                    CapabilityStatus::Failed
                });
                eprintln!("honk300: pointer permission ended ({error})");
            }
        }
        frame
    }

    pub(crate) fn status(&self) -> WaylandStatus {
        let observed = self.bridge.as_ref().and_then(Bridge::snapshot).is_some();
        let state = if observed {
            CapabilityStatus::Supported
        } else if self.failed || self.observed {
            CapabilityStatus::Failed
        } else if self.bridge.is_some() {
            CapabilityStatus::Unprobed
        } else {
            CapabilityStatus::Unsupported
        };
        WaylandStatus {
            windows: state,
            movement: state,
            pointer_observation: state,
            pointer_control: self.pointer_status(),
            fullscreen: state,
            ..WaylandStatus::default()
        }
    }

    pub(crate) fn dragged(&mut self, frame: &Frame) -> Option<ForeignWindowSnapshot> {
        self.identities
            .retain(|identity, _| frame.windows.iter().any(|window| &window.id == identity));
        let window = frame.dragged_window()?;
        let id = if let Some(id) = self.identities.get(&window.id) {
            *id
        } else {
            self.next_id = self.next_id.checked_add(1)?;
            let id = ForeignWindowId(self.next_id);
            self.identities.insert(window.id.clone(), id);
            id
        };
        let [x, y, width, height] = window.geometry;
        Some(ForeignWindowSnapshot::top_center(
            id,
            Rect::new(
                Vec2::new(x as f32, y as f32),
                Vec2::new((x + width) as f32, (y + height) as f32),
            ),
        ))
    }
}
