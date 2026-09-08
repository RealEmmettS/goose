use super::{Frame, MAX_AGE};
use crate::native_observer::{Observer as Worker, Source};
use std::fs::{self, File};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, Instant};
use zbus::{connection, Connection as BusConnection, Proxy};

const SHELL: &str = "org.gnome.Shell";
const PATH: &str = "/dev/emmetts/Honk300/Gnome1";
const IFACE: &str = "dev.emmetts.Honk300.Gnome1";
// Authentication, owner checks and response decoding share one total deadline.
fn bounded<T>(
    duration: Duration,
    work: impl std::future::Future<Output = io::Result<T>>,
) -> io::Result<T> {
    async_io::block_on(futures_lite::future::or(work, async {
        async_io::Timer::after(duration).await;
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "GNOME observation deadline exceeded",
        ))
    }))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn bus_error(error: zbus::Error) -> io::Error {
    match error {
        zbus::Error::MethodError(name, message, _)
            if matches!(
                name.as_str(),
                "dev.emmetts.Honk300.Gnome1.Deadline" | "dev.emmetts.Honk300.Gnome1.Unavailable"
            ) =>
        {
            // Only these fixed provider phases may reach logs. Never forward an
            // arbitrary remote description, consent record or window identity.
            let phase = match message.as_deref() {
                Some("GNOME observation failed during admission") => "admission",
                Some("GNOME observation failed during consent-before") => "consent before query",
                Some("GNOME observation failed during credentials") => "caller credentials",
                Some("GNOME observation failed during caller") => "caller executable",
                Some("GNOME observation failed during consent-after") => "consent after query",
                Some("GNOME observation failed during windows") => "window observation",
                _ => "unknown phase",
            };
            let expired = name.as_str().ends_with(".Deadline");
            io::Error::new(
                if expired {
                    io::ErrorKind::TimedOut
                } else {
                    io::ErrorKind::Other
                },
                format!(
                    "GNOME {} during {phase}",
                    if expired {
                        "deadline exceeded"
                    } else {
                        "observation failed"
                    }
                ),
            )
        }
        zbus::Error::MethodError(name, _, _)
            if name.as_str() == "dev.emmetts.Honk300.Gnome1.Revoked" =>
        {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "GNOME observations revoked",
            )
        }
        zbus::Error::MethodError(name, _, _)
            if name.as_str() == "dev.emmetts.Honk300.Gnome1.Busy" =>
        {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "GNOME credential checks are busy",
            )
        }
        zbus::Error::InputOutput(error) if error.kind() == io::ErrorKind::TimedOut => {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "GNOME observation deadline exceeded",
            )
        }
        _ => io::Error::other("GNOME bus or extension is unavailable; repeat explicit setup"),
    }
}

/// One bus connection pins a unique Shell owner and its loaded system executable.
/// No window or pointer action is represented by this transport.
pub struct Connection {
    bus: BusConnection,
    owner: String,
    pid: u32,
    executable: File,
    nonce: String,
    build: String,
}

impl Connection {
    pub fn connect(nonce: String, build: String) -> io::Result<Self> {
        if build.len() != 64
            || !build.bytes().all(|byte| byte.is_ascii_hexdigit())
            || nonce.len() != 32
            || !nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(invalid("Invalid GNOME consent identity"));
        }
        Self::connect_using(
            nonce,
            build,
            connection::Builder::session().map_err(bus_error)?,
        )
    }

    fn connect_using(
        nonce: String,
        build: String,
        builder: connection::Builder<'_>,
    ) -> io::Result<Self> {
        bounded(MAX_AGE, Self::connect_async(nonce, build, builder))
    }

    async fn connect_async(
        nonce: String,
        build: String,
        builder: connection::Builder<'_>,
    ) -> io::Result<Self> {
        let connection = builder
            .method_timeout(MAX_AGE)
            .max_queued(4)
            .build()
            .await
            .map_err(bus_error)?;
        let bus = Proxy::new(
            &connection,
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
        )
        .await
        .map_err(bus_error)?;
        let owner: String = bus
            .call("GetNameOwner", &(SHELL,))
            .await
            .map_err(bus_error)?;
        let uid: u32 = bus
            .call("GetConnectionUnixUser", &(&owner,))
            .await
            .map_err(bus_error)?;
        let pid: u32 = bus
            .call("GetConnectionUnixProcessID", &(&owner,))
            .await
            .map_err(bus_error)?;
        if uid != unsafe { libc::geteuid() } || pid == 0 {
            return Err(invalid("GNOME Shell has an unrelated bus identity"));
        }
        let path = format!("/proc/{pid}/exe");
        let resolved = fs::read_link(&path)?;
        let executable = File::open(&path)?;
        let metadata = executable.metadata()?;
        if resolved
            .file_name()
            .is_none_or(|name| name != "gnome-shell")
            || !metadata.is_file()
            || metadata.uid() != 0
            || metadata.mode() & 0o022 != 0
        {
            return Err(invalid(
                "GNOME peer must use the system-owned Shell executable",
            ));
        }
        drop(bus);
        Ok(Self {
            bus: connection,
            owner,
            pid,
            executable,
            nonce,
            build,
        })
    }

    async fn current(&self) -> io::Result<()> {
        let bus = Proxy::new(
            &self.bus,
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
        )
        .await
        .map_err(bus_error)?;
        let owner: String = bus
            .call("GetNameOwner", &(SHELL,))
            .await
            .map_err(bus_error)?;
        let now = fs::metadata(format!("/proc/{}/exe", self.pid))?;
        let pinned = self.executable.metadata()?;
        if owner != self.owner
            || now.dev() != pinned.dev()
            || now.ino() != pinned.ino()
            || now.len() != pinned.len()
            || now.uid() != 0
            || now.mode() & 0o022 != 0
        {
            return Err(invalid("The pinned GNOME Shell owner changed"));
        }
        Ok(())
    }

    pub fn snapshot(&mut self) -> io::Result<Frame> {
        bounded(MAX_AGE, self.snapshot_async())
    }

    async fn snapshot_async(&self) -> io::Result<Frame> {
        self.current().await?;
        let extension = Proxy::new(&self.bus, self.owner.as_str(), PATH, IFACE)
            .await
            .map_err(bus_error)?;
        let raw: String = extension
            .call("Snapshot", &(&self.nonce,))
            .await
            .map_err(bus_error)?;
        self.current().await?;
        Frame::decode(raw.as_bytes(), self.pid, &self.build).map_err(invalid)
    }

    /// Called only by explicit setup/removal after validating the exact owned
    /// companion. It never changes the global extension switch or another UUID.
    pub fn set_companion_enabled(&self, enabled: bool) -> io::Result<bool> {
        bounded(
            Duration::from_secs(3),
            self.set_companion_enabled_async(enabled),
        )
    }

    async fn set_companion_enabled_async(&self, enabled: bool) -> io::Result<bool> {
        self.current().await?;
        let extensions = Proxy::new(
            &self.bus,
            self.owner.as_str(),
            "/org/gnome/Shell",
            "org.gnome.Shell.Extensions",
        )
        .await
        .map_err(bus_error)?;
        let version: String = extensions
            .get_property("ShellVersion")
            .await
            .map_err(bus_error)?;
        if !matches!(version.as_str(), "46.0" | "48.7") {
            return Err(invalid(
                "This GNOME Shell version has not passed native qualification",
            ));
        }
        let method = if enabled {
            "EnableExtension"
        } else {
            "DisableExtension"
        };
        let accepted: bool = extensions
            .call(method, &("honk300@emmetts.dev",))
            .await
            .map_err(bus_error)?;
        if !accepted {
            return Ok(false);
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            self.current().await?;
            let info: std::collections::HashMap<String, zbus::zvariant::OwnedValue> = extensions
                .call("GetExtensionInfo", &("honk300@emmetts.dev",))
                .await
                .map_err(bus_error)?;
            let state = info
                .get("state")
                .and_then(|value| f64::try_from(value).ok());
            if if enabled {
                state == Some(1.0)
            } else {
                !matches!(state, Some(1.0 | 8.0))
            } {
                self.current().await?;
                return Ok(true);
            }
            async_io::Timer::after(Duration::from_millis(25)).await;
        }
        Ok(false)
    }
}

impl Source for Connection {
    type Frame = Frame;
    const NAME: &'static str = "gnome";
    const MAX_AGE: Duration = MAX_AGE;
    fn connect() -> io::Result<Self> {
        Err(invalid("GNOME requires explicit companion consent"))
    }
    fn snapshot(&mut self) -> io::Result<Frame> {
        self.snapshot()
    }
    fn retryable(error: &io::Error) -> bool {
        // A bounded busy response from the pinned Shell may retry that same
        // owner. Stale frames are withdrawn; every authority/error change ends it.
        error.kind() == io::ErrorKind::WouldBlock
    }
}

pub struct Observer(Worker<Connection>);
impl Observer {
    pub fn start(nonce: String, build: String) -> io::Result<Self> {
        Worker::spawn(move || Connection::connect(nonce, build)).map(Self)
    }
    pub fn snapshot(&self) -> Option<Frame> {
        self.0.snapshot()
    }
    pub fn running(&self) -> bool {
        self.0.running()
    }
    pub fn failed(&self) -> bool {
        self.0.failed()
    }
    pub fn revoked(&self) -> bool {
        self.0.revoked()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::os::unix::net::UnixListener;

    #[test]
    fn real_bus_authentication_is_bounded_before_any_shell_query() {
        let directory =
            std::env::temp_dir().join(format!("honk-gnome-auth-{}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let socket = directory.join("bus");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = std::thread::spawn(move || {
            let (mut peer, _) = listener.accept().unwrap();
            peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            let mut bytes = [0_u8; 256];
            let mut received = 0;
            loop {
                match peer.read(&mut bytes) {
                    Ok(0) => break,
                    Ok(count) => received += count,
                    Err(error) => panic!("Cancelled authentication retained its peer: {error}"),
                }
            }
            assert!(
                received > 0,
                "The production connection never began real bus authentication"
            );
        });
        let address = format!("unix:path={}", socket.display());
        let builder = connection::Builder::address(address.as_str()).unwrap();
        let started = Instant::now();
        let result = Connection::connect_using("a".repeat(32), "b".repeat(64), builder);
        assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::TimedOut));
        assert!(started.elapsed() < Duration::from_millis(600));
        server.join().unwrap();
        fs::remove_file(socket).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
