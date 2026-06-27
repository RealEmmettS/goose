#![cfg_attr(not(windows), allow(dead_code))]

use super::protocol::{ControlCommand, ControlResponse};
use std::io;
use std::sync::mpsc::{self, Receiver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingletonStatus {
    Acquired,
    AlreadyRunning,
}

pub struct Singleton {
    _imp: imp::Singleton,
}

impl Singleton {
    pub fn acquire() -> io::Result<(Self, SingletonStatus)> {
        imp::Singleton::acquire().map(|(imp, status)| (Self { _imp: imp }, status))
    }
}

pub struct CommandServer {
    _imp: imp::CommandServer,
    rx: Receiver<ControlCommand>,
}

impl CommandServer {
    pub fn start() -> io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let imp = imp::CommandServer::start(tx)?;
        Ok(Self { _imp: imp, rx })
    }

    pub fn try_recv(&self) -> Option<ControlCommand> {
        self.rx.try_recv().ok()
    }
}

pub fn send_command(command: ControlCommand) -> io::Result<ControlResponse> {
    imp::send_command(command)
}

#[cfg(windows)]
mod imp {
    use super::{ControlCommand, ControlResponse, SingletonStatus};
    use std::collections::hash_map::DefaultHasher;
    use std::fs::OpenOptions;
    use std::hash::{Hash, Hasher};
    use std::io::{self, Read, Write};
    use std::os::windows::io::FromRawHandle;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::Sender;
    use std::sync::Arc;
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
    use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
    use windows::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE,
        PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
    };
    use windows::Win32::System::Threading::CreateMutexW;

    pub struct Singleton {
        handle: Option<HANDLE>,
    }

    impl Singleton {
        pub fn acquire() -> io::Result<(Self, SingletonStatus)> {
            let name = wide_null(&mutex_name());
            let handle = unsafe { CreateMutexW(None, true, PCWSTR(name.as_ptr())) }
                .map_err(|_| io::Error::last_os_error())?;
            let already_running = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
            let status = if already_running {
                SingletonStatus::AlreadyRunning
            } else {
                SingletonStatus::Acquired
            };
            Ok((
                Self {
                    handle: Some(handle),
                },
                status,
            ))
        }
    }

    impl Drop for Singleton {
        fn drop(&mut self) {
            if let Some(handle) = self.handle.take() {
                unsafe {
                    let _ = CloseHandle(handle);
                }
            }
        }
    }

    pub struct CommandServer {
        shutdown: Arc<AtomicBool>,
        join: Option<JoinHandle<()>>,
    }

    impl CommandServer {
        pub fn start(tx: Sender<ControlCommand>) -> io::Result<Self> {
            let shutdown = Arc::new(AtomicBool::new(false));
            let thread_shutdown = Arc::clone(&shutdown);
            let join = thread::spawn(move || server_loop(tx, thread_shutdown));
            Ok(Self {
                shutdown,
                join: Some(join),
            })
        }
    }

    impl Drop for CommandServer {
        fn drop(&mut self) {
            self.shutdown.store(true, Ordering::SeqCst);
            let _ = send_command(ControlCommand::Reload);
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }

    pub fn send_command(command: ControlCommand) -> io::Result<ControlResponse> {
        let deadline = Instant::now() + Duration::from_secs(2);
        let path = pipe_path();
        let mut last_error = None;
        while Instant::now() < deadline {
            match OpenOptions::new().read(true).write(true).open(&path) {
                Ok(mut pipe) => {
                    pipe.write_all(command.encode().as_bytes())?;
                    pipe.flush()?;
                    let mut buf = [0u8; 128];
                    let len = pipe.read(&mut buf)?;
                    return ControlResponse::decode(&buf[..len])
                        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err));
                }
                Err(err) => {
                    last_error = Some(err);
                    thread::sleep(Duration::from_millis(25));
                }
            }
        }
        let err = last_error.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "no running honk300 instance")
        });
        if err.kind() == io::ErrorKind::NotFound {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no running honk300 instance",
            ))
        } else {
            Err(err)
        }
    }

    fn server_loop(tx: Sender<ControlCommand>, shutdown: Arc<AtomicBool>) {
        while !shutdown.load(Ordering::SeqCst) {
            let pipe = match create_pipe() {
                Ok(pipe) => pipe,
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            };
            let connected = unsafe { ConnectNamedPipe(pipe, None).is_ok() };
            if shutdown.load(Ordering::SeqCst) {
                unsafe {
                    let _ = CloseHandle(pipe);
                }
                break;
            }
            if !connected {
                unsafe {
                    let _ = CloseHandle(pipe);
                }
                continue;
            }

            let mut file = unsafe { std::fs::File::from_raw_handle(pipe.0) };
            let mut buf = [0u8; 128];
            let response = match file.read(&mut buf) {
                Ok(0) => ControlResponse::Err("EMPTY".into()),
                Ok(len) => match ControlCommand::decode(&buf[..len]) {
                    Ok(command) => match tx.send(command) {
                        Ok(()) => ControlResponse::Ok,
                        Err(_) => ControlResponse::Err("SERVER_CLOSED".into()),
                    },
                    Err(err) => ControlResponse::Err(protocol_code(&err.to_string())),
                },
                Err(_) => ControlResponse::Err("READ_FAILED".into()),
            };
            let _ = file.write_all(response.encode().as_bytes());
            let _ = file.flush();
        }
    }

    fn create_pipe() -> io::Result<HANDLE> {
        let name = wide_null(&pipe_path());
        let handle = unsafe {
            CreateNamedPipeW(
                PCWSTR(name.as_ptr()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                128,
                128,
                0,
                None,
            )
        };
        if handle.is_invalid() {
            Err(io::Error::last_os_error())
        } else {
            Ok(handle)
        }
    }

    fn mutex_name() -> String {
        format!("Local\\honk300-{}", user_hash())
    }

    fn pipe_path() -> String {
        format!(r"\\.\pipe\honk300-{}", user_hash())
    }

    fn user_hash() -> u64 {
        let user = std::env::var("USERNAME").unwrap_or_else(|_| "unknown".into());
        let domain = std::env::var("USERDOMAIN").unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        domain.hash(&mut hasher);
        user.hash(&mut hasher);
        hasher.finish()
    }

    fn wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn protocol_code(message: &str) -> String {
        message
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect()
    }
}

#[cfg(unix)]
mod imp {
    use super::{ControlCommand, ControlResponse, SingletonStatus};
    use std::fs::{self, File, OpenOptions};
    use std::io::{self, Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::Sender;
    use std::sync::Arc;
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    pub struct Singleton {
        _file: Option<File>,
        path: PathBuf,
        acquired: bool,
    }

    impl Singleton {
        pub fn acquire() -> io::Result<(Self, SingletonStatus)> {
            let dir = runtime_dir()?;
            fs::create_dir_all(&dir)?;
            let path = dir.join("lock");
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => Ok((
                    Self {
                        _file: Some(file),
                        path,
                        acquired: true,
                    },
                    SingletonStatus::Acquired,
                )),
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok((
                    Self {
                        _file: None,
                        path,
                        acquired: false,
                    },
                    SingletonStatus::AlreadyRunning,
                )),
                Err(err) => Err(err),
            }
        }
    }

    impl Drop for Singleton {
        fn drop(&mut self) {
            if self.acquired {
                let _ = fs::remove_file(&self.path);
            }
        }
    }

    pub struct CommandServer {
        shutdown: Arc<AtomicBool>,
        join: Option<JoinHandle<()>>,
        path: PathBuf,
    }

    impl CommandServer {
        pub fn start(tx: Sender<ControlCommand>) -> io::Result<Self> {
            let path = socket_path()?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = fs::remove_file(&path);
            let listener = UnixListener::bind(&path)?;
            listener.set_nonblocking(true)?;
            let shutdown = Arc::new(AtomicBool::new(false));
            let thread_shutdown = Arc::clone(&shutdown);
            let join = thread::spawn(move || server_loop(listener, tx, thread_shutdown));
            Ok(Self {
                shutdown,
                join: Some(join),
                path,
            })
        }
    }

    impl Drop for CommandServer {
        fn drop(&mut self) {
            self.shutdown.store(true, Ordering::SeqCst);
            let _ = send_command(ControlCommand::Reload);
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
            let _ = fs::remove_file(&self.path);
        }
    }

    pub fn send_command(command: ControlCommand) -> io::Result<ControlResponse> {
        let mut stream = UnixStream::connect(socket_path()?)?;
        stream.write_all(command.encode().as_bytes())?;
        stream.flush()?;
        let mut buf = [0u8; 128];
        let len = stream.read(&mut buf)?;
        ControlResponse::decode(&buf[..len])
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    fn server_loop(listener: UnixListener, tx: Sender<ControlCommand>, shutdown: Arc<AtomicBool>) {
        while !shutdown.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    if shutdown.load(Ordering::SeqCst) {
                        break;
                    }
                    let mut buf = [0u8; 128];
                    let response = match stream.read(&mut buf) {
                        Ok(0) => ControlResponse::Err("EMPTY".into()),
                        Ok(len) => match ControlCommand::decode(&buf[..len]) {
                            Ok(command) => match tx.send(command) {
                                Ok(()) => ControlResponse::Ok,
                                Err(_) => ControlResponse::Err("SERVER_CLOSED".into()),
                            },
                            Err(err) => ControlResponse::Err(err.to_string()),
                        },
                        Err(_) => ControlResponse::Err("READ_FAILED".into()),
                    };
                    let _ = stream.write_all(response.encode().as_bytes());
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(_) => thread::sleep(Duration::from_millis(100)),
            }
        }
    }

    fn socket_path() -> io::Result<PathBuf> {
        Ok(runtime_dir()?.join("control.sock"))
    }

    fn runtime_dir() -> io::Result<PathBuf> {
        if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            return Ok(PathBuf::from(dir).join("honk300"));
        }
        Ok(std::env::temp_dir().join(format!("honk300-{}", user_id())))
    }

    fn user_id() -> String {
        std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_else(|_| "unknown".into())
    }
}

#[cfg(not(any(windows, unix)))]
mod imp {
    use super::{ControlCommand, ControlResponse, SingletonStatus};
    use std::io;
    use std::sync::mpsc::Sender;

    pub struct Singleton;

    impl Singleton {
        pub fn acquire() -> io::Result<(Self, SingletonStatus)> {
            Ok((Self, SingletonStatus::Acquired))
        }
    }

    pub struct CommandServer;

    impl CommandServer {
        pub fn start(_tx: Sender<ControlCommand>) -> io::Result<Self> {
            Ok(Self)
        }
    }

    pub fn send_command(_command: ControlCommand) -> io::Result<ControlResponse> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "IPC is unsupported on this platform",
        ))
    }
}
