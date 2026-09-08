use super::{Connection, Frame, MAX_AGE};
use crate::native_observer::Source;
use std::{io, time::Duration};

pub type Observer = crate::native_observer::Observer<Connection>;

impl Source for Connection {
    type Frame = Frame;
    const NAME: &'static str = "hyprland";
    const MAX_AGE: Duration = MAX_AGE;
    fn connect() -> io::Result<Self> {
        Connection::connect()
    }
    fn snapshot(&mut self) -> io::Result<Frame> {
        Connection::snapshot(self)
    }
    fn retryable(error: &io::Error) -> bool {
        // The compositor can stall its whole event loop during a native
        // fullscreen configure. Discard that observation; a later read must
        // revalidate the same retained socket/process/executable identity.
        // Disconnect, replacement, malformed replies and permission loss never
        // take this path. No new peer or session is followed automatically.
        error.kind() == io::ErrorKind::TimedOut
    }
}
