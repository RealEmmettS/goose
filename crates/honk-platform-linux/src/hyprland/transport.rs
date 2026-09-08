use super::{Frame, Monitor, Version, MAX_AGE, MAX_REPLY};
pub use crate::native_socket::Peer;
use crate::native_socket::{connect, peer, socket_identity, wait};
use std::fs;
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Instant;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn location(uid: u32) -> io::Result<PathBuf> {
    let root = PathBuf::from(
        std::env::var_os("XDG_RUNTIME_DIR")
            .ok_or_else(|| invalid("A private runtime directory is required for Hyprland"))?,
    );
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| invalid("Hyprland has not supplied its instance identity"))?;
    if signature.is_empty()
        || signature.len() > 128
        || !signature
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'_')
    {
        return Err(invalid("Invalid Hyprland instance identity"));
    }
    if !root.is_absolute() {
        return Err(invalid("Hyprland runtime path must be absolute"));
    }
    for (path, mask) in [
        (&root, 0o077),
        (&root.join("hypr"), 0o022),
        (&root.join("hypr").join(&signature), 0o022),
    ] {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || metadata.uid() != uid
            || metadata.mode() & mask != 0
        {
            return Err(invalid(
                "Hyprland socket directories are not privately owned",
            ));
        }
    }
    Ok(root.join("hypr").join(signature).join(".socket.sock"))
}

/// Every request uses a fresh connection to the one initially authenticated
/// compositor. A replacement socket or process is never followed automatically.
pub struct Connection {
    socket: PathBuf,
    inode: (u64, u64),
    peer: Peer,
    executable: fs::File,
    version: Version,
}
impl Connection {
    pub fn connect() -> io::Result<Self> {
        // geteuid has no memory or ownership preconditions.
        let uid = unsafe { libc::geteuid() };
        let socket = location(uid)?;
        let inode = socket_identity(&socket, uid)?;
        let deadline = Instant::now() + MAX_AGE;
        let mut stream = connect(&socket, deadline)?;
        let peer = peer(&stream)?;
        if peer.uid != uid || socket_identity(&socket, uid)? != inode {
            return Err(invalid(
                "Hyprland socket ownership changed during connection",
            ));
        }
        let proc = PathBuf::from(format!("/proc/{}/exe", peer.pid));
        let path = fs::read_link(&proc)?;
        let executable = fs::File::open(proc)?;
        let metadata = executable.metadata()?;
        if path.file_name().is_none_or(|name| name != "Hyprland")
            || !metadata.is_file()
            || metadata.uid() != 0
            || metadata.mode() & 0o022 != 0
        {
            return Err(invalid(
                "Hyprland peer must be an unmodified system-owned compositor executable",
            ));
        }
        let version = Version::decode(&exchange(&mut stream, b"j/version", 4096, deadline)?)
            .map_err(invalid)?;
        Ok(Self {
            socket,
            inode,
            peer,
            executable,
            version,
        })
    }
    pub fn peer(&self) -> Peer {
        self.peer
    }
    pub fn version(&self) -> &Version {
        &self.version
    }
    fn request(&self, query: &[u8], deadline: Instant) -> io::Result<Vec<u8>> {
        if !matches!(query, b"j/monitors" | b"j/clients") {
            return Err(invalid("Unsupported Hyprland observation request"));
        }
        if location(self.peer.uid)? != self.socket
            || socket_identity(&self.socket, self.peer.uid)? != self.inode
        {
            return Err(invalid("Hyprland socket was replaced"));
        }
        let mut stream = connect(&self.socket, deadline)?;
        if peer(&stream)? != self.peer {
            return Err(invalid("Hyprland process identity changed"));
        }
        let current = fs::metadata(format!("/proc/{}/exe", self.peer.pid))?;
        let retained = self.executable.metadata()?;
        if (current.dev(), current.ino()) != (retained.dev(), retained.ino()) {
            return Err(invalid("Hyprland executable identity changed"));
        }
        let bytes = exchange(&mut stream, query, MAX_REPLY, deadline)?;
        if socket_identity(&self.socket, self.peer.uid)? != self.inode {
            return Err(invalid("Hyprland socket changed during observation"));
        }
        Ok(bytes)
    }
    pub fn snapshot(&self) -> io::Result<Frame> {
        // Hyprland supplies separate replies. Check active output/workspace
        // identity on both sides of the client read, under one total deadline.
        let deadline = Instant::now() + MAX_AGE;
        let before = Monitor::decode(&self.request(b"j/monitors", deadline)?).map_err(invalid)?;
        let clients = self.request(b"j/clients", deadline)?;
        let after = Monitor::decode(&self.request(b"j/monitors", deadline)?).map_err(invalid)?;
        if before != after {
            return Err(invalid("Hyprland output state changed during observation"));
        }
        Frame::decode(&self.version, &before, &clients).map_err(invalid)
    }
}

fn exchange(
    stream: &mut UnixStream,
    query: &[u8],
    limit: usize,
    deadline: Instant,
) -> io::Result<Vec<u8>> {
    let mut sent = 0;
    while sent < query.len() {
        wait(stream.as_raw_fd(), libc::POLLOUT, deadline)?;
        match stream.write(&query[sent..]) {
            Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
            Ok(count) => sent += count,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) => {}
            Err(error) => return Err(error),
        }
    }
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        wait(stream.as_raw_fd(), libc::POLLIN | libc::POLLHUP, deadline)?;
        match stream.read(&mut buffer) {
            Ok(0) if bytes.is_empty() => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(0) => return Ok(bytes),
            Ok(count) if count <= limit.saturating_sub(bytes.len()) => {
                bytes.extend_from_slice(&buffer[..count])
            }
            Ok(_) => return Err(invalid("Hyprland response exceeds its bound")),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) => {}
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};
    #[test]
    fn actual_exchange_requires_eof_and_never_resets_its_deadline() {
        for complete in [true, false] {
            let (mut client, mut server) = UnixStream::pair().unwrap();
            client.set_nonblocking(true).unwrap();
            let sender = thread::spawn(move || {
                let mut request = [0; 9];
                server.read_exact(&mut request).unwrap();
                assert_eq!(&request, b"j/clients");
                server.write_all(b"[").unwrap();
                server.write_all(b"]").unwrap();
                if !complete {
                    thread::sleep(Duration::from_millis(350));
                }
            });
            let result = exchange(&mut client, b"j/clients", 32, Instant::now() + MAX_AGE);
            if complete {
                assert_eq!(result.unwrap(), b"[]");
            } else {
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
            }
            sender.join().unwrap();
        }
    }
    #[test]
    fn oversized_reply_is_refused_without_waiting_for_close() {
        let (mut client, mut server) = UnixStream::pair().unwrap();
        client.set_nonblocking(true).unwrap();
        let (done, hold) = std::sync::mpsc::channel::<()>();
        let sender = thread::spawn(move || {
            let mut request = [0; 9];
            server.read_exact(&mut request).unwrap();
            server.write_all(&[b'x'; 65]).unwrap();
            let _ = hold.recv_timeout(Duration::from_secs(2));
        });
        assert_eq!(
            exchange(&mut client, b"j/clients", 64, Instant::now() + MAX_AGE)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        done.send(()).unwrap();
        sender.join().unwrap();
    }
}
