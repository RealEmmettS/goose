use super::{Frame, Version, MAX_AGE, MAX_REPLY};
use std::fs;
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Instant;

pub use crate::native_socket::Peer;
use crate::native_socket::{connect, peer, socket_identity, wait};

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
        if socket_identity(&self.socket, self.peer.uid)? != self.inode {
            return Err(invalid("Sway socket was replaced"));
        }
        let bytes = exchange(&mut self.stream, kind)?;
        if socket_identity(&self.socket, self.peer.uid)? != self.inode {
            return Err(invalid("Sway socket changed during observation"));
        }
        Ok(bytes)
    }
}

fn exchange(stream: &mut UnixStream, kind: u32) -> io::Result<Vec<u8>> {
    if !matches!(kind, 4 | 7) {
        return Err(invalid("Unsupported observation request"));
    }
    let deadline = Instant::now() + MAX_AGE;
    let mut header = *b"i3-ipc\0\0\0\0\0\0\0\0";
    header[10..14].copy_from_slice(&kind.to_ne_bytes());
    let mut offset = 0;
    while offset < header.len() {
        wait(stream.as_raw_fd(), libc::POLLOUT, deadline)?;
        match stream.write(&header[offset..]) {
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
    read_until(stream, &mut header, deadline)?;
    if &header[..6] != b"i3-ipc" || u32::from_ne_bytes(header[10..14].try_into().unwrap()) != kind {
        return Err(invalid("Invalid Sway IPC response header"));
    }
    let size = u32::from_ne_bytes(header[6..10].try_into().unwrap()) as usize;
    let limit = if kind == 7 { 4096 } else { MAX_REPLY };
    if size == 0 || size > limit {
        return Err(invalid("Sway IPC response exceeds its bound"));
    }
    let mut bytes = vec![0; size];
    read_until(stream, &mut bytes, deadline)?;
    Ok(bytes)
}

fn read_until(stream: &mut UnixStream, bytes: &mut [u8], deadline: Instant) -> io::Result<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        wait(stream.as_raw_fd(), libc::POLLIN, deadline)?;
        match stream.read(&mut bytes[offset..]) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    fn response(size: u32, kind: u32) -> [u8; 14] {
        let mut header = *b"i3-ipc\0\0\0\0\0\0\0\0";
        header[6..10].copy_from_slice(&size.to_ne_bytes());
        header[10..14].copy_from_slice(&kind.to_ne_bytes());
        header
    }

    #[test]
    fn actual_exchange_rejects_oversize_and_mismatched_headers_before_body_read() {
        for reply in [
            response(MAX_REPLY as u32 + 1, 4),
            response(4097, 7),
            response(2, 0),
        ] {
            let (mut client, mut server) = UnixStream::pair().unwrap();
            client.set_nonblocking(true).unwrap();
            let (done, hold) = std::sync::mpsc::channel::<()>();
            let server = thread::spawn(move || {
                let mut request = [0; 14];
                server.read_exact(&mut request).unwrap();
                server.write_all(&reply).unwrap();
                let _ = hold.recv_timeout(Duration::from_secs(2));
            });
            let kind = if reply[10] == 7 { 7 } else { 4 };
            assert_eq!(
                exchange(&mut client, kind).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
            done.send(()).unwrap();
            server.join().unwrap();
        }
    }

    #[test]
    fn partial_reply_cannot_extend_the_single_request_deadline() {
        let (mut client, mut server) = UnixStream::pair().unwrap();
        client.set_nonblocking(true).unwrap();
        let server = thread::spawn(move || {
            let mut request = [0; 14];
            server.read_exact(&mut request).unwrap();
            for byte in response(2, 4) {
                if server.write_all(&[byte]).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(80));
            }
        });
        let started = Instant::now();
        assert_eq!(
            exchange(&mut client, 4).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert!(started.elapsed() < Duration::from_millis(750));
        drop(client);
        server.join().unwrap();
    }

    #[test]
    fn actual_exchange_rejects_disconnect_and_accepts_a_complete_fragmented_reply() {
        for complete in [false, true] {
            let (mut client, mut server) = UnixStream::pair().unwrap();
            client.set_nonblocking(true).unwrap();
            let server = thread::spawn(move || {
                let mut request = [0; 14];
                server.read_exact(&mut request).unwrap();
                assert_eq!(request, response(0, 4));
                server.write_all(&response(2, 4)).unwrap();
                server.write_all(b"[").unwrap();
                if complete {
                    server.write_all(b"]").unwrap();
                }
            });
            let result = exchange(&mut client, 4);
            if complete {
                assert_eq!(result.unwrap(), b"[]");
            } else {
                assert!(result.is_err());
            }
            server.join().unwrap();
        }
    }
}
