//! Linux platform helpers for honk300.
//!
//! M17/M18 are intentionally split by display-server capability. This crate keeps the
//! session detection, local-time sampling, fallback bounds, and terminal-target classifier
//! out of `honk-engine` while the X11/Wayland presentation backends continue to mature.

use honk_engine::{LocalTime, Rect, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

impl DisplayServer {
    pub fn label(self) -> &'static str {
        match self {
            Self::X11 => "X11/XWayland",
            Self::Wayland => "Wayland",
            Self::Unknown => "unknown Linux display server",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub display_server: DisplayServer,
    pub display: Option<String>,
    pub wayland_display: Option<String>,
    pub xdg_session_type: Option<String>,
    pub forced_wayland: bool,
}

impl SessionInfo {
    pub fn detect(force_wayland: bool) -> Self {
        let display = non_empty_env("DISPLAY");
        let wayland_display = non_empty_env("WAYLAND_DISPLAY");
        let xdg_session_type = non_empty_env("XDG_SESSION_TYPE");
        let display_server = detect_display_server(
            xdg_session_type.as_deref(),
            display.as_deref(),
            wayland_display.as_deref(),
            force_wayland,
        );
        Self {
            display_server,
            display,
            wayland_display,
            xdg_session_type,
            forced_wayland: force_wayland,
        }
    }
}

pub fn detect_display_server(
    xdg_session_type: Option<&str>,
    display: Option<&str>,
    wayland_display: Option<&str>,
    force_wayland: bool,
) -> DisplayServer {
    if force_wayland {
        return DisplayServer::Wayland;
    }

    if non_empty(display).is_some() {
        return DisplayServer::X11;
    }

    let session = xdg_session_type.map(|value| value.trim().to_ascii_lowercase());
    if session.as_deref() == Some("x11") {
        return DisplayServer::X11;
    }

    if non_empty(wayland_display).is_some() || session.as_deref() == Some("wayland") {
        return DisplayServer::Wayland;
    }

    DisplayServer::Unknown
}

pub fn default_world_bounds(session: DisplayServer) -> Rect {
    match session {
        DisplayServer::X11 | DisplayServer::Wayland | DisplayServer::Unknown => {
            Rect::new(Vec2::new(0.0, 0.0), Vec2::new(1280.0, 720.0))
        }
    }
}

pub fn local_time() -> LocalTime {
    imp::local_time()
}

pub fn presence_supported(_session: DisplayServer) -> bool {
    false
}

pub fn cursor_mischief_supported(_session: DisplayServer) -> bool {
    false
}

pub fn foreign_window_watch_supported(_session: DisplayServer) -> bool {
    false
}

pub fn collect_window_supported(_session: DisplayServer) -> bool {
    false
}

pub fn display_cursor_mischief_supported(session: DisplayServer) -> bool {
    session == DisplayServer::X11
}

pub fn display_foreign_window_watch_supported(session: DisplayServer) -> bool {
    session == DisplayServer::X11
}

pub fn display_collect_window_supported(_session: DisplayServer) -> bool {
    false
}

#[cfg(test)]
fn x11_overlay_count(monitor_count: usize) -> usize {
    monitor_count
}

#[cfg(any(test, target_os = "linux"))]
fn x11_mapping_allowed(argb: bool, compositor: bool, empty_input_shape: bool) -> bool {
    argb && compositor && empty_input_shape
}

#[cfg(any(test, target_os = "linux"))]
fn xfixes_supports_regions(major_version: u32) -> bool {
    major_version >= 2
}

#[cfg(test)]
fn x11_setup_steps() -> [&'static str; 2] {
    ["shape", "map"]
}

#[cfg(test)]
fn x11_event_requires_reconcile(randr_event: bool) -> bool {
    randr_event
}

#[cfg(any(test, target_os = "linux"))]
const WAYLAND_MAX_BUFFERS_PER_OUTPUT: usize = 3;

#[cfg(any(test, target_os = "linux"))]
fn wayland_scaled_dimension(logical: u32, scale_120: u32) -> u32 {
    wayland_scale_ceil(logical, scale_120).max(1)
}

#[cfg(any(test, target_os = "linux"))]
fn wayland_scale_floor(logical: u32, scale_120: u32) -> u32 {
    ((logical as u64 * scale_120 as u64) / 120) as u32
}

#[cfg(any(test, target_os = "linux"))]
fn wayland_scale_ceil(logical: u32, scale_120: u32) -> u32 {
    ((logical as u64 * scale_120 as u64).div_ceil(120)) as u32
}

#[cfg(test)]
fn wayland_retained_buffer_count(frame_count: usize) -> usize {
    frame_count.min(WAYLAND_MAX_BUFFERS_PER_OUTPUT)
}

#[cfg(test)]
fn wayland_surface_count(output_count: usize) -> usize {
    output_count
}

#[cfg(test)]
fn wayland_pump_steps() -> [&'static str; 4] {
    ["prepare_read", "poll", "read", "dispatch_pending"]
}

#[cfg(target_os = "linux")]
pub use platform::{Overlay, OverlayMode};

#[cfg(target_os = "linux")]
mod platform {
    use super::{default_world_bounds, DisplayServer};
    use honk_engine::tiny_skia::Pixmap;
    use honk_engine::{Pointer, Rect, Vec2};
    use std::io;

    pub struct Overlay {
        inner: OverlayInner,
    }

    enum OverlayInner {
        X11(x11::X11Overlay),
        Wayland(wayland::WaylandOverlay),
        Headless(HeadlessOverlay),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OverlayMode {
        X11,
        Wayland,
        Headless,
    }

    impl Overlay {
        pub fn new(preferred: DisplayServer) -> io::Result<Self> {
            let inner = match preferred {
                DisplayServer::X11 => match x11::X11Overlay::new() {
                    Ok(overlay) => OverlayInner::X11(overlay),
                    Err(err) => OverlayInner::Headless(headless_fallback_or_fail(
                        DisplayServer::X11,
                        "X11",
                        Some(&err),
                    )?),
                },
                DisplayServer::Wayland => match wayland::WaylandOverlay::new() {
                    Ok(overlay) => OverlayInner::Wayland(overlay),
                    Err(err) => OverlayInner::Headless(headless_fallback_or_fail(
                        DisplayServer::Wayland,
                        "Wayland layer-shell",
                        Some(&err),
                    )?),
                },
                DisplayServer::Unknown => OverlayInner::Headless(headless_fallback_or_fail(
                    DisplayServer::Unknown,
                    "Linux display server",
                    None,
                )?),
            };
            Ok(Self { inner })
        }

        pub fn mode(&self) -> OverlayMode {
            match &self.inner {
                OverlayInner::X11(_) => OverlayMode::X11,
                OverlayInner::Wayland(_) => OverlayMode::Wayland,
                OverlayInner::Headless(_) => OverlayMode::Headless,
            }
        }

        pub fn display_server(&self) -> DisplayServer {
            match &self.inner {
                OverlayInner::X11(_) => DisplayServer::X11,
                OverlayInner::Wayland(_) => DisplayServer::Wayland,
                OverlayInner::Headless(overlay) => overlay.display_server,
            }
        }

        pub fn bounds(&self) -> Rect {
            match &self.inner {
                OverlayInner::X11(overlay) => overlay.bounds(),
                OverlayInner::Wayland(overlay) => overlay.bounds(),
                OverlayInner::Headless(overlay) => overlay.bounds(),
            }
        }

        pub fn monitor_bounds(&self) -> Vec<Rect> {
            match &self.inner {
                OverlayInner::X11(overlay) => overlay.monitor_bounds(),
                OverlayInner::Wayland(overlay) => overlay.monitor_bounds(),
                OverlayInner::Headless(overlay) => vec![overlay.bounds()],
            }
        }

        pub fn take_topology_changed(&mut self) -> bool {
            match &mut self.inner {
                OverlayInner::X11(overlay) => overlay.take_topology_changed(),
                OverlayInner::Wayland(overlay) => overlay.take_topology_changed(),
                OverlayInner::Headless(_) => false,
            }
        }

        pub fn pointer_state(&self) -> Pointer {
            match &self.inner {
                OverlayInner::X11(overlay) => overlay.pointer_state().unwrap_or_default(),
                OverlayInner::Wayland(_) | OverlayInner::Headless(_) => Pointer::default(),
            }
        }

        pub fn foreign_window_drag(&self) -> Option<honk_engine::ForeignWindowSnapshot> {
            match &self.inner {
                OverlayInner::X11(overlay) => overlay.foreign_window_drag().ok().flatten(),
                OverlayInner::Wayland(_) | OverlayInner::Headless(_) => None,
            }
        }

        pub fn warp_cursor(&self, pos: Vec2) -> io::Result<()> {
            match &self.inner {
                OverlayInner::X11(overlay) => overlay.warp_cursor(pos),
                OverlayInner::Wayland(_) | OverlayInner::Headless(_) => Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "global cursor warp is unsupported in this Linux display mode",
                )),
            }
        }

        pub fn set_input_region(&mut self, rect: Option<Rect>) -> io::Result<()> {
            match &mut self.inner {
                OverlayInner::X11(overlay) => overlay.set_input_region(rect),
                OverlayInner::Wayland(overlay) => overlay.set_input_region(rect),
                OverlayInner::Headless(_) => Ok(()),
            }
        }

        pub fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
            maybe_write_smoke_frame(pixmap);
            match &mut self.inner {
                OverlayInner::X11(overlay) => overlay.present(dirty, pixmap),
                OverlayInner::Wayland(overlay) => overlay.present(dirty, pixmap),
                OverlayInner::Headless(_) => Ok(()),
            }
        }

        pub fn pump(&mut self) -> bool {
            match &mut self.inner {
                OverlayInner::X11(overlay) => overlay.pump().unwrap_or(false),
                OverlayInner::Wayland(overlay) => overlay.pump().unwrap_or(false),
                OverlayInner::Headless(_) => true,
            }
        }
    }

    struct HeadlessOverlay {
        display_server: DisplayServer,
        bounds: Rect,
    }

    impl HeadlessOverlay {
        fn new(display_server: DisplayServer) -> Self {
            Self {
                display_server,
                bounds: default_world_bounds(display_server),
            }
        }

        fn bounds(&self) -> Rect {
            self.bounds
        }
    }

    /// Opt-in escape hatch that permits the invisible headless overlay fallback.
    ///
    /// Without it, a failed (or entirely absent) X11/Wayland overlay is a hard, fatal start
    /// error instead of a silent no-op that leaves the process reporting "running" while it
    /// renders nothing. Hosted CI runners with no display set this to exercise headless on
    /// purpose.
    const ALLOW_HEADLESS_ENV: &str = "HONK300_ALLOW_HEADLESS";

    fn headless_allowed() -> bool {
        std::env::var(ALLOW_HEADLESS_ENV)
            .map(|value| value.trim() == "1")
            .unwrap_or(false)
    }

    /// Either build the invisible headless fallback (only when explicitly allowed) or fail loudly.
    ///
    /// `backend` names the visible overlay we could not bring up. `cause` carries the creation
    /// error when there was an attempt (X11/Wayland) and is `None` when no display server was
    /// detected at all. On refusal the returned error names the tried backend and the escape
    /// hatch so the failure is diagnosable rather than a zombie process.
    fn headless_fallback_or_fail(
        server: DisplayServer,
        backend: &str,
        cause: Option<&io::Error>,
    ) -> io::Result<HeadlessOverlay> {
        let reason = match cause {
            Some(err) => format!("{backend} overlay creation failed ({err})"),
            None => format!("no {backend} available (DISPLAY/WAYLAND_DISPLAY unset)"),
        };
        if headless_allowed() {
            eprintln!(
                "honk300: {reason}; {ALLOW_HEADLESS_ENV}=1 set — running headless (invisible, no rendering)."
            );
            Ok(HeadlessOverlay::new(server))
        } else {
            Err(io::Error::other(format!(
                "honk300: {reason}; refusing to run a headless no-op overlay. \
                 Set {ALLOW_HEADLESS_ENV}=1 to allow an invisible headless run."
            )))
        }
    }

    fn maybe_write_smoke_frame(pixmap: &Pixmap) {
        let Ok(path) = std::env::var("HONK300_SMOKE_FRAME") else {
            return;
        };
        if path.trim().is_empty() {
            return;
        }
        let _ = pixmap.save_png(path);
    }

    fn x11_bgra_from_rgba(pixmap: &Pixmap) -> Vec<u8> {
        let mut out = Vec::with_capacity(pixmap.data().len());
        for px in pixmap.data().chunks_exact(4) {
            out.push(px[2]);
            out.push(px[1]);
            out.push(px[0]);
            out.push(px[3]);
        }
        out
    }

    fn clamp_i16(value: f32) -> i16 {
        value.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16
    }

    fn clamp_u16(value: f32) -> u16 {
        value.ceil().clamp(1.0, u16::MAX as f32) as u16
    }

    mod x11 {
        use super::super::{x11_mapping_allowed, xfixes_supports_regions};
        use super::{clamp_i16, clamp_u16, x11_bgra_from_rgba};
        use honk_engine::tiny_skia::Pixmap;
        use honk_engine::{ForeignWindowId, ForeignWindowSnapshot, Pointer, Rect, Vec2};
        use std::collections::HashMap;
        use std::io;
        use x11rb::connection::Connection;
        use x11rb::protocol::randr::{ConnectionExt as RandrConnectionExt, NotifyMask};
        use x11rb::protocol::render::ConnectionExt as RenderConnectionExt;
        use x11rb::protocol::shape::{self, ConnectionExt as ShapeConnectionExt};
        use x11rb::protocol::xfixes::ConnectionExt as XFixesConnectionExt;
        use x11rb::protocol::xinerama::ConnectionExt as XineramaConnectionExt;
        use x11rb::protocol::xproto::{
            AtomEnum, ButtonMask, ColormapAlloc, ConfigureWindowAux,
            ConnectionExt as XprotoConnectionExt, CreateGCAux, CreateWindowAux, EventMask,
            GetPropertyReply, ImageFormat, PropMode, Rectangle, Screen, StackMode, Visualid,
            Window, WindowClass,
        };
        use x11rb::protocol::Event;
        use x11rb::rust_connection::RustConnection;
        use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;
        use x11rb::NONE;

        x11rb::atom_manager! {
            Atoms: AtomsCookie {
                _NET_WM_NAME,
                _NET_WM_WINDOW_TYPE,
                _NET_WM_WINDOW_TYPE_DOCK,
                _NET_WM_STATE,
                _NET_WM_STATE_ABOVE,
                UTF8_STRING,
            }
        }

        pub struct X11Overlay {
            conn: RustConnection,
            root: u32,
            screen_num: usize,
            windows: Vec<X11Window>,
            bounds: Rect,
            atoms: Atoms,
            topology_changed: bool,
        }

        #[derive(Debug, Clone, Copy, PartialEq)]
        struct Monitor {
            id: u32,
            bounds: Rect,
            primary: bool,
        }

        struct X11Window {
            monitor_id: u32,
            window: u32,
            gc: u32,
            depth: u8,
            colormap: Option<u32>,
            bounds: Rect,
            // Last input region actually applied (post-intersection with monitor bounds).
            last_input_region: Option<Option<Rect>>,
        }

        impl X11Overlay {
            pub fn new() -> io::Result<Self> {
                let (conn, screen_num) = x11rb::connect(None).map_err(to_io)?;
                let screen = &conn.setup().roots[screen_num];
                let root = screen.root;
                let atoms = Atoms::new(&conn).map_err(to_io)?.reply().map_err(to_io)?;
                initialize_input_shape_extensions(&conn)?;
                let visual = choose_argb_visual(&conn, screen_num)
                    .ok_or_else(|| io::Error::other("X11 overlay requires a 32-bit ARGB visual"))?;
                if !compositor_running(&conn, screen_num)? {
                    return Err(io::Error::other(
                        "X11 overlay requires an active compositing manager",
                    ));
                }
                let monitors = query_monitors(&conn, root, screen);
                let bounds = monitor_union(&monitors);
                let mut windows = Vec::with_capacity(monitors.len());
                for monitor in monitors {
                    windows.push(create_overlay_window(
                        &conn,
                        root,
                        screen.root_visual,
                        &atoms,
                        visual,
                        monitor,
                    )?);
                }
                conn.randr_select_input(
                    root,
                    NotifyMask::SCREEN_CHANGE
                        | NotifyMask::CRTC_CHANGE
                        | NotifyMask::OUTPUT_CHANGE
                        | NotifyMask::RESOURCE_CHANGE,
                )
                .map_err(to_io)?
                .check()
                .map_err(to_io)?;
                conn.flush().map_err(to_io)?;

                Ok(Self {
                    conn,
                    root,
                    screen_num,
                    windows,
                    bounds,
                    atoms,
                    topology_changed: false,
                })
            }

            pub fn bounds(&self) -> Rect {
                self.bounds
            }

            pub fn monitor_bounds(&self) -> Vec<Rect> {
                self.windows.iter().map(|window| window.bounds).collect()
            }

            pub fn take_topology_changed(&mut self) -> bool {
                std::mem::take(&mut self.topology_changed)
            }

            pub fn pointer_state(&self) -> io::Result<Pointer> {
                let reply = self
                    .conn
                    .query_pointer(self.root)
                    .map_err(to_io)?
                    .reply()
                    .map_err(to_io)?;
                Ok(Pointer {
                    pos: Vec2::new(reply.root_x as f32, reply.root_y as f32),
                    present: reply.same_screen,
                    left_down: reply.mask.contains(ButtonMask::M1),
                })
            }

            pub fn foreign_window_drag(&self) -> io::Result<Option<ForeignWindowSnapshot>> {
                let pointer = self.pointer_state()?;
                if !pointer.left_down {
                    return Ok(None);
                }
                let focus = self
                    .conn
                    .get_input_focus()
                    .map_err(to_io)?
                    .reply()
                    .map_err(to_io)?
                    .focus;
                let Some(window) = self.foreign_target_window(focus)? else {
                    return Ok(None);
                };
                if self.is_protected_window(window)? {
                    return Ok(None);
                }
                let geometry = self
                    .conn
                    .get_geometry(window)
                    .map_err(to_io)?
                    .reply()
                    .map_err(to_io)?;
                if geometry.width <= 1 || geometry.height <= 1 {
                    return Ok(None);
                }
                let translated = self
                    .conn
                    .translate_coordinates(window, self.root, 0, 0)
                    .map_err(to_io)?
                    .reply()
                    .map_err(to_io)?;
                let rect = Rect::new(
                    Vec2::new(translated.dst_x as f32, translated.dst_y as f32),
                    Vec2::new(
                        translated.dst_x as f32 + geometry.width as f32,
                        translated.dst_y as f32 + geometry.height as f32,
                    ),
                );
                Ok(Some(ForeignWindowSnapshot::top_center(
                    ForeignWindowId(window as u64),
                    rect,
                )))
            }

            pub fn warp_cursor(&self, pos: Vec2) -> io::Result<()> {
                self.conn
                    .warp_pointer(
                        NONE,
                        self.root,
                        0,
                        0,
                        0,
                        0,
                        clamp_i16(pos.x),
                        clamp_i16(pos.y),
                    )
                    .map_err(to_io)?;
                self.conn.flush().map_err(to_io)
            }

            pub fn set_input_region(&mut self, rect: Option<Rect>) -> io::Result<()> {
                for window in &mut self.windows {
                    let effective = rect.and_then(|rect| rect.intersection(window.bounds));
                    if window.last_input_region == Some(effective) {
                        continue;
                    }
                    apply_input_shape(&self.conn, window.window, window.bounds, effective)?;
                    window.last_input_region = Some(effective);
                }
                self.conn.flush().map_err(to_io)?;
                Ok(())
            }

            pub fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
                if pixmap.width() == 0 || pixmap.height() == 0 {
                    return Ok(());
                }
                for window in &self.windows {
                    let Some(clip) = dirty.intersection(window.bounds).map(Rect::pixel_aligned)
                    else {
                        continue;
                    };
                    let width = clamp_u16(clip.width());
                    let height = clamp_u16(clip.height());
                    let data = clipped_bgra(dirty, clip, pixmap, width, height);
                    self.conn
                        .put_image(
                            ImageFormat::Z_PIXMAP,
                            window.window,
                            window.gc,
                            width,
                            height,
                            clamp_i16(clip.min.x - window.bounds.min.x),
                            clamp_i16(clip.min.y - window.bounds.min.y),
                            0,
                            window.depth,
                            &data,
                        )
                        .map_err(to_io)?;
                }
                self.conn.flush().map_err(to_io)
            }

            pub fn pump(&mut self) -> io::Result<bool> {
                let mut reconcile = false;
                while let Some(event) = self.conn.poll_for_event().map_err(to_io)? {
                    if matches!(
                        event,
                        Event::RandrNotify(_) | Event::RandrScreenChangeNotify(_)
                    ) {
                        reconcile = true;
                    }
                }
                if reconcile {
                    self.reconcile_monitors()?;
                }
                Ok(true)
            }

            fn foreign_target_window(&self, focus: Window) -> io::Result<Option<Window>> {
                if focus == NONE
                    || focus == self.root
                    || self.windows.iter().any(|overlay| overlay.window == focus)
                {
                    return Ok(None);
                }
                let attrs = self
                    .conn
                    .get_window_attributes(focus)
                    .map_err(to_io)?
                    .reply()
                    .map_err(to_io)?;
                if attrs.override_redirect {
                    return Ok(None);
                }
                Ok(Some(focus))
            }

            fn reconcile_monitors(&mut self) -> io::Result<()> {
                let screen = &self.conn.setup().roots[self.screen_num];
                let monitors = query_monitors(&self.conn, self.root, screen);
                let visual = choose_argb_visual(&self.conn, self.screen_num)
                    .ok_or_else(|| io::Error::other("X11 overlay lost its required ARGB visual"))?;
                if !compositor_running(&self.conn, self.screen_num)? {
                    return Err(io::Error::other(
                        "X11 compositing manager disappeared during topology refresh",
                    ));
                }
                let prior = self.monitor_bounds();
                let mut existing = self
                    .windows
                    .drain(..)
                    .map(|window| (window.monitor_id, window))
                    .collect::<HashMap<_, _>>();
                let mut windows = Vec::with_capacity(monitors.len());
                for monitor in &monitors {
                    let mut window = match existing.remove(&monitor.id) {
                        Some(window) => window,
                        None => create_overlay_window(
                            &self.conn,
                            self.root,
                            screen.root_visual,
                            &self.atoms,
                            visual,
                            *monitor,
                        )?,
                    };
                    if window.bounds != monitor.bounds {
                        self.conn
                            .configure_window(
                                window.window,
                                &ConfigureWindowAux::new()
                                    .x(clamp_i16(monitor.bounds.min.x) as i32)
                                    .y(clamp_i16(monitor.bounds.min.y) as i32)
                                    .width(clamp_u16(monitor.bounds.width()) as u32)
                                    .height(clamp_u16(monitor.bounds.height()) as u32),
                            )
                            .map_err(to_io)?
                            .check()
                            .map_err(to_io)?;
                        window.bounds = monitor.bounds;
                        window.last_input_region = None;
                    }
                    windows.push(window);
                }
                for (_, window) in existing {
                    destroy_overlay_window(&self.conn, window);
                }
                self.windows = windows;
                self.bounds = monitor_union(&monitors);
                self.topology_changed |= prior != self.monitor_bounds();
                self.conn.flush().map_err(to_io)
            }

            fn is_protected_window(&self, window: Window) -> io::Result<bool> {
                let class = self.string_property(
                    window,
                    AtomEnum::WM_CLASS.into(),
                    AtomEnum::STRING.into(),
                )?;
                let title = self
                    .string_property(window, self.atoms._NET_WM_NAME, self.atoms.UTF8_STRING)
                    .or_else(|_| {
                        self.string_property(
                            window,
                            AtomEnum::WM_NAME.into(),
                            AtomEnum::STRING.into(),
                        )
                    })?;
                Ok(super::super::is_protected_terminal_app(
                    class.as_deref(),
                    title.as_deref(),
                ))
            }

            fn string_property(
                &self,
                window: Window,
                property: u32,
                ty: u32,
            ) -> io::Result<Option<String>> {
                let reply = self
                    .conn
                    .get_property(false, window, property, ty, 0, 1024)
                    .map_err(to_io)?
                    .reply()
                    .map_err(to_io)?;
                Ok(property_string(reply))
            }
        }

        fn property_string(reply: GetPropertyReply) -> Option<String> {
            if reply.value.is_empty() {
                return None;
            }
            let value = reply
                .value
                .split(|byte| *byte == 0)
                .rfind(|part| !part.is_empty())
                .unwrap_or(&reply.value);
            Some(String::from_utf8_lossy(value).into_owned())
        }

        fn query_monitors(conn: &RustConnection, root: u32, screen: &Screen) -> Vec<Monitor> {
            if let Ok(cookie) = conn.randr_get_monitors(root, true) {
                if let Ok(reply) = cookie.reply() {
                    let mut monitors = reply
                        .monitors
                        .into_iter()
                        .filter(|monitor| monitor.width > 0 && monitor.height > 0)
                        .map(|monitor| Monitor {
                            id: monitor.name,
                            bounds: Rect::new(
                                Vec2::new(monitor.x as f32, monitor.y as f32),
                                Vec2::new(
                                    monitor.x as f32 + monitor.width as f32,
                                    monitor.y as f32 + monitor.height as f32,
                                ),
                            ),
                            primary: monitor.primary,
                        })
                        .collect::<Vec<_>>();
                    monitors.sort_by_key(|monitor| !monitor.primary);
                    if !monitors.is_empty() {
                        return monitors;
                    }
                }
            }

            if conn
                .xinerama_is_active()
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .is_some_and(|reply| reply.state != 0)
            {
                if let Ok(cookie) = conn.xinerama_query_screens() {
                    if let Ok(reply) = cookie.reply() {
                        let monitors = reply
                            .screen_info
                            .into_iter()
                            .enumerate()
                            .map(|(index, info)| Monitor {
                                id: 0x8000_0000 | index as u32,
                                bounds: Rect::new(
                                    Vec2::new(info.x_org as f32, info.y_org as f32),
                                    Vec2::new(
                                        info.x_org as f32 + info.width as f32,
                                        info.y_org as f32 + info.height as f32,
                                    ),
                                ),
                                primary: index == 0,
                            })
                            .collect::<Vec<_>>();
                        if !monitors.is_empty() {
                            return monitors;
                        }
                    }
                }
            }

            let bounds = conn
                .get_geometry(root)
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|geometry| {
                    Rect::new(
                        Vec2::ZERO,
                        Vec2::new(geometry.width as f32, geometry.height as f32),
                    )
                })
                .unwrap_or_else(|| {
                    Rect::new(
                        Vec2::ZERO,
                        Vec2::new(
                            screen.width_in_pixels as f32,
                            screen.height_in_pixels as f32,
                        ),
                    )
                });
            vec![Monitor {
                id: root,
                bounds,
                primary: true,
            }]
        }

        fn monitor_union(monitors: &[Monitor]) -> Rect {
            monitors
                .iter()
                .map(|monitor| monitor.bounds)
                .reduce(Rect::union)
                .unwrap_or_else(|| Rect::new(Vec2::ZERO, Vec2::new(1.0, 1.0)))
        }

        #[derive(Clone, Copy)]
        struct ChosenVisual {
            depth: u8,
            visual: Visualid,
        }

        fn choose_argb_visual(conn: &RustConnection, screen_num: usize) -> Option<ChosenVisual> {
            let Ok(cookie) = conn.render_query_pict_formats() else {
                return None;
            };
            let Ok(reply) = cookie.reply() else {
                return None;
            };
            let screen = reply.screens.get(screen_num)?;
            for depth in &screen.depths {
                if depth.depth != 32 {
                    continue;
                }
                for visual in &depth.visuals {
                    let Some(format) = reply
                        .formats
                        .iter()
                        .find(|format| format.id == visual.format)
                    else {
                        continue;
                    };
                    if format.depth == 32 && format.direct.alpha_mask != 0 {
                        return Some(ChosenVisual {
                            depth: depth.depth,
                            visual: visual.visual,
                        });
                    }
                }
            }
            None
        }

        fn compositor_running(conn: &RustConnection, screen_num: usize) -> io::Result<bool> {
            let atom_name = format!("_NET_WM_CM_S{screen_num}");
            let atom = conn
                .intern_atom(false, atom_name.as_bytes())
                .map_err(to_io)?
                .reply()
                .map_err(to_io)?
                .atom;
            let owner = conn
                .get_selection_owner(atom)
                .map_err(to_io)?
                .reply()
                .map_err(to_io)?
                .owner;
            Ok(owner != NONE)
        }

        fn initialize_input_shape_extensions(conn: &RustConnection) -> io::Result<()> {
            // XFixes keeps a negotiated version per client. Region requests issued before
            // QueryVersion are rejected with BadRequest by Xorg/Xvfb even when the extension is
            // installed, which previously made the fail-closed pre-map input-shape check reject
            // every otherwise-valid overlay.
            let xfixes = conn
                .xfixes_query_version(5, 0)
                .map_err(to_io)?
                .reply()
                .map_err(to_io)?;
            if !xfixes_supports_regions(xfixes.major_version) {
                return Err(io::Error::other(format!(
                    "X11 overlay requires XFixes region support (server reported {}.{})",
                    xfixes.major_version, xfixes.minor_version
                )));
            }
            conn.shape_query_version()
                .map_err(to_io)?
                .reply()
                .map_err(to_io)?;
            Ok(())
        }

        fn create_overlay_window(
            conn: &RustConnection,
            root: u32,
            root_visual: Visualid,
            atoms: &Atoms,
            visual: ChosenVisual,
            monitor: Monitor,
        ) -> io::Result<X11Window> {
            let window = conn.generate_id().map_err(to_io)?;
            let gc = conn.generate_id().map_err(to_io)?;
            let colormap = if visual.visual != root_visual {
                let colormap = conn.generate_id().map_err(to_io)?;
                conn.create_colormap(ColormapAlloc::NONE, colormap, root, visual.visual)
                    .map_err(to_io)?
                    .check()
                    .map_err(to_io)?;
                Some(colormap)
            } else {
                None
            };
            let mut aux = CreateWindowAux::new()
                .override_redirect(1)
                .background_pixel(0)
                .border_pixel(0)
                .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY);
            if let Some(colormap) = colormap {
                aux = aux.colormap(colormap);
            }
            conn.create_window(
                visual.depth,
                window,
                root,
                clamp_i16(monitor.bounds.min.x),
                clamp_i16(monitor.bounds.min.y),
                clamp_u16(monitor.bounds.width()),
                clamp_u16(monitor.bounds.height()),
                0,
                WindowClass::INPUT_OUTPUT,
                visual.visual,
                &aux,
            )
            .map_err(to_io)?
            .check()
            .map_err(to_io)?;
            conn.create_gc(gc, window, &CreateGCAux::new())
                .map_err(to_io)?
                .check()
                .map_err(to_io)?;
            conn.change_property8(
                PropMode::REPLACE,
                window,
                AtomEnum::WM_NAME,
                AtomEnum::STRING,
                b"honk300 overlay",
            )
            .map_err(to_io)?
            .check()
            .map_err(to_io)?;
            conn.change_property8(
                PropMode::REPLACE,
                window,
                atoms._NET_WM_NAME,
                atoms.UTF8_STRING,
                b"honk300 overlay",
            )
            .map_err(to_io)?
            .check()
            .map_err(to_io)?;
            conn.change_property32(
                PropMode::REPLACE,
                window,
                atoms._NET_WM_WINDOW_TYPE,
                AtomEnum::ATOM,
                &[atoms._NET_WM_WINDOW_TYPE_DOCK],
            )
            .map_err(to_io)?
            .check()
            .map_err(to_io)?;
            conn.change_property32(
                PropMode::REPLACE,
                window,
                atoms._NET_WM_STATE,
                AtomEnum::ATOM,
                &[atoms._NET_WM_STATE_ABOVE],
            )
            .map_err(to_io)?
            .check()
            .map_err(to_io)?;

            // An empty input region must be accepted before the window can ever become visible.
            // This prevents a transparent, input-blocking overlay when XFixes/XShape is absent.
            apply_input_shape(conn, window, monitor.bounds, None)?;
            if !x11_mapping_allowed(true, true, true) {
                return Err(io::Error::other("X11 overlay safety prerequisites failed"));
            }
            conn.map_window(window)
                .map_err(to_io)?
                .check()
                .map_err(to_io)?;
            conn.configure_window(
                window,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )
            .map_err(to_io)?
            .check()
            .map_err(to_io)?;
            Ok(X11Window {
                monitor_id: monitor.id,
                window,
                gc,
                depth: visual.depth,
                colormap,
                bounds: monitor.bounds,
                last_input_region: Some(None),
            })
        }

        fn apply_input_shape(
            conn: &RustConnection,
            window: u32,
            monitor_bounds: Rect,
            effective: Option<Rect>,
        ) -> io::Result<()> {
            let region = conn.generate_id().map_err(to_io)?;
            let rectangles = effective
                .map(|rect| {
                    vec![Rectangle {
                        x: clamp_i16(rect.min.x - monitor_bounds.min.x),
                        y: clamp_i16(rect.min.y - monitor_bounds.min.y),
                        width: clamp_u16(rect.width()),
                        height: clamp_u16(rect.height()),
                    }]
                })
                .unwrap_or_default();
            conn.xfixes_create_region(region, &rectangles)
                .map_err(to_io)?
                .check()
                .map_err(to_io)?;
            conn.xfixes_set_window_shape_region(window, shape::SK::INPUT, 0, 0, region)
                .map_err(to_io)?
                .check()
                .map_err(to_io)?;
            conn.xfixes_destroy_region(region)
                .map_err(to_io)?
                .check()
                .map_err(to_io)
        }

        fn clipped_bgra(
            dirty: Rect,
            clip: Rect,
            pixmap: &Pixmap,
            width: u16,
            height: u16,
        ) -> Vec<u8> {
            let src = x11_bgra_from_rgba(pixmap);
            let src_stride = pixmap.width() as usize * 4;
            let dst_stride = width as usize * 4;
            let src_x = (clip.min.x - dirty.min.x).round().max(0.0) as usize;
            let src_y = (clip.min.y - dirty.min.y).round().max(0.0) as usize;
            let mut out = vec![0; dst_stride * height as usize];
            for row in 0..height as usize {
                let start = (src_y + row) * src_stride + src_x * 4;
                let end = start.saturating_add(dst_stride).min(src.len());
                if end > start {
                    let len = end - start;
                    out[row * dst_stride..row * dst_stride + len].copy_from_slice(&src[start..end]);
                }
            }
            out
        }

        fn destroy_overlay_window(conn: &RustConnection, window: X11Window) {
            let _ = conn.free_gc(window.gc);
            let _ = conn.destroy_window(window.window);
            if let Some(colormap) = window.colormap {
                let _ = conn.free_colormap(colormap);
            }
        }

        fn to_io(err: impl std::fmt::Display) -> io::Error {
            io::Error::other(err.to_string())
        }

        impl Drop for X11Overlay {
            fn drop(&mut self) {
                for window in std::mem::take(&mut self.windows) {
                    destroy_overlay_window(&self.conn, window);
                }
                let _ = self.conn.flush();
            }
        }
    }

    mod wayland {
        use super::super::{
            wayland_scale_ceil, wayland_scale_floor, wayland_scaled_dimension,
            WAYLAND_MAX_BUFFERS_PER_OUTPUT,
        };
        use super::x11_bgra_from_rgba;
        use honk_engine::tiny_skia::Pixmap;
        use honk_engine::{Rect, Vec2};
        use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState, Region};
        use smithay_client_toolkit::delegate_compositor;
        use smithay_client_toolkit::delegate_layer;
        use smithay_client_toolkit::delegate_output;
        use smithay_client_toolkit::delegate_registry;
        use smithay_client_toolkit::delegate_shm;
        use smithay_client_toolkit::output::{OutputHandler, OutputInfo, OutputState};
        use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
        use smithay_client_toolkit::registry_handlers;
        use smithay_client_toolkit::shell::wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        };
        use smithay_client_toolkit::shell::WaylandSurface;
        use smithay_client_toolkit::shm::{
            slot::{Buffer, SlotPool},
            Shm, ShmHandler,
        };
        use std::collections::VecDeque;
        use std::io;
        use std::os::fd::AsRawFd;
        use wayland_client::globals::registry_queue_init;
        use wayland_client::protocol::{wl_output, wl_shm, wl_surface};
        use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
        use wayland_protocols::wp::fractional_scale::v1::client::{
            wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
            wp_fractional_scale_v1::{self, WpFractionalScaleV1},
        };
        use wayland_protocols::wp::viewporter::client::{
            wp_viewport::WpViewport, wp_viewporter::WpViewporter,
        };

        const DEFAULT_WIDTH: u32 = 1280;
        const DEFAULT_HEIGHT: u32 = 720;

        pub struct WaylandOverlay {
            conn: Connection,
            event_queue: EventQueue<WaylandLayer>,
            state: WaylandLayer,
        }

        impl WaylandOverlay {
            pub fn new() -> io::Result<Self> {
                let conn = Connection::connect_to_env().map_err(to_io)?;
                let (globals, event_queue) = registry_queue_init(&conn).map_err(to_io)?;
                let qh = event_queue.handle();
                let compositor = CompositorState::bind(&globals, &qh).map_err(to_io)?;
                let layer_shell = LayerShell::bind(&globals, &qh).map_err(to_io)?;
                let shm = Shm::bind(&globals, &qh).map_err(to_io)?;
                let viewporter = globals.bind::<WpViewporter, _, _>(&qh, 1..=1, ()).ok();
                let fractional_scale_manager = globals
                    .bind::<WpFractionalScaleManagerV1, _, _>(&qh, 1..=1, ())
                    .ok();
                let mut state = WaylandLayer {
                    registry_state: RegistryState::new(&globals),
                    output_state: OutputState::new(&globals, &qh),
                    shm,
                    compositor,
                    layer_shell,
                    viewporter,
                    fractional_scale_manager,
                    surfaces: Vec::new(),
                    topology_changed: false,
                    fatal_error: None,
                };
                let mut event_queue = event_queue;
                event_queue.roundtrip(&mut state).map_err(to_io)?;
                state.sync_outputs(&qh)?;
                if state.surfaces.is_empty() {
                    return Err(io::Error::other(
                        "Wayland compositor advertised no active outputs",
                    ));
                }
                for _ in 0..16 {
                    event_queue.blocking_dispatch(&mut state).map_err(to_io)?;
                    if state.surfaces.iter().all(|surface| surface.configured) {
                        break;
                    }
                }
                if let Some(err) = state.fatal_error.take() {
                    return Err(io::Error::other(err));
                }
                Ok(Self {
                    conn,
                    event_queue,
                    state,
                })
            }

            pub fn bounds(&self) -> Rect {
                self.state.bounds()
            }

            pub fn monitor_bounds(&self) -> Vec<Rect> {
                self.state
                    .surfaces
                    .iter()
                    .filter(|surface| !surface.closed)
                    .map(OutputSurface::bounds)
                    .collect()
            }

            pub fn take_topology_changed(&mut self) -> bool {
                std::mem::take(&mut self.state.topology_changed)
            }

            pub fn set_input_region(&mut self, rect: Option<Rect>) -> io::Result<()> {
                if let Some(_rect) = rect {
                    // Native Wayland reduced mode intentionally remains click-through; the
                    // compositor still controls global input and pointer grabs.
                }
                Ok(())
            }

            pub fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
                self.state.present(dirty, pixmap)?;
                self.conn.flush().map_err(to_io)
            }

            pub fn pump(&mut self) -> io::Result<bool> {
                self.conn.flush().map_err(to_io)?;
                self.event_queue
                    .dispatch_pending(&mut self.state)
                    .map_err(to_io)?;
                if let Some(guard) = self.event_queue.prepare_read() {
                    let mut descriptor = libc::pollfd {
                        fd: guard.connection_fd().as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    let ready = unsafe { libc::poll(&mut descriptor, 1, 0) };
                    if ready < 0 {
                        return Err(io::Error::last_os_error());
                    }
                    if descriptor.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                        drop(guard);
                        return Ok(false);
                    }
                    if ready > 0 && descriptor.revents & libc::POLLIN != 0 {
                        guard.read().map_err(to_io)?;
                    } else {
                        drop(guard);
                    }
                }
                self.event_queue
                    .dispatch_pending(&mut self.state)
                    .map_err(to_io)?;
                if let Some(err) = self.state.fatal_error.take() {
                    return Err(io::Error::other(err));
                }
                Ok(self.state.surfaces.iter().any(|surface| !surface.closed))
            }
        }

        struct WaylandLayer {
            registry_state: RegistryState,
            output_state: OutputState,
            shm: Shm,
            compositor: CompositorState,
            layer_shell: LayerShell,
            viewporter: Option<WpViewporter>,
            fractional_scale_manager: Option<WpFractionalScaleManagerV1>,
            surfaces: Vec<OutputSurface>,
            topology_changed: bool,
            fatal_error: Option<String>,
        }

        struct OutputSurface {
            output: wl_output::WlOutput,
            layer: LayerSurface,
            _input_region: Option<Region>,
            viewport: Option<WpViewport>,
            _fractional_scale: Option<WpFractionalScaleV1>,
            pool: SlotPool,
            buffers: VecDeque<Buffer>,
            position: (i32, i32),
            width: u32,
            height: u32,
            integer_scale: i32,
            preferred_scale_120: Option<u32>,
            configured: bool,
            closed: bool,
        }

        #[derive(Debug, Clone)]
        struct FractionalScaleData {
            output: wl_output::WlOutput,
        }

        #[derive(Clone, Copy)]
        struct ScaleGlobals<'a> {
            viewporter: Option<&'a WpViewporter>,
            fractional_scale_manager: Option<&'a WpFractionalScaleManagerV1>,
        }

        impl WaylandLayer {
            fn sync_outputs(&mut self, qh: &QueueHandle<Self>) -> io::Result<()> {
                let before = self.bounds_list();
                let outputs = self.output_state.outputs().collect::<Vec<_>>();
                self.surfaces
                    .retain(|surface| outputs.iter().any(|output| output == &surface.output));
                for output in &outputs {
                    if self
                        .surfaces
                        .iter()
                        .all(|surface| &surface.output != output)
                    {
                        self.surfaces.push(OutputSurface::new(
                            output.clone(),
                            &self.compositor,
                            &self.layer_shell,
                            &self.shm,
                            ScaleGlobals {
                                viewporter: self.viewporter.as_ref(),
                                fractional_scale_manager: self.fractional_scale_manager.as_ref(),
                            },
                            qh,
                            self.surfaces.len(),
                        )?);
                    }
                }
                for (index, surface) in self.surfaces.iter_mut().enumerate() {
                    if let Some(info) = self.output_state.info(&surface.output) {
                        surface.update_info(&info, index);
                    }
                }
                self.topology_changed |= before != self.bounds_list();
                Ok(())
            }

            fn bounds_list(&self) -> Vec<Rect> {
                self.surfaces
                    .iter()
                    .filter(|surface| !surface.closed)
                    .map(OutputSurface::bounds)
                    .collect()
            }

            fn bounds(&self) -> Rect {
                self.bounds_list()
                    .into_iter()
                    .reduce(Rect::union)
                    .unwrap_or_else(|| {
                        Rect::new(
                            Vec2::ZERO,
                            Vec2::new(DEFAULT_WIDTH as f32, DEFAULT_HEIGHT as f32),
                        )
                    })
            }

            fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
                for surface in &mut self.surfaces {
                    if !surface.closed && surface.configured {
                        surface.present(dirty, pixmap)?;
                    }
                }
                Ok(())
            }
        }

        impl OutputSurface {
            fn new(
                output: wl_output::WlOutput,
                compositor: &CompositorState,
                layer_shell: &LayerShell,
                shm: &Shm,
                scale_globals: ScaleGlobals<'_>,
                qh: &QueueHandle<WaylandLayer>,
                index: usize,
            ) -> io::Result<Self> {
                let surface = compositor.create_surface(qh);
                let input_region = Region::new(compositor).map_err(to_io)?;
                // The region is deliberately left empty: native Wayland reduced mode is always
                // click-through and never accepts pointer or keyboard input.
                surface.set_input_region(Some(input_region.wl_region()));
                let layer = layer_shell.create_layer_surface(
                    qh,
                    surface,
                    Layer::Top,
                    Some("honk300"),
                    Some(&output),
                );
                layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
                layer.set_keyboard_interactivity(KeyboardInteractivity::None);
                layer.set_exclusive_zone(-1);
                layer.set_size(0, 0);
                let (viewport, fractional_scale) = match (
                    scale_globals.viewporter,
                    scale_globals.fractional_scale_manager,
                ) {
                    (Some(viewporter), Some(manager)) => (
                        Some(viewporter.get_viewport(layer.wl_surface(), qh, ())),
                        Some(manager.get_fractional_scale(
                            layer.wl_surface(),
                            qh,
                            FractionalScaleData {
                                output: output.clone(),
                            },
                        )),
                    ),
                    _ => (None, None),
                };
                layer.commit();
                let pool = SlotPool::new((DEFAULT_WIDTH * DEFAULT_HEIGHT * 4) as usize, shm)
                    .map_err(to_io)?;
                Ok(Self {
                    output,
                    layer,
                    _input_region: Some(input_region),
                    viewport,
                    _fractional_scale: fractional_scale,
                    pool,
                    buffers: VecDeque::new(),
                    position: ((index as i32) * DEFAULT_WIDTH as i32, 0),
                    width: DEFAULT_WIDTH,
                    height: DEFAULT_HEIGHT,
                    integer_scale: 1,
                    preferred_scale_120: None,
                    configured: false,
                    closed: false,
                })
            }

            fn update_info(&mut self, info: &OutputInfo, index: usize) {
                self.integer_scale = info.scale_factor.max(1);
                self.position = info
                    .logical_position
                    .unwrap_or(((index as i32) * self.width as i32, 0));
                let fallback_size = info.modes.iter().find(|mode| mode.current).map(|mode| {
                    (
                        (mode.dimensions.0 / self.integer_scale).max(1),
                        (mode.dimensions.1 / self.integer_scale).max(1),
                    )
                });
                if let Some((width, height)) = info.logical_size.or(fallback_size) {
                    self.width = width.max(1) as u32;
                    self.height = height.max(1) as u32;
                }
                self.apply_surface_scale();
            }

            fn effective_scale_120(&self) -> u32 {
                self.preferred_scale_120
                    .unwrap_or_else(|| self.integer_scale.max(1) as u32 * 120)
                    .max(1)
            }

            fn apply_surface_scale(&self) {
                if let Some(viewport) = &self.viewport {
                    self.layer.wl_surface().set_buffer_scale(1);
                    viewport.set_destination(self.width as i32, self.height as i32);
                } else {
                    self.layer
                        .wl_surface()
                        .set_buffer_scale(self.integer_scale.max(1));
                }
            }

            fn bounds(&self) -> Rect {
                Rect::new(
                    Vec2::new(self.position.0 as f32, self.position.1 as f32),
                    Vec2::new(
                        self.position.0 as f32 + self.width as f32,
                        self.position.1 as f32 + self.height as f32,
                    ),
                )
            }

            fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
                let Some(clip) = dirty.intersection(self.bounds()).map(Rect::pixel_aligned) else {
                    return Ok(());
                };
                let scale_120 = self.effective_scale_120();
                let surface_bounds = self.bounds();
                let buffer_width = wayland_scaled_dimension(self.width.max(1), scale_120);
                let buffer_height = wayland_scaled_dimension(self.height.max(1), scale_120);
                let stride = buffer_width as i32 * 4;
                let surface = self.layer.wl_surface().clone();

                // Retain at most three real wl_buffers. Released buffers are redrawn in place;
                // active buffers are never dropped merely to satisfy the VecDeque length, since
                // doing so would hide compositor-owned slots from the apparent pool bound.
                let mut index = 0;
                while index < self.buffers.len() {
                    let dimensions_match = self.buffers[index].height() == buffer_height as i32
                        && self.buffers[index].stride() == stride;
                    let released = self.buffers[index].canvas(&mut self.pool).is_some();
                    if released && !dimensions_match {
                        self.buffers.remove(index);
                    } else {
                        index += 1;
                    }
                }

                let released = (0..self.buffers.len()).find(|&index| {
                    self.buffers[index].height() == buffer_height as i32
                        && self.buffers[index].stride() == stride
                        && self.buffers[index].canvas(&mut self.pool).is_some()
                });
                let buffer_index = if let Some(index) = released {
                    index
                } else if self.buffers.len() < WAYLAND_MAX_BUFFERS_PER_OUTPUT {
                    let (buffer, _canvas) = self
                        .pool
                        .create_buffer(
                            buffer_width as i32,
                            buffer_height as i32,
                            stride,
                            wl_shm::Format::Argb8888,
                        )
                        .map_err(to_io)?;
                    self.buffers.push_back(buffer);
                    self.buffers.len() - 1
                } else {
                    // All three buffers are still owned by the compositor. Dropping this frame
                    // is preferable to allocating without a bound or blocking the event pump.
                    return Ok(());
                };

                if let Some(canvas) = self.buffers[buffer_index].canvas(&mut self.pool) {
                    canvas.fill(0);
                    blit_to_canvas(
                        canvas,
                        buffer_width,
                        buffer_height,
                        scale_120,
                        surface_bounds,
                        dirty,
                        clip,
                        pixmap,
                    );
                }
                self.layer.wl_surface().damage_buffer(
                    0,
                    0,
                    buffer_width as i32,
                    buffer_height as i32,
                );
                self.buffers[buffer_index]
                    .attach_to(&surface)
                    .map_err(to_io)?;
                self.layer.commit();
                Ok(())
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn blit_to_canvas(
            canvas: &mut [u8],
            width: u32,
            height: u32,
            scale_120: u32,
            surface_bounds: Rect,
            dirty: Rect,
            clip: Rect,
            pixmap: &Pixmap,
        ) {
            let src = x11_bgra_from_rgba(pixmap);
            let src_x = (clip.min.x - dirty.min.x).round().max(0.0) as u32;
            let src_y = (clip.min.y - dirty.min.y).round().max(0.0) as u32;
            let dst_x = (clip.min.x - surface_bounds.min.x).round().max(0.0) as u32;
            let dst_y = (clip.min.y - surface_bounds.min.y).round().max(0.0) as u32;
            let logical_width = clip.width().ceil().max(0.0) as u32;
            let logical_height = clip.height().ceil().max(0.0) as u32;
            for y in 0..logical_height {
                for x in 0..logical_width {
                    let source_x = src_x + x;
                    let source_y = src_y + y;
                    if source_x >= pixmap.width() || source_y >= pixmap.height() {
                        continue;
                    }
                    let src_idx = ((source_y * pixmap.width() + source_x) * 4) as usize;
                    let target_x0 = wayland_scale_floor(dst_x + x, scale_120);
                    let target_x1 = wayland_scale_ceil(dst_x + x + 1, scale_120);
                    let target_y0 = wayland_scale_floor(dst_y + y, scale_120);
                    let target_y1 = wayland_scale_ceil(dst_y + y + 1, scale_120);
                    for target_y in target_y0..target_y1 {
                        for target_x in target_x0..target_x1 {
                            if target_x >= width || target_y >= height {
                                continue;
                            }
                            let dst_idx = ((target_y * width + target_x) * 4) as usize;
                            canvas[dst_idx..dst_idx + 4]
                                .copy_from_slice(&src[src_idx..src_idx + 4]);
                        }
                    }
                }
            }
        }

        impl CompositorHandler for WaylandLayer {
            fn scale_factor_changed(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                surface: &wl_surface::WlSurface,
                new_factor: i32,
            ) {
                if let Some(output) = self
                    .surfaces
                    .iter_mut()
                    .find(|output| output.layer.wl_surface() == surface)
                {
                    output.integer_scale = new_factor.max(1);
                    output.apply_surface_scale();
                    self.topology_changed = true;
                }
            }

            fn transform_changed(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _surface: &wl_surface::WlSurface,
                _new_transform: wl_output::Transform,
            ) {
            }

            fn frame(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _surface: &wl_surface::WlSurface,
                _time: u32,
            ) {
            }

            fn surface_enter(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _surface: &wl_surface::WlSurface,
                _output: &wl_output::WlOutput,
            ) {
            }

            fn surface_leave(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _surface: &wl_surface::WlSurface,
                _output: &wl_output::WlOutput,
            ) {
            }
        }

        impl OutputHandler for WaylandLayer {
            fn output_state(&mut self) -> &mut OutputState {
                &mut self.output_state
            }

            fn new_output(
                &mut self,
                _conn: &Connection,
                qh: &QueueHandle<Self>,
                _output: wl_output::WlOutput,
            ) {
                if let Err(err) = self.sync_outputs(qh) {
                    self.fatal_error = Some(err.to_string());
                }
            }

            fn update_output(
                &mut self,
                _conn: &Connection,
                qh: &QueueHandle<Self>,
                _output: wl_output::WlOutput,
            ) {
                if let Err(err) = self.sync_outputs(qh) {
                    self.fatal_error = Some(err.to_string());
                }
            }

            fn output_destroyed(
                &mut self,
                _conn: &Connection,
                qh: &QueueHandle<Self>,
                _output: wl_output::WlOutput,
            ) {
                if let Err(err) = self.sync_outputs(qh) {
                    self.fatal_error = Some(err.to_string());
                }
            }
        }

        impl LayerShellHandler for WaylandLayer {
            fn closed(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                layer: &LayerSurface,
            ) {
                if let Some(surface) = self
                    .surfaces
                    .iter_mut()
                    .find(|surface| &surface.layer == layer)
                {
                    surface.closed = true;
                    self.topology_changed = true;
                }
            }

            fn configure(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                layer: &LayerSurface,
                configure: LayerSurfaceConfigure,
                _serial: u32,
            ) {
                if let Some(surface) = self
                    .surfaces
                    .iter_mut()
                    .find(|surface| &surface.layer == layer)
                {
                    if configure.new_size.0 > 0 {
                        surface.width = configure.new_size.0;
                    }
                    if configure.new_size.1 > 0 {
                        surface.height = configure.new_size.1;
                    }
                    surface.apply_surface_scale();
                    surface.configured = true;
                    surface.closed = false;
                    self.topology_changed = true;
                }
            }
        }

        impl ShmHandler for WaylandLayer {
            fn shm_state(&mut self) -> &mut Shm {
                &mut self.shm
            }
        }

        impl Dispatch<WpFractionalScaleV1, FractionalScaleData> for WaylandLayer {
            fn event(
                state: &mut Self,
                _proxy: &WpFractionalScaleV1,
                event: wp_fractional_scale_v1::Event,
                data: &FractionalScaleData,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
            ) {
                if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event {
                    if let Some(surface) = state
                        .surfaces
                        .iter_mut()
                        .find(|surface| surface.output == data.output)
                    {
                        surface.preferred_scale_120 = Some(scale.max(1));
                        surface.apply_surface_scale();
                        state.topology_changed = true;
                    }
                }
            }
        }

        delegate_compositor!(WaylandLayer);
        delegate_output!(WaylandLayer);
        delegate_shm!(WaylandLayer);
        delegate_layer!(WaylandLayer);
        delegate_registry!(WaylandLayer);
        wayland_client::delegate_noop!(WaylandLayer: ignore WpViewporter);
        wayland_client::delegate_noop!(WaylandLayer: ignore WpViewport);
        wayland_client::delegate_noop!(WaylandLayer: ignore WpFractionalScaleManagerV1);

        impl ProvidesRegistryState for WaylandLayer {
            fn registry(&mut self) -> &mut RegistryState {
                &mut self.registry_state
            }
            registry_handlers![OutputState];
        }

        fn to_io(err: impl std::fmt::Display) -> io::Error {
            io::Error::other(err.to_string())
        }
    }
}

pub fn is_protected_terminal_app(wm_class: Option<&str>, app_name: Option<&str>) -> bool {
    wm_class
        .into_iter()
        .chain(app_name)
        .flat_map(|value| {
            value
                .split(['.', '-', '_', ' ', ':', ';', ','])
                .filter(|part| !part.is_empty())
        })
        .map(normalize_token)
        .any(|token| {
            matches!(
                token.as_str(),
                "terminal"
                    | "xterm"
                    | "uxterm"
                    | "rxvt"
                    | "urxvt"
                    | "alacritty"
                    | "kitty"
                    | "foot"
                    | "ghostty"
                    | "wezterm"
                    | "konsole"
                    | "kgx"
                    | "tilix"
                    | "terminator"
                    | "lxterminal"
                    | "qterminal"
                    | "blackbox"
                    | "ptyxis"
                    | "rio"
            )
        })
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .and_then(|value| non_empty(Some(value.as_str())).map(str::to_string))
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(unix)]
mod imp {
    use super::LocalTime;

    #[allow(deprecated)]
    pub fn local_time() -> LocalTime {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs() as libc::time_t)
            .unwrap_or(0);
        let mut out = std::mem::MaybeUninit::<libc::tm>::zeroed();
        let ok = unsafe { !libc::localtime_r(&now, out.as_mut_ptr()).is_null() };
        if !ok {
            return fallback_time();
        }
        let time = unsafe { out.assume_init() };
        let year = time.tm_year + 1900;
        let month = time.tm_mon + 1;
        let day = time.tm_mday;
        LocalTime {
            day: year * 10_000 + month * 100 + day,
            hour: time.tm_hour as u8,
            minute: time.tm_min as u8,
            second: time.tm_sec as u8,
        }
    }

    fn fallback_time() -> LocalTime {
        LocalTime {
            day: 19700101,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

#[cfg(not(unix))]
mod imp {
    use super::LocalTime;

    pub fn local_time() -> LocalTime {
        LocalTime {
            day: 19700101,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x11_requires_one_overlay_per_monitor() {
        let monitor_count = 3;
        assert_eq!(x11_overlay_count(monitor_count), monitor_count);
    }

    #[test]
    fn x11_fails_closed_without_an_argb_visual() {
        assert!(!x11_mapping_allowed(false, true, true));
        assert!(!x11_mapping_allowed(true, false, true));
        assert!(!x11_mapping_allowed(true, true, false));
        assert!(x11_mapping_allowed(true, true, true));
    }

    #[test]
    fn x11_requires_negotiated_xfixes_region_support() {
        assert!(!xfixes_supports_regions(1));
        assert!(xfixes_supports_regions(2));
        assert!(xfixes_supports_regions(6));
    }

    #[test]
    fn wayland_buffer_budget_is_capped_per_output() {
        assert_eq!(wayland_retained_buffer_count(4), 3);
    }

    #[test]
    fn wayland_fractional_scaling_rounds_outward_without_zero_sized_buffers() {
        assert_eq!(wayland_scaled_dimension(100, 180), 150);
        assert_eq!(wayland_scale_floor(1, 180), 1);
        assert_eq!(wayland_scale_ceil(1, 180), 2);
        assert_eq!(wayland_scaled_dimension(0, 180), 1);
    }

    #[test]
    fn x11_empty_input_shape_is_verified_before_map() {
        assert_eq!(x11_setup_steps(), ["shape", "map"]);
    }

    #[test]
    fn x11_randr_change_triggers_topology_reconciliation() {
        assert!(x11_event_requires_reconcile(true));
        assert!(!x11_event_requires_reconcile(false));
    }

    #[test]
    fn wayland_owns_one_surface_per_output() {
        let active_outputs = 3;
        assert_eq!(wayland_surface_count(active_outputs), active_outputs);
    }

    #[test]
    fn wayland_pump_reads_socket_before_dispatching() {
        assert_eq!(
            wayland_pump_steps(),
            ["prepare_read", "poll", "read", "dispatch_pending"]
        );
    }

    #[test]
    fn x11_is_default_when_display_is_available_even_inside_wayland_session() {
        assert_eq!(
            detect_display_server(Some("wayland"), Some(":0"), Some("wayland-0"), false),
            DisplayServer::X11
        );
    }

    #[test]
    fn forced_wayland_overrides_xwayland_display() {
        assert_eq!(
            detect_display_server(Some("wayland"), Some(":0"), Some("wayland-0"), true),
            DisplayServer::Wayland
        );
    }

    #[test]
    fn wayland_is_used_when_no_x11_display_exists() {
        assert_eq!(
            detect_display_server(Some("wayland"), None, Some("wayland-1"), false),
            DisplayServer::Wayland
        );
    }

    #[test]
    fn unknown_session_remains_unknown_without_display_env() {
        assert_eq!(
            detect_display_server(Some("tty"), None, None, false),
            DisplayServer::Unknown
        );
    }

    #[test]
    fn default_bounds_are_positive_and_stable() {
        let bounds = default_world_bounds(DisplayServer::Wayland);
        assert_eq!(bounds.min, Vec2::new(0.0, 0.0));
        assert_eq!(bounds.max, Vec2::new(1280.0, 720.0));
    }

    #[test]
    fn display_capabilities_match_x11_first_reduced_wayland_contract() {
        assert!(display_cursor_mischief_supported(DisplayServer::X11));
        assert!(display_foreign_window_watch_supported(DisplayServer::X11));
        assert!(!display_collect_window_supported(DisplayServer::X11));

        assert!(!display_cursor_mischief_supported(DisplayServer::Wayland));
        assert!(!display_foreign_window_watch_supported(
            DisplayServer::Wayland
        ));
        assert!(!display_collect_window_supported(DisplayServer::Wayland));
    }

    #[test]
    fn local_time_returns_valid_calendar_shape() {
        let time = local_time();
        let year = time.day / 10_000;
        let month = (time.day / 100) % 100;
        let day = time.day % 100;
        assert!(year >= 1970);
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        assert!(time.hour < 24);
        assert!(time.minute < 60);
        assert!(time.second < 61);
    }

    #[test]
    fn terminal_app_classifier_covers_common_linux_terminals() {
        for (class, name) in [
            (Some("Alacritty"), None),
            (Some("org.gnome.Terminal"), Some("Terminal")),
            (Some("kitty"), Some("kitty")),
            (Some("org.kde.konsole"), Some("Konsole")),
            (Some("com.mitchellh.ghostty"), Some("Ghostty")),
            (Some("wezterm"), Some("WezTerm")),
            (Some("xfce4-terminal"), Some("Terminal")),
            (Some("org.gnome.Ptyxis"), Some("Ptyxis")),
        ] {
            assert!(
                is_protected_terminal_app(class, name),
                "{class:?} {name:?} should be protected"
            );
        }
    }

    #[test]
    fn terminal_app_classifier_does_not_block_regular_apps() {
        for (class, name) in [
            (Some("firefox"), Some("Firefox")),
            (Some("org.gnome.Nautilus"), Some("Files")),
            (Some("code"), Some("Visual Studio Code")),
        ] {
            assert!(
                !is_protected_terminal_app(class, name),
                "{class:?} {name:?} should not be protected"
            );
        }
    }
}
