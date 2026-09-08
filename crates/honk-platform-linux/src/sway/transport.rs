use super::{Frame, Version, MAX_AGE, MAX_REPLY};
use std::fs;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Peer {
    pub pid: u32,
    pub uid: u32,
}

/// One exact compositor connection. There is intentionally no general IPC
/// command method: only version discovery and tree observation are implemented.
pub struct Connection {
    stream: UnixStream,
    socket: PathBuf,
    inode: (u64, u64),
    peer: Peer,
    version: Version,
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn wait(fd: i32, events: i16, deadline: Instant) -> io::Result<()> {
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "Sway IPC deadline exceeded"))?;
        let timeout = remaining.as_millis().clamp(1, 250) as i32;
        let mut entry = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        // entry is one initialized descriptor retained by the caller.
        let result = unsafe { libc::poll(&mut entry, 1, timeout) };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if result > 0 {
            if entry.revents & events != 0 {
                return Ok(());
            }
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Sway IPC disconnected",
            ));
        }
    }
}

fn connect(path: &Path, deadline: Instant) -> io::Result<UnixStream> {
    let bytes = path.as_os_str().as_bytes();
    // Zero initializes the NUL-terminated pathname and platform padding.
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if bytes.is_empty() || bytes.contains(&0) || bytes.len() >= address.sun_path.len() {
        return Err(invalid("Invalid Sway socket pathname"));
    }
    address.sun_family = libc::AF_UNIX as libc::sa_family_t;
    for (out, byte) in address.sun_path.iter_mut().zip(bytes) {
        *out = *byte as libc::c_char;
    }
    // Both flags are set atomically so no inherited descriptor or blocking
    // connect can escape the bounded optional adapter owner.
    let fd = unsafe {
        libc::socket(
            libc::AF_UNIX,
            libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
            0,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // The successful socket call transfers this fresh descriptor exactly once.
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    let size =
        (std::mem::offset_of!(libc::sockaddr_un, sun_path) + bytes.len() + 1) as libc::socklen_t;
    // address is initialized and size stays inside its pathname capacity.
    if unsafe { libc::connect(fd, (&address as *const libc::sockaddr_un).cast(), size) } < 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINPROGRESS) {
            return Err(error);
        }
        wait(fd, libc::POLLOUT, deadline)?;
        let mut pending = 0_i32;
        let mut length = std::mem::size_of_val(&pending) as libc::socklen_t;
        // pending and length describe writable storage of the expected type.
        if unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                (&mut pending as *mut i32).cast(),
                &mut length,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        if pending != 0 {
            return Err(io::Error::from_raw_os_error(pending));
        }
    }
    Ok(UnixStream::from(owned))
}

fn peer(stream: &UnixStream) -> io::Result<Peer> {
    // ucred is a plain output structure with no invalid zero representation.
    let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
    let mut length = std::mem::size_of_val(&credentials) as libc::socklen_t;
    // The retained socket and output buffer remain valid throughout the call.
    if unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credentials as *mut libc::ucred).cast(),
            &mut length,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    if length as usize != std::mem::size_of_val(&credentials) || credentials.pid <= 0 {
        return Err(invalid("Invalid Sway socket peer credentials"));
    }
    Ok(Peer {
        pid: credentials.pid as u32,
        uid: credentials.uid,
    })
}

fn socket_identity(path: &Path, uid: u32) -> io::Result<(u64, u64)> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket() || metadata.uid() != uid || metadata.mode() & 0o022 != 0 {
        return Err(invalid("Sway socket is not privately owned by this user"));
    }
    Ok((metadata.dev(), metadata.ino()))
}

impl Connection {
    pub fn connect() -> io::Result<Self> {
        let runtime = PathBuf::from(
            std::env::var_os("XDG_RUNTIME_DIR")
                .ok_or_else(|| invalid("A private runtime directory is required for Sway"))?,
        );
        let socket = PathBuf::from(
            std::env::var_os("SWAYSOCK")
                .ok_or_else(|| invalid("Sway has not supplied its IPC socket"))?,
        );
        // geteuid has no memory or ownership preconditions.
        let uid = unsafe { libc::geteuid() };
        let metadata = fs::symlink_metadata(&runtime)?;
        if !runtime.is_absolute()
            || !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || metadata.uid() != uid
            || metadata.mode() & 0o077 != 0
            || socket.parent() != Some(runtime.as_path())
        {
            return Err(invalid(
                "Sway IPC must belong to the private runtime directory",
            ));
        }
        let inode = socket_identity(&socket, uid)?;
        let stream = connect(&socket, Instant::now() + MAX_AGE)?;
        let peer = peer(&stream)?;
        if peer.uid != uid || socket_identity(&socket, uid)? != inode {
            return Err(invalid("Sway socket ownership changed during connection"));
        }
        let executable = PathBuf::from(format!("/proc/{}/exe", peer.pid));
        let path = fs::read_link(&executable)?;
        let file = fs::File::open(&executable)?;
        let metadata = file.metadata()?;
        if path.file_name().is_none_or(|name| name != "sway")
            || !metadata.is_file()
            || metadata.uid() != 0
            || metadata.mode() & 0o022 != 0
        {
            return Err(invalid(
                "Sway peer must be an unmodified system-owned compositor executable",
            ));
        }
        // The field is replaced only after actual native version discovery.
        let mut connection = Self {
            stream,
            socket,
            inode,
            peer,
            version: Version {
                major: 0,
                minor: 0,
                patch: 0,
                variant: String::new(),
            },
        };
        connection.version = Version::decode(&connection.request(7)?).map_err(invalid)?;
        Ok(connection)
    }

    pub fn peer(&self) -> Peer {
        self.peer
    }
    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn snapshot(&mut self) -> io::Result<Frame> {
        Frame::decode(&self.request(4)?).map_err(invalid)
    }

    fn request(&mut self, kind: u32) -> io::Result<Vec<u8>> {
        if !matches!(kind, 4 | 7) {
            return Err(invalid("Unsupported observation request"));
        }
        if socket_identity(&self.socket, self.peer.uid)? != self.inode {
            return Err(invalid("Sway socket was replaced"));
        }
        let deadline = Instant::now() + MAX_AGE;
        let mut header = *b"i3-ipc\0\0\0\0\0\0\0\0";
        header[10..14].copy_from_slice(&kind.to_ne_bytes());
        let mut offset = 0;
        while offset < header.len() {
            wait(self.stream.as_raw_fd(), libc::POLLOUT, deadline)?;
            match self.stream.write(&header[offset..]) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(count) => offset += count,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(error) => return Err(error),
            }
        }
        self.read(&mut header, deadline)?;
        if &header[..6] != b"i3-ipc"
            || u32::from_ne_bytes(header[10..14].try_into().unwrap()) != kind
        {
            return Err(invalid("Invalid Sway IPC response header"));
        }
        let size = u32::from_ne_bytes(header[6..10].try_into().unwrap()) as usize;
        let limit = if kind == 7 { 4096 } else { MAX_REPLY };
        if size == 0 || size > limit {
            return Err(invalid("Sway IPC response exceeds its bound"));
        }
        let mut bytes = vec![0; size];
        self.read(&mut bytes, deadline)?;
        Ok(bytes)
    }

    fn read(&mut self, bytes: &mut [u8], deadline: Instant) -> io::Result<()> {
        let mut offset = 0;
        while offset < bytes.len() {
            wait(self.stream.as_raw_fd(), libc::POLLIN, deadline)?;
            match self.stream.read(&mut bytes[offset..]) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(count) => offset += count,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
