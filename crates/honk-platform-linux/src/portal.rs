//! Explicit, ephemeral RemoteDesktop permission and a libei pointer-only sender.
//! The optional system libraries are loaded only for a user-requested session.
use libloading::Library;
use std::{
    ffi::{c_char, c_int, c_void},
    io, ptr,
    time::{Duration, Instant},
};

type Object = *mut c_void;
const POINTER_ABSOLUTE: c_int = 2;
const VIRTUAL_DEVICE: c_int = 1;
const MAX_DEVICES: usize = 16;

macro_rules! api {
    ($name:ident, $library:literal, {$($field:ident: $signature:ty),+ $(,)?}) => {
        struct $name { $($field: $signature,)+ _library: Library }
        impl $name {
            fn load() -> io::Result<Self> {
                // SAFETY: fixed system ABI names; the retained Library outlives
                // every copied symbol and all objects created through it.
                unsafe {
                    let library = Library::new($library).map_err(|_| io::Error::new(
                        io::ErrorKind::Unsupported, concat!("Install the optional system library ", $library)))?;
                    Ok(Self { $($field: *library.get::<$signature>(concat!(stringify!($field), "\0").as_bytes())
                        .map_err(|_| io::Error::new(io::ErrorKind::Unsupported, "The installed input library lacks a required API"))?,)+
                        _library: library })
                }
            }
        }
    }
}

api!(Oeffis, "liboeffis.so.1", {
    oeffis_new: unsafe extern "C" fn(Object) -> Object,
    oeffis_unref: unsafe extern "C" fn(Object) -> Object,
    oeffis_create_session: unsafe extern "C" fn(Object, u32),
    oeffis_get_fd: unsafe extern "C" fn(Object) -> c_int,
    oeffis_get_eis_fd: unsafe extern "C" fn(Object) -> c_int,
    oeffis_dispatch: unsafe extern "C" fn(Object),
    oeffis_get_event: unsafe extern "C" fn(Object) -> c_int,
});
api!(Ei, "libei.so.1", {
    ei_new_sender: unsafe extern "C" fn(Object) -> Object,
    ei_unref: unsafe extern "C" fn(Object) -> Object,
    ei_configure_name: unsafe extern "C" fn(Object, *const c_char),
    ei_setup_backend_fd: unsafe extern "C" fn(Object, c_int) -> c_int,
    ei_get_fd: unsafe extern "C" fn(Object) -> c_int,
    ei_dispatch: unsafe extern "C" fn(Object),
    ei_get_event: unsafe extern "C" fn(Object) -> Object,
    ei_event_unref: unsafe extern "C" fn(Object) -> Object,
    ei_event_get_type: unsafe extern "C" fn(Object) -> c_int,
    ei_event_get_device: unsafe extern "C" fn(Object) -> Object,
    ei_event_get_seat: unsafe extern "C" fn(Object) -> Object,
    ei_seat_has_capability: unsafe extern "C" fn(Object, c_int) -> bool,
    ei_seat_bind_capabilities: unsafe extern "C" fn(Object, ...),
    ei_device_ref: unsafe extern "C" fn(Object) -> Object,
    ei_device_unref: unsafe extern "C" fn(Object) -> Object,
    ei_device_get_type: unsafe extern "C" fn(Object) -> c_int,
    ei_device_has_capability: unsafe extern "C" fn(Object, c_int) -> bool,
    ei_device_get_region_at: unsafe extern "C" fn(Object, f64, f64) -> Object,
    ei_device_start_emulating: unsafe extern "C" fn(Object, u32),
    ei_device_stop_emulating: unsafe extern "C" fn(Object),
    ei_device_pointer_motion_absolute: unsafe extern "C" fn(Object, f64, f64),
    ei_device_frame: unsafe extern "C" fn(Object, u64),
    ei_now: unsafe extern "C" fn(Object) -> u64,
});

struct Device {
    object: Object,
    resumed: bool,
}

/// Main-thread-only RAII owner. Drop closes the portal bus, EIS fd and devices.
/// There is deliberately no constructor accepting a socket path or restore token.
pub struct Session {
    portal: Object,
    context: Object,
    devices: Vec<Device>,
    started: Instant,
    sequence: u32,
    closed: bool,
    oeffis: Oeffis,
    ei: Ei,
}

impl Session {
    /// Call only in response to explicit user setup. The desktop owns the grant UI.
    pub fn request() -> io::Result<Self> {
        let oeffis = Oeffis::load()?;
        let ei = Ei::load()?;
        // SAFETY: no custom userdata or callback; null is supported by liboeffis.
        let portal = unsafe { (oeffis.oeffis_new)(ptr::null_mut()) };
        if portal.is_null() {
            return Err(io::Error::other("Cannot initialize the desktop portal"));
        }
        let session = Self {
            portal,
            context: ptr::null_mut(),
            devices: Vec::new(),
            started: Instant::now(),
            sequence: 0,
            closed: false,
            oeffis,
            ei,
        };
        // POINTER only. No keyboard, buttons, touch, screencast, or persistent token.
        unsafe {
            (session.oeffis.oeffis_create_session)(portal, 2);
        }
        Ok(session)
    }

    pub fn ready(&self) -> bool {
        !self.closed && self.devices.iter().any(|device| device.resumed)
    }

    pub fn poll(&mut self) -> io::Result<()> {
        let result = self.poll_inner();
        if result.is_err() {
            self.closed = true;
        }
        result
    }

    fn poll_inner(&mut self) -> io::Result<()> {
        if self.closed {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Pointer session closed",
            ));
        }
        if self.context.is_null() && self.started.elapsed() > Duration::from_secs(120) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Pointer permission was not granted in time",
            ));
        }
        // SAFETY: both contexts and the API libraries are retained by self;
        // dispatch is called only after nonblocking poll reports a readable fd.
        unsafe {
            if readable((self.oeffis.oeffis_get_fd)(self.portal))? {
                (self.oeffis.oeffis_dispatch)(self.portal);
            }
            for _ in 0..16 {
                match (self.oeffis.oeffis_get_event)(self.portal) {
                    0 => break,
                    1 => self.connect_eis()?,
                    2 => {
                        return Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "Desktop pointer permission ended",
                        ))
                    }
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::ConnectionAborted,
                            "Desktop pointer portal disconnected",
                        ))
                    }
                }
            }
            if self.context.is_null() {
                return Ok(());
            }
            if readable((self.ei.ei_get_fd)(self.context))? {
                (self.ei.ei_dispatch)(self.context);
            }
            for _ in 0..256 {
                let event = (self.ei.ei_get_event)(self.context);
                if event.is_null() {
                    return Ok(());
                }
                let result = self.event(event);
                (self.ei.ei_event_unref)(event);
                result?;
            }
        }
        Err(io::Error::other(
            "Input event queue exceeded its processing bound",
        ))
    }

    unsafe fn connect_eis(&mut self) -> io::Result<()> {
        if !self.context.is_null() {
            return Err(io::Error::other("Duplicate portal input connection"));
        }
        self.context = (self.ei.ei_new_sender)(ptr::null_mut());
        if self.context.is_null() {
            return Err(io::Error::other("Cannot initialize pointer input"));
        }
        (self.ei.ei_configure_name)(self.context, c"Honk300 pointer companion".as_ptr());
        let fd = (self.oeffis.oeffis_get_eis_fd)(self.portal);
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // ei_setup_backend_fd takes ownership, including teardown on failure.
        let result = (self.ei.ei_setup_backend_fd)(self.context, fd);
        if result < 0 {
            return Err(io::Error::from_raw_os_error(-result));
        }
        Ok(())
    }

    unsafe fn event(&mut self, event: Object) -> io::Result<()> {
        match (self.ei.ei_event_get_type)(event) {
            2 => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Granted input device disconnected",
                ))
            }
            3 => {
                let seat = (self.ei.ei_event_get_seat)(event);
                if !seat.is_null() && (self.ei.ei_seat_has_capability)(seat, POINTER_ABSOLUTE) {
                    (self.ei.ei_seat_bind_capabilities)(
                        seat,
                        POINTER_ABSOLUTE,
                        ptr::null::<c_void>(),
                    );
                }
            }
            5 => {
                let device = (self.ei.ei_event_get_device)(event);
                if !device.is_null()
                    && (self.ei.ei_device_get_type)(device) == VIRTUAL_DEVICE
                    && (self.ei.ei_device_has_capability)(device, POINTER_ABSOLUTE)
                {
                    if self.devices.len() == MAX_DEVICES
                        || self.devices.iter().any(|item| item.object == device)
                    {
                        return Err(io::Error::other(
                            "Input device identities exceeded their bound",
                        ));
                    }
                    self.devices.push(Device {
                        object: (self.ei.ei_device_ref)(device),
                        resumed: false,
                    });
                }
            }
            6..=8 => {
                let device = (self.ei.ei_event_get_device)(event);
                if let Some(index) = self.devices.iter().position(|item| item.object == device) {
                    match (self.ei.ei_event_get_type)(event) {
                        6 | 7 => return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                            "Granted pointer device was paused or removed; request permission again")),
                        8 => self.devices[index].resumed = true,
                        _ => unreachable!(),
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Obtain a newly authenticated compositor frame for each move.
    /// Session readiness alone never permits movement over protected/unknown windows.
    pub fn warp(&mut self, bridge: &crate::kwin::Bridge, target: [f64; 2]) -> io::Result<()> {
        self.poll()?;
        let frame = bridge.snapshot().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Pointer motion requires a fresh compositor observation",
            )
        })?;
        if !self.ready() || !frame.permits_pointer_motion(target) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Pointer motion lacks live safe target authority",
            ));
        }
        unsafe {
            let eligible = self
                .devices
                .iter()
                .filter(|device| {
                    device.resumed
                        && !(self.ei.ei_device_get_region_at)(
                            device.object,
                            frame.pointer[0],
                            frame.pointer[1],
                        )
                        .is_null()
                        && !(self.ei.ei_device_get_region_at)(device.object, target[0], target[1])
                            .is_null()
                })
                .collect::<Vec<_>>();
            if eligible.len() != 1 {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "Pointer region is unavailable or ambiguous",
                ));
            }
            self.sequence = self
                .sequence
                .checked_add(1)
                .ok_or_else(|| io::Error::other("Input sequence exhausted"))?;
            let device = eligible[0].object;
            (self.ei.ei_device_start_emulating)(device, self.sequence);
            (self.ei.ei_device_pointer_motion_absolute)(device, target[0], target[1]);
            (self.ei.ei_device_frame)(device, (self.ei.ei_now)(self.context));
            (self.ei.ei_device_stop_emulating)(device);
        }
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // SAFETY: release exactly the references created above before unloading
        // their libraries. The session bus lifetime owns the portal permission.
        unsafe {
            for device in self.devices.drain(..) {
                (self.ei.ei_device_unref)(device.object);
            }
            if !self.context.is_null() {
                (self.ei.ei_unref)(self.context);
            }
            (self.oeffis.oeffis_unref)(self.portal);
        }
    }
}

fn readable(fd: c_int) -> io::Result<bool> {
    if fd < 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "Input connection has no event descriptor",
        ));
    }
    let mut descriptor = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let result = unsafe { libc::poll(&mut descriptor, 1, 0) };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            return Ok(false);
        }
        return Err(error);
    }
    // Let the library consume its disconnect event on HUP/ERR as documented.
    Ok(result != 0)
}
