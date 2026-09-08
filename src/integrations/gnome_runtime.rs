use super::{directory, gnome_consent, Error};
use honk_control::{CapabilityStatus, WaylandStatus};
use honk_platform_linux::gnome::{Frame, Observer};
use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct GnomeRuntime {
    observer: Option<Observer>,
    consent: Option<gnome_consent::Consent>,
    last_check: Option<Instant>,
    failed: bool,
    observed: bool,
    identities: std::collections::HashMap<(u32, u32), honk_engine::ForeignWindowId>,
    next_id: u64,
}

impl GnomeRuntime {
    pub(crate) fn start() -> Self {
        let mut runtime = Self::default();
        match directory().and_then(|path| gnome_consent::read(&path).map_err(Into::into)) {
            Ok(None) => {}
            Ok(Some(_)) => {
                if let Err(error) = runtime.enable() {
                    eprintln!("honk300: optional Gnome observations unavailable ({error})");
                }
            }
            Err(_) => runtime.failed = true,
        }
        runtime
    }

    pub(crate) fn enable(&mut self) -> Result<(), Error> {
        let desired = (|| -> Result<_, Error> {
            let consent = gnome_consent::read(&directory()?)?
                .ok_or("Use Gnome setup before enabling observations")?;
            if !consent.current(&std::env::current_exe()?) {
                return Err("Repeat Gnome setup for this observation update".into());
            }
            gnome_consent::files_match(&directory()?, &consent)?;
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
        if self.consent.as_ref() == Some(&consent)
            && self
                .observer
                .as_ref()
                .is_some_and(|worker| worker.running())
        {
            return Ok(());
        }
        self.disable();
        self.failed = true;
        self.observer = Some(Observer::start(
            consent.nonce.clone(),
            gnome_consent::build_identity(),
        )?);
        self.consent = Some(consent);
        self.last_check = Some(Instant::now());
        self.failed = false;
        Ok(())
    }

    pub(crate) fn disable(&mut self) {
        self.observer = None;
        self.consent = None;
        self.last_check = None;
        self.failed = false;
        self.observed = false;
        self.identities.clear();
    }

    pub(crate) fn poll(&mut self) -> Option<Frame> {
        self.observer.as_ref()?;
        if self
            .last_check
            .is_none_or(|at| at.elapsed() >= Duration::from_millis(100))
        {
            self.last_check = Some(Instant::now());
            let saved = directory()
                .ok()
                .and_then(|path| gnome_consent::read(&path).ok())
                .flatten();
            if saved != self.consent {
                self.disable();
                return None;
            }
        }
        let frame = self.observer.as_ref()?.snapshot();
        self.observed |= frame.is_some();
        frame
    }

    pub(crate) fn status(&mut self) -> WaylandStatus {
        let frame = self.poll();
        let state = if frame.is_some() {
            CapabilityStatus::Supported
        } else if self.failed
            || self.observed
            || self.observer.as_ref().is_some_and(Observer::failed)
        {
            CapabilityStatus::Failed
        } else if self.observer.is_some() {
            CapabilityStatus::Unprobed
        } else {
            CapabilityStatus::Unsupported
        };
        WaylandStatus {
            windows: state,
            fullscreen: state,
            ..WaylandStatus::default()
        }
    }

    pub(crate) fn dragged(&mut self, frame: &Frame) -> Option<honk_engine::ForeignWindowSnapshot> {
        use honk_engine::{ForeignWindowId, ForeignWindowSnapshot, Rect, Vec2};
        self.identities.retain(|(pid, id), _| {
            frame
                .windows
                .iter()
                .any(|window| window.id == *id && window.pid == Some(*pid))
        });
        let window = frame.dragged_window()?;
        let key = (window.pid?, window.id);
        let id = if let Some(id) = self.identities.get(&key) {
            *id
        } else {
            self.next_id = self.next_id.checked_add(1)?;
            let id = ForeignWindowId(self.next_id);
            self.identities.insert(key, id);
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
