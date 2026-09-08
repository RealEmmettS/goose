use super::{Connection, Frame, MAX_AGE};
use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Default)]
struct State {
    frame: Option<(Instant, Frame)>,
    failed: bool,
}

/// One bounded observation worker. No compositor I/O runs in the presentation
/// loop, and a lost owner is never followed by an automatic reconnect.
pub struct Observer {
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Observer {
    pub fn start() -> io::Result<Self> {
        let state = Arc::new(Mutex::new(State::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let shared = Arc::clone(&state);
        let stopping = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("sway-observer".into())
            .spawn(move || {
                let result = (|| -> io::Result<()> {
                    let mut connection = Connection::connect()?;
                    while !stopping.load(Ordering::Acquire) {
                        let started = Instant::now();
                        let frame = connection.snapshot()?;
                        if stopping.load(Ordering::Acquire) {
                            break;
                        }
                        if let Ok(mut state) = shared.lock() {
                            // Age includes transport latency; a slow reply never
                            // becomes a newly fresh observation on completion.
                            state.frame = Some((started, frame));
                        } else {
                            return Err(io::Error::other("Sway observation state unavailable"));
                        }
                        thread::park_timeout(Duration::from_millis(50));
                    }
                    Ok(())
            })();
            if let Err(error) = &result {
                eprintln!("honk300: optional Sway observations ended ({error}); repeat Sway setup to retry");
            }
                if let Ok(mut state) = shared.lock() {
                    state.frame = None;
                    state.failed = result.is_err();
                }
            })?;
        Ok(Self {
            state,
            stop,
            worker: Some(worker),
        })
    }

    pub fn snapshot(&self) -> Option<Frame> {
        self.state
            // The producer only assigns an already decoded frame under this
            // lock. Brief publication contention is not a capability failure.
            .lock()
            .ok()?
            .frame
            .as_ref()
            .filter(|(at, _)| at.elapsed() < MAX_AGE)
            .map(|(_, frame)| frame.clone())
    }

    pub fn failed(&self) -> bool {
        self.state.lock().map_or(true, |state| state.failed)
    }
}

impl Drop for Observer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            // Every native connect/request has a deadline. The exact worker is
            // retained until it exits; no detached thread survives removal.
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn publication_contention_preserves_the_healthy_snapshot() {
        let observer = Arc::new(Observer {
            state: Arc::new(Mutex::new(State {
                frame: Some((Instant::now(), Frame { windows: vec![] })),
                failed: false,
            })),
            stop: Arc::new(AtomicBool::new(false)),
            worker: None,
        });
        let mut publishing = observer.state.lock().unwrap();
        let reader = Arc::clone(&observer);
        let (started, start) = mpsc::channel();
        let (result, received) = mpsc::channel();
        let thread = thread::spawn(move || {
            started.send(()).unwrap();
            result.send((reader.snapshot(), reader.failed())).unwrap();
        });
        start.recv().unwrap();
        assert!(matches!(
            received.recv_timeout(Duration::from_millis(25)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        publishing.frame.as_mut().unwrap().0 = Instant::now();
        drop(publishing);
        let (frame, failed) = received.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(frame.is_some());
        assert!(!failed);
        thread.join().unwrap();
    }
}
