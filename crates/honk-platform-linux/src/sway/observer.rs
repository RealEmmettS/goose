use super::{Connection, Frame, MAX_AGE};
use crate::native_observer::Source;
use std::{io, time::Duration};

pub type Observer = crate::native_observer::Observer<Connection>;

impl Source for Connection {
    type Frame = Frame;
    const NAME: &'static str = "sway";
    const MAX_AGE: Duration = MAX_AGE;
    fn connect() -> io::Result<Self> {
        Connection::connect()
    }
    fn snapshot(&mut self) -> io::Result<Frame> {
        Connection::snapshot(self)
    }
}
