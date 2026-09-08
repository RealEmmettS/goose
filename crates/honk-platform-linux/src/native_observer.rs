//! Retained bounded workers for independently authenticated native observations.
use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub trait Source: Send + Sized + 'static {
    type Frame: Clone + Send + 'static;
    const NAME: &'static str;
    const MAX_AGE: Duration;
    fn connect() -> io::Result<Self>;
    fn snapshot(&mut self) -> io::Result<Self::Frame>;
    fn retryable(_error: &io::Error) -> bool {
        false
    }
}

struct State<F> {
    frame: Option<(Instant, F)>,
    failed: bool,
}

/// No native I/O runs in the presentation loop. A source may retry a late
/// read only while retaining its original authenticated owner; every other
/// error terminates the worker. Old frames are withdrawn before any retry.
pub struct Observer<S: Source> {
    state: Arc<Mutex<State<S::Frame>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl<S: Source> Observer<S> {
    pub fn start() -> io::Result<Self> {
        Self::spawn(S::connect)
    }
    pub(crate) fn spawn(
        connect: impl FnOnce() -> io::Result<S> + Send + 'static,
    ) -> io::Result<Self> {
        let state = Arc::new(Mutex::new(State {
            frame: None,
            failed: false,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let shared = Arc::clone(&state);
        let stopping = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name(format!("{}-observer", S::NAME))
            .spawn(move || {
                let result = (|| -> io::Result<()> {
                    let mut connection = connect()?;
                    while !stopping.load(Ordering::Acquire) {
                        let started = Instant::now();
                        let observed = connection.snapshot();
                        if stopping.load(Ordering::Acquire) {
                            break;
                        }
                        let mut state = shared.lock().map_err(|_| {
                            io::Error::other("Native observation state unavailable")
                        })?;
                        match observed {
                            Ok(frame) => {
                                // Include transport latency in age. Completion never
                                // renews information that was already too old.
                                state.frame = Some((started, frame));
                                state.failed = false;
                            }
                            Err(error) => {
                                state.frame = None;
                                state.failed = true;
                                if !S::retryable(&error) {
                                    return Err(error);
                                }
                            }
                        }
                        drop(state);
                        thread::park_timeout(Duration::from_millis(50));
                    }
                    Ok(())
                })();
                if let Err(error) = &result {
                    eprintln!(
                        "honk300: optional {} observations ended ({error}); repeat setup to retry",
                        S::NAME
                    );
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

    pub fn snapshot(&self) -> Option<S::Frame> {
        // Publication only assigns an already decoded bounded value. Brief
        // lock contention is not a capability failure.
        self.state
            .lock()
            .ok()?
            .frame
            .as_ref()
            .filter(|(at, _)| at.elapsed() < S::MAX_AGE)
            .map(|(_, frame)| frame.clone())
    }
    pub fn failed(&self) -> bool {
        self.state.lock().map_or(true, |state| state.failed)
    }
    /// A recovering or connecting worker still owns its authenticated source.
    /// Snapshot freshness determines capability, not permission to replace it.
    pub fn running(&self) -> bool {
        self.worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
    }
}

impl<S: Source> Drop for Observer<S> {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            // Every source request is bounded; retain the exact worker until
            // it exits, including while permission is being removed.
            let joined = worker.join();
            if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
                && std::env::var("HONK300_TRACE_OBSERVER").as_deref() == Ok("1")
            {
                eprintln!(
                    "honk300 observer trace: name={} joined={}",
                    S::NAME,
                    joined.is_ok()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    struct Fixture;
    impl Source for Fixture {
        type Frame = ();
        const NAME: &'static str = "fixture";
        const MAX_AGE: Duration = Duration::from_millis(250);
        fn connect() -> io::Result<Self> {
            unreachable!()
        }
        fn snapshot(&mut self) -> io::Result<()> {
            unreachable!()
        }
    }
    #[test]
    fn publication_contention_preserves_the_healthy_snapshot() {
        let observer = Arc::new(Observer::<Fixture> {
            state: Arc::new(Mutex::new(State {
                frame: Some((Instant::now(), ())),
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

    #[test]
    fn timeout_withdraws_old_data_and_recovers_only_the_retained_source() {
        struct Retained {
            answers: mpsc::Receiver<io::Result<u8>>,
            requests: mpsc::Sender<()>,
        }
        impl Source for Retained {
            type Frame = u8;
            const NAME: &'static str = "retained-fixture";
            const MAX_AGE: Duration = Duration::from_millis(250);
            fn connect() -> io::Result<Self> {
                unreachable!()
            }
            fn snapshot(&mut self) -> io::Result<u8> {
                let _ = self.requests.send(());
                self.answers
                    .recv_timeout(Self::MAX_AGE)
                    .unwrap_or_else(|_| Err(io::ErrorKind::TimedOut.into()))
            }
            fn retryable(error: &io::Error) -> bool {
                error.kind() == io::ErrorKind::TimedOut
            }
        }
        fn eventually(check: impl Fn() -> bool) {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !check() {
                assert!(Instant::now() < deadline, "worker state did not arrive");
                thread::sleep(Duration::from_millis(1));
            }
        }
        let (answers, replies) = mpsc::channel();
        let (requests, calls) = mpsc::channel();
        let connects = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count = Arc::clone(&connects);
        let observer = Observer::<Retained>::spawn(move || {
            count.fetch_add(1, Ordering::Relaxed);
            Ok(Retained {
                answers: replies,
                requests,
            })
        })
        .unwrap();
        calls.recv_timeout(Duration::from_secs(2)).unwrap();
        answers.send(Ok(7)).unwrap();
        eventually(|| observer.snapshot() == Some(7));
        calls.recv_timeout(Duration::from_secs(2)).unwrap();
        answers.send(Err(io::ErrorKind::TimedOut.into())).unwrap();
        eventually(|| observer.failed() && observer.snapshot().is_none());
        calls.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(observer.snapshot().is_none());
        answers.send(Ok(8)).unwrap();
        eventually(|| !observer.failed() && observer.snapshot() == Some(8));
        calls.recv_timeout(Duration::from_secs(2)).unwrap();
        answers
            .send(Err(io::ErrorKind::PermissionDenied.into()))
            .unwrap();
        eventually(|| observer.failed() && observer.snapshot().is_none());
        assert!(matches!(
            calls.recv_timeout(Duration::from_secs(2)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
        assert_eq!(connects.load(Ordering::Relaxed), 1);
        drop(observer);
    }
}
