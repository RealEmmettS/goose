//! One retained worker observes AX fullscreen without blocking AppKit or IPC.
use honk_control::CapabilityStatus;
use std::io;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(75);
const FRESHNESS: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Target {
    pid: i32,
    generation: u64,
}

type Observation = Result<bool, CapabilityStatus>;

#[derive(Default)]
struct State {
    target: Option<Target>,
    sample: Option<(Instant, Observation)>,
    stop: bool,
}

struct Worker {
    shared: Arc<(Mutex<State>, Condvar)>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(mut query: impl FnMut(Target) -> Observation + Send + 'static) -> io::Result<Self> {
        let shared = Arc::new((Mutex::new(State::default()), Condvar::new()));
        let worker_shared = shared.clone();
        let thread = thread::Builder::new()
            .name("honk-fullscreen".into())
            .spawn(move || {
                let (lock, wake) = &*worker_shared;
                let mut state = lock.lock().unwrap_or_else(|error| error.into_inner());
                loop {
                    if state.stop {
                        break;
                    }
                    let Some(target) = state.target else {
                        state = wake.wait(state).unwrap_or_else(|error| error.into_inner());
                        continue;
                    };
                    drop(state);
                    let started = Instant::now();
                    let observation = query(target);
                    state = lock.lock().unwrap_or_else(|error| error.into_inner());
                    if state.target == Some(target) {
                        // Age starts before the native query, not after a slow response.
                        state.sample = Some((started, observation));
                    }
                    if state.stop {
                        break;
                    }
                    if state.target != Some(target) {
                        continue;
                    }
                    state = wake
                        .wait_timeout(state, SAMPLE_INTERVAL)
                        .unwrap_or_else(|error| error.into_inner())
                        .0;
                }
            })?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    fn poll(&self, target: Option<Target>) -> Observation {
        if self.thread.as_ref().is_none_or(JoinHandle::is_finished) {
            return Err(CapabilityStatus::Failed);
        }
        let (lock, wake) = &*self.shared;
        let mut state = lock.lock().unwrap_or_else(|error| error.into_inner());
        if state.target != target {
            state.target = target;
            state.sample = None;
            wake.notify_one();
        }
        state
            .sample
            .filter(|(started, _)| started.elapsed() <= FRESHNESS)
            .map(|(_, result)| result)
            .unwrap_or(Err(CapabilityStatus::Unprobed))
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let (lock, wake) = &*self.shared;
        lock.lock().unwrap_or_else(|error| error.into_inner()).stop = true;
        wake.notify_one();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(target_os = "macos")]
mod native {
    use super::*;
    use honk_control::PresenceStatus;
    use honk_engine::PresenceSnapshot;
    use objc2::runtime::NSObjectProtocol;
    use objc2::{rc::Retained, MainThreadMarker};
    use objc2_app_kit::{NSRunningApplication, NSWorkspace};
    use objc2_application_services::{AXError, AXIsProcessTrusted, AXUIElement};
    use objc2_core_foundation::{CFBoolean, CFRetained, CFString, CFType};
    use std::ptr::{self, NonNull};

    const QUERY_TIMEOUT: Duration = Duration::from_millis(100);

    pub struct PresenceObserver {
        worker: Worker,
        application: Option<Retained<NSRunningApplication>>,
        generation: u64,
        _main_thread: MainThreadMarker,
    }

    impl PresenceObserver {
        pub fn new() -> io::Result<Self> {
            let mtm = MainThreadMarker::new().ok_or_else(|| {
                io::Error::other("presence observation requires the AppKit main thread")
            })?;
            Ok(Self {
                worker: Worker::new(query)?,
                application: None,
                generation: 0,
                _main_thread: mtm,
            })
        }

        pub fn poll(&mut self) -> (PresenceSnapshot, PresenceStatus) {
            let observation = self.observe();
            let fullscreen = observation
                .map(|_| CapabilityStatus::Supported)
                .unwrap_or_else(|state| state);
            let snapshot = match observation {
                Ok(true) => PresenceSnapshot::fullscreen(),
                Ok(false) => PresenceSnapshot::available(),
                Err(_) => PresenceSnapshot::unsupported(),
            };
            (
                snapshot,
                PresenceStatus {
                    fullscreen,
                    dnd: CapabilityStatus::Unsupported,
                },
            )
        }

        fn observe(&mut self) -> Observation {
            if !AXIsProcessTrusted() {
                self.application = None;
                self.worker.poll(None).ok();
                return Err(CapabilityStatus::Denied);
            }
            // AppKit identity stays on the main thread; only a PID and generation
            // cross into the worker. A changed or terminated app withdraws old data.
            let current = NSWorkspace::sharedWorkspace()
                .frontmostApplication()
                .filter(|app| !app.isTerminated() && app.processIdentifier() > 0);
            let Some(current) = current else {
                self.application = None;
                self.worker.poll(None).ok();
                return Err(CapabilityStatus::Unprobed);
            };
            let unchanged = self
                .application
                .as_ref()
                .is_some_and(|previous| !previous.isTerminated() && previous.isEqual(&current));
            if !unchanged {
                self.generation = self.generation.wrapping_add(1);
            }
            let target = Target {
                pid: current.processIdentifier(),
                generation: self.generation,
            };
            self.application = Some(current);
            self.worker.poll(Some(target))
        }
    }

    fn error_state(error: AXError) -> CapabilityStatus {
        match error {
            AXError::APIDisabled => CapabilityStatus::Denied,
            AXError::AttributeUnsupported | AXError::NoValue => CapabilityStatus::Unsupported,
            _ => CapabilityStatus::Failed,
        }
    }

    fn bounded(element: &AXUIElement, deadline: Instant) -> Result<(), CapabilityStatus> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(CapabilityStatus::Failed)?;
        let error = unsafe { element.set_messaging_timeout(remaining.as_secs_f32()) };
        if error == AXError::Success {
            Ok(())
        } else {
            Err(error_state(error))
        }
    }

    fn attribute(
        element: &AXUIElement,
        name: &'static str,
        deadline: Instant,
    ) -> Result<CFRetained<CFType>, CapabilityStatus> {
        bounded(element, deadline)?;
        let mut raw: *const CFType = ptr::null();
        let slot = NonNull::from(&mut raw);
        let error = unsafe { element.copy_attribute_value(&CFString::from_static_str(name), slot) };
        if error != AXError::Success {
            return Err(error_state(error));
        }
        let raw = NonNull::new(raw.cast_mut()).ok_or(CapabilityStatus::Failed)?;
        Ok(unsafe { CFRetained::from_raw(raw) })
    }

    fn query(target: Target) -> Observation {
        if !AXIsProcessTrusted() {
            return Err(CapabilityStatus::Denied);
        }
        let deadline = Instant::now() + QUERY_TIMEOUT;
        let application = unsafe { AXUIElement::new_application(target.pid) };
        let focused = attribute(&application, "AXFocusedWindow", deadline)?;
        let window = focused
            .downcast_ref::<AXUIElement>()
            .ok_or(CapabilityStatus::Failed)?;
        bounded(window, deadline)?;
        let mut pid = 0;
        let error = unsafe { window.pid(NonNull::from(&mut pid)) };
        if error != AXError::Success {
            return Err(error_state(error));
        }
        if pid != target.pid {
            return Err(CapabilityStatus::Failed);
        }
        let value = attribute(window, "AXFullScreen", deadline)?;
        if Instant::now() > deadline || !AXIsProcessTrusted() {
            return Err(CapabilityStatus::Failed);
        }
        value
            .downcast_ref::<CFBoolean>()
            .map(CFBoolean::value)
            .ok_or(CapabilityStatus::Unsupported)
    }
}

#[cfg(target_os = "macos")]
pub use native::PresenceObserver;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn changed_target_and_revocation_withdraw_inflight_results_without_waiting() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker = Worker::new(move |target| {
            started_tx.send(target).unwrap();
            release_rx.recv().unwrap();
            Ok(true)
        })
        .unwrap();
        let old = Target {
            pid: 1,
            generation: 1,
        };
        let new = Target {
            pid: 1,
            generation: 2,
        };
        assert!(worker.poll(Some(old)).is_err());
        assert_eq!(
            started_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            old
        );
        assert!(worker.poll(Some(new)).is_err());
        assert!(worker.poll(None).is_err());
        release_tx.send(()).unwrap();
        assert!(worker.poll(None).is_err());
        drop(worker); // The in-flight worker must finish before ownership is released.
    }

    #[test]
    fn an_actual_slow_query_cannot_refresh_fullscreen_with_an_old_sample() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker = Worker::new(move |_| {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            Ok(true)
        })
        .unwrap();
        let target = Target {
            pid: 2,
            generation: 1,
        };
        worker.poll(Some(target)).ok();
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        thread::sleep(FRESHNESS + Duration::from_millis(10));
        release_tx.send(()).unwrap();
        // The next real query starts only after the first result was delivered.
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(worker.poll(Some(target)).is_err());
        worker.poll(None).ok();
        release_tx.send(()).unwrap();
        drop(worker);
    }
}
