use super::{directory, installed, Error, SCRIPT};
use honk_control::{CapabilityStatus, WaylandStatus};
use honk_engine::{ForeignWindowId, ForeignWindowSnapshot, Rect, Vec2};
use honk_platform_linux::kwin::{Bridge, Frame};
use std::{
    collections::HashMap,
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
}

impl KwinRuntime {
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
        self.disable();
        self.failed = true;
        let consent = installed::read(&directory()?)?
            .ok_or("Use KDE setup before enabling its integration")?;
        if !consent.current() {
            return Err("Repeat KDE setup for this installed update".into());
        }
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
            self.identities.clear();
            if self.observed && !self.warned_frame_loss {
                self.warned_frame_loss = true;
                eprintln!("honk300: KDE observations expired; optional actions are cancelled until fresh observations return");
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
