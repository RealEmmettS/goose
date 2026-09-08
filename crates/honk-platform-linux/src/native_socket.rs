//! Bounded Unix socket primitives shared by independently authenticated observers.
use std::fs;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Instant;
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Peer {
    pub pid: u32,
    pub uid: u32,
}

pub(crate) fn wait(fd: i32, events: i16, deadline: Instant) -> io::Result<()> {
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::TimedOut, "Compositor IPC deadline exceeded")
            })?;
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
                "Compositor IPC disconnected",
            ));
        }
    }
}

pub(crate) fn connect(path: &Path, deadline: Instant) -> io::Result<UnixStream> {
    let bytes = path.as_os_str().as_bytes();
    // Zero initializes the NUL-terminated pathname and platform padding.
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if bytes.is_empty() || bytes.contains(&0) || bytes.len() >= address.sun_path.len() {
        return Err(invalid("Invalid Compositor socket pathname"));
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

pub(crate) fn peer(stream: &UnixStream) -> io::Result<Peer> {
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
        return Err(invalid("Invalid Compositor socket peer credentials"));
    }
    Ok(Peer {
        pid: credentials.pid as u32,
        uid: credentials.uid,
    })
}

pub(crate) fn socket_identity(path: &Path, uid: u32) -> io::Result<(u64, u64)> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket() || metadata.uid() != uid || metadata.mode() & 0o022 != 0 {
        return Err(invalid(
            "Compositor socket is not privately owned by this user",
        ));
    }
    Ok((metadata.dev(), metadata.ino()))
}
