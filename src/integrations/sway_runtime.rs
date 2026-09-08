use super::{directory, sway_consent, Error};
use honk_control::{CapabilityStatus, WaylandStatus};
use honk_platform_linux::sway::{Frame, Observer};
use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct SwayRuntime {
    observer: Option<Observer>,
    consent: Option<sway_consent::Consent>,
    last_check: Option<Instant>,
    failed: bool,
    observed: bool,
}

impl SwayRuntime {
    pub(crate) fn start() -> Self {
        let mut runtime = Self::default();
        match directory().and_then(|path| sway_consent::read(&path).map_err(Into::into)) {
            Ok(None) => {}
            Ok(Some(_)) => {
                if let Err(error) = runtime.enable() {
                    eprintln!("honk300: optional Sway observations unavailable ({error})");
                }
            }
            Err(_) => runtime.failed = true,
        }
        runtime
    }

    pub(crate) fn enable(&mut self) -> Result<(), Error> {
        let desired = (|| -> Result<_, Error> {
            let consent = sway_consent::read(&directory()?)?
                .ok_or("Use Sway setup before enabling observations")?;
            if !consent.current() {
                return Err("Repeat Sway setup for this observation update".into());
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
        if self.consent.as_ref() == Some(&consent)
            && self
                .observer
                .as_ref()
                .is_some_and(|worker| worker.snapshot().is_some())
        {
            return Ok(());
        }
        self.disable();
        self.failed = true;
        self.observer = Some(Observer::start()?);
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
                .and_then(|path| sway_consent::read(&path).ok())
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
}
