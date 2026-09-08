use super::{Frame, State, Window, MAX_AGE};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use zbus::blocking::{connection, Connection, Proxy};
use zbus::message::Header;

const SERVICE: &str = "org.emmetts.Honk300.Wayland";
const PATH: &str = "/org/emmetts/Honk300/KWin";
const KWIN: &str = "org.kde.KWin";

struct Endpoint {
    state: Arc<Mutex<State>>,
}

#[zbus::interface(name = "org.emmetts.Honk300.KWin1")]
impl Endpoint {
    async fn exchange(
        &self,
        raw: &str,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> zbus::fdo::Result<String> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::AccessDenied("Missing sender".into()))?;
        {
            let state = self
                .state
                .lock()
                .map_err(|_| zbus::fdo::Error::Failed("KWin state unavailable".into()))?;
            if sender.as_str() != state.owner {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Unexpected compositor sender".into(),
                ));
            }
        }
        let bus = zbus::fdo::DBusProxy::new(connection).await?;
        let owner = bus
            .get_name_owner(KWIN.try_into().expect("constant bus name"))
            .await;
        let mut state = self
            .state
            .lock()
            .map_err(|_| zbus::fdo::Error::Failed("KWin state unavailable".into()))?;
        let owner = match owner {
            Ok(owner) => owner,
            Err(error) => {
                state.stop();
                return Err(error);
            }
        };
        state
            .exchange(sender.as_str(), owner.as_str(), raw, Instant::now())
            .map_err(|error| zbus::fdo::Error::Failed(error.into()))
    }
}

/// Session-bus ownership never opts the user into an integration. Call only after
/// explicit setup is validated by the runtime; dropping the bridge revokes it.
pub struct Bridge {
    connection: Option<Connection>,
    state: Arc<Mutex<State>>,
}

impl Bridge {
    pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let discovery = connection::Builder::session()?
            .method_timeout(MAX_AGE)
            .max_queued(8)
            .build()?;
        let bus = Proxy::new(
            &discovery,
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
        )?;
        let owner: String = bus.call("GetNameOwner", &(KWIN,))?;
        let uid: u32 = bus.call("GetConnectionUnixUser", &(&owner,))?;
        // The session bus identifies its peer; an environment desktop hint does not.
        if uid != unsafe { libc::geteuid() } {
            return Err("KWin belongs to another user".into());
        }
        let state = Arc::new(Mutex::new(State::new(owner)));
        let connection = connection::Builder::session()?
            .method_timeout(MAX_AGE)
            .max_queued(8)
            .allow_name_replacements(false)
            .replace_existing_names(false)
            .serve_at(
                PATH,
                Endpoint {
                    state: state.clone(),
                },
            )?
            .name(SERVICE)?
            .build()?;
        Ok(Self {
            connection: Some(connection),
            state,
        })
    }

    pub fn snapshot(&self) -> Option<Frame> {
        self.state.lock().ok()?.snapshot(Instant::now()).cloned()
    }

    pub fn queue_move(&self, window: &Window, to: [f64; 2]) -> Result<(), &'static str> {
        self.state
            .lock()
            .map_err(|_| "KWin state unavailable")?
            .queue_move(window, to, Instant::now())
    }

    pub fn stop(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.stop();
        }
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        self.stop();
        if let Some(connection) = self.connection.take() {
            let _ = connection.close();
        }
    }
}
