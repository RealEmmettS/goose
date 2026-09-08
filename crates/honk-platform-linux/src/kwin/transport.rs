use super::{Frame, State, Window, MAX_AGE};
use std::fs::File;
use std::io::Write;
use std::os::fd::{AsRawFd, FromRawFd};
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
    script: Option<(String, File)>,
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
            script: None,
        })
    }

    /// Activate the explicitly installed companion from immutable bytes. KWin reads
    /// a sealed memfd owned by this runtime, never a path another writer can replace
    /// between identity validation and its asynchronous script load.
    pub fn load_script(
        &mut self,
        name: &str,
        bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.script.is_some()
            || !name.starts_with("honk300-")
            || name.len() > 96
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || bytes.is_empty()
            || bytes.len() > 65_536
        {
            return Err("Invalid or already loaded Honk300 companion".into());
        }
        let owner = self
            .state
            .lock()
            .map_err(|_| "KWin state unavailable")?
            .owner
            .clone();
        let connection = self.connection.as_ref().ok_or("KWin connection closed")?;
        let scripting = Proxy::new(
            connection,
            owner.as_str(),
            "/Scripting",
            "org.kde.kwin.Scripting",
        )?;
        let loaded: bool = scripting.call("isScriptLoaded", &(name,))?;
        if loaded {
            return Err(
                "This companion is already registered in KWin; remove and set it up again".into(),
            );
        }
        let mut file = sealed_script(bytes)?;
        file.flush()?;
        let path = format!("/proc/{}/fd/{}", std::process::id(), file.as_raw_fd());
        let id: i32 = scripting.call("loadScript", &(&path, name))?;
        if id < 0 {
            return Err("KWin rejected the companion registration".into());
        }
        // Retain ownership before any remaining fallible operation, so Drop will
        // unload exactly the script registered above if activation fails.
        self.script = Some((name.to_owned(), file));
        let modern_path = format!("/Scripting/Script{id}");
        let modern = Proxy::new(
            connection,
            owner.as_str(),
            modern_path.as_str(),
            "org.freedesktop.DBus.Introspectable",
        )?;
        let modern_result: Result<String, _> = modern.call("Introspect", &());
        drop(modern);
        let script_path = match modern_result {
            Ok(xml) if xml.contains("org.kde.kwin.Script") => modern_path,
            _ => format!("/{id}"),
        };
        let script = Proxy::new(
            connection,
            owner.as_str(),
            script_path.as_str(),
            "org.kde.kwin.Script",
        )?;
        script.call::<_, _, ()>("run", &())?;
        Ok(())
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
        if let (Some(connection), Some((name, _))) = (&self.connection, &self.script) {
            if let Ok(state) = self.state.lock() {
                if let Ok(proxy) = Proxy::new(
                    connection,
                    state.owner.as_str(),
                    "/Scripting",
                    "org.kde.kwin.Scripting",
                ) {
                    let _ = proxy.call::<_, _, bool>("unloadScript", &(name,));
                }
            }
        }
        if let Some(connection) = self.connection.take() {
            let _ = connection.close();
        }
    }
}

fn sealed_script(bytes: &[u8]) -> std::io::Result<File> {
    // SAFETY: static NUL-terminated name and supported Linux flags; ownership of
    // a successful descriptor transfers immediately to File.
    let descriptor = unsafe {
        libc::memfd_create(
            c"honk300-kwin".as_ptr(),
            libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut file = unsafe { File::from_raw_fd(descriptor) };
    file.write_all(bytes)?;
    let seals = libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_ADD_SEALS, seals) } < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek, SeekFrom};

    #[test]
    fn compositor_script_descriptor_preserves_bytes_and_refuses_mutation() {
        let bytes = b"print('native sealed companion');";
        let mut file = sealed_script(bytes).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        assert!(file.write_all(b"changed").is_err());
        assert!(file.set_len(1).is_err());
        let mut actual = Vec::new();
        file.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, bytes);
    }
}
