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
                    Err(err) => {
                        eprintln!(
                            "honk300: X11 overlay unavailable; falling back headless ({err})"
                        );
                        OverlayInner::Headless(HeadlessOverlay::new(DisplayServer::X11))
                    }
                },
                DisplayServer::Wayland => match wayland::WaylandOverlay::new() {
                    Ok(overlay) => OverlayInner::Wayland(overlay),
                    Err(err) => {
                        eprintln!(
                            "honk300: Wayland layer-shell overlay unavailable; falling back headless ({err})"
                        );
                        OverlayInner::Headless(HeadlessOverlay::new(DisplayServer::Wayland))
                    }
                },
                DisplayServer::Unknown => OverlayInner::Headless(HeadlessOverlay::new(preferred)),
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
        use super::{clamp_i16, clamp_u16, x11_bgra_from_rgba};
        use honk_engine::tiny_skia::Pixmap;
        use honk_engine::{ForeignWindowId, ForeignWindowSnapshot, Pointer, Rect, Vec2};
        use std::io;
        use x11rb::connection::Connection;
        use x11rb::protocol::render::ConnectionExt as RenderConnectionExt;
        use x11rb::protocol::shape;
        use x11rb::protocol::xfixes::ConnectionExt as XFixesConnectionExt;
        use x11rb::protocol::xinerama::ConnectionExt as XineramaConnectionExt;
        use x11rb::protocol::xproto::{
            AtomEnum, ButtonMask, ChangeWindowAttributesAux, ColormapAlloc, ConfigureWindowAux,
            ConnectionExt as XprotoConnectionExt, CreateGCAux, CreateWindowAux, EventMask,
            GetPropertyReply, ImageFormat, PropMode, Rectangle, StackMode, Visualid, Window,
            WindowClass,
        };
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
            window: u32,
            gc: u32,
            depth: u8,
            colormap: Option<u32>,
            bounds: Rect,
            atoms: Atoms,
        }

        impl X11Overlay {
            pub fn new() -> io::Result<Self> {
                let (conn, screen_num) = x11rb::connect(None).map_err(to_io)?;
                let screen = &conn.setup().roots[screen_num];
                let root = screen.root;
                let bounds = query_bounds(&conn, root).unwrap_or_else(|| {
                    Rect::new(
                        Vec2::ZERO,
                        Vec2::new(
                            screen.width_in_pixels as f32,
                            screen.height_in_pixels as f32,
                        ),
                    )
                });
                let width = clamp_u16(bounds.width());
                let height = clamp_u16(bounds.height());
                let window = conn.generate_id().map_err(to_io)?;
                let gc = conn.generate_id().map_err(to_io)?;
                let atoms = Atoms::new(&conn).map_err(to_io)?.reply().map_err(to_io)?;
                let visual =
                    choose_argb_visual(&conn, screen_num, screen.root_depth, screen.root_visual);
                let colormap = if visual.visual != screen.root_visual {
                    let colormap = conn.generate_id().map_err(to_io)?;
                    conn.create_colormap(ColormapAlloc::NONE, colormap, root, visual.visual)
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
                    clamp_i16(bounds.min.x),
                    clamp_i16(bounds.min.y),
                    width,
                    height,
                    0,
                    WindowClass::INPUT_OUTPUT,
                    visual.visual,
                    &aux,
                )
                .map_err(to_io)?;
                conn.create_gc(gc, window, &CreateGCAux::new())
                    .map_err(to_io)?;
                conn.change_property8(
                    PropMode::REPLACE,
                    window,
                    AtomEnum::WM_NAME,
                    AtomEnum::STRING,
                    b"honk300 overlay",
                )
                .map_err(to_io)?;
                conn.change_property8(
                    PropMode::REPLACE,
                    window,
                    atoms._NET_WM_NAME,
                    atoms.UTF8_STRING,
                    b"honk300 overlay",
                )
                .map_err(to_io)?;
                conn.change_property32(
                    PropMode::REPLACE,
                    window,
                    atoms._NET_WM_WINDOW_TYPE,
                    AtomEnum::ATOM,
                    &[atoms._NET_WM_WINDOW_TYPE_DOCK],
                )
                .map_err(to_io)?;
                conn.change_property32(
                    PropMode::REPLACE,
                    window,
                    atoms._NET_WM_STATE,
                    AtomEnum::ATOM,
                    &[atoms._NET_WM_STATE_ABOVE],
                )
                .map_err(to_io)?;
                conn.map_window(window).map_err(to_io)?;
                conn.configure_window(
                    window,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                )
                .map_err(to_io)?;
                conn.flush().map_err(to_io)?;

                Ok(Self {
                    depth: visual.depth,
                    conn,
                    root,
                    window,
                    gc,
                    colormap,
                    bounds,
                    atoms,
                })
            }

            pub fn bounds(&self) -> Rect {
                self.bounds
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
                let region = self.conn.generate_id().map_err(to_io)?;
                let rectangles = rect
                    .and_then(|rect| rect.intersection(self.bounds))
                    .map(|rect| {
                        vec![Rectangle {
                            x: clamp_i16(rect.min.x - self.bounds.min.x),
                            y: clamp_i16(rect.min.y - self.bounds.min.y),
                            width: clamp_u16(rect.width()),
                            height: clamp_u16(rect.height()),
                        }]
                    })
                    .unwrap_or_default();
                self.conn
                    .xfixes_create_region(region, &rectangles)
                    .map_err(to_io)?;
                self.conn
                    .xfixes_set_window_shape_region(self.window, shape::SK::INPUT, 0, 0, region)
                    .map_err(to_io)?;
                self.conn.xfixes_destroy_region(region).map_err(to_io)?;
                self.conn.flush().map_err(to_io)
            }

            pub fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
                if pixmap.width() == 0 || pixmap.height() == 0 {
                    return Ok(());
                }
                let data = x11_bgra_from_rgba(pixmap);
                self.conn
                    .put_image(
                        ImageFormat::Z_PIXMAP,
                        self.window,
                        self.gc,
                        pixmap.width() as u16,
                        pixmap.height() as u16,
                        clamp_i16(dirty.min.x - self.bounds.min.x),
                        clamp_i16(dirty.min.y - self.bounds.min.y),
                        0,
                        self.depth,
                        &data,
                    )
                    .map_err(to_io)?;
                self.conn.flush().map_err(to_io)
            }

            pub fn pump(&mut self) -> io::Result<bool> {
                while self.conn.poll_for_event().map_err(to_io)?.is_some() {}
                let _ = self.conn.change_window_attributes(
                    self.window,
                    &ChangeWindowAttributesAux::new().event_mask(EventMask::EXPOSURE),
                );
                Ok(true)
            }

            fn foreign_target_window(&self, focus: Window) -> io::Result<Option<Window>> {
                if focus == NONE || focus == self.root || focus == self.window {
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
                .filter(|part| !part.is_empty())
                .next_back()
                .unwrap_or(&reply.value);
            Some(String::from_utf8_lossy(value).into_owned())
        }

        fn query_bounds(conn: &RustConnection, root: u32) -> Option<Rect> {
            let active = conn.xinerama_is_active().ok()?.reply().ok()?.state != 0;
            if !active {
                return None;
            }
            let screens = conn
                .xinerama_query_screens()
                .ok()?
                .reply()
                .ok()?
                .screen_info;
            screens
                .into_iter()
                .map(|screen| {
                    Rect::new(
                        Vec2::new(screen.x_org as f32, screen.y_org as f32),
                        Vec2::new(
                            screen.x_org as f32 + screen.width as f32,
                            screen.y_org as f32 + screen.height as f32,
                        ),
                    )
                })
                .reduce(Rect::union)
                .or_else(|| {
                    conn.get_geometry(root).ok()?.reply().ok().map(|geometry| {
                        Rect::new(
                            Vec2::ZERO,
                            Vec2::new(geometry.width as f32, geometry.height as f32),
                        )
                    })
                })
        }

        #[derive(Clone, Copy)]
        struct ChosenVisual {
            depth: u8,
            visual: Visualid,
        }

        fn choose_argb_visual(
            conn: &RustConnection,
            screen_num: usize,
            fallback_depth: u8,
            fallback_visual: Visualid,
        ) -> ChosenVisual {
            let Ok(cookie) = conn.render_query_pict_formats() else {
                return ChosenVisual {
                    depth: fallback_depth,
                    visual: fallback_visual,
                };
            };
            let Ok(reply) = cookie.reply() else {
                return ChosenVisual {
                    depth: fallback_depth,
                    visual: fallback_visual,
                };
            };
            let Some(screen) = reply.screens.get(screen_num) else {
                return ChosenVisual {
                    depth: fallback_depth,
                    visual: fallback_visual,
                };
            };
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
                        return ChosenVisual {
                            depth: depth.depth,
                            visual: visual.visual,
                        };
                    }
                }
            }
            ChosenVisual {
                depth: fallback_depth,
                visual: fallback_visual,
            }
        }

        fn to_io(err: impl std::fmt::Display) -> io::Error {
            io::Error::other(err.to_string())
        }

        impl Drop for X11Overlay {
            fn drop(&mut self) {
                let _ = self.conn.free_gc(self.gc);
                let _ = self.conn.destroy_window(self.window);
                if let Some(colormap) = self.colormap {
                    let _ = self.conn.free_colormap(colormap);
                }
                let _ = self.conn.flush();
            }
        }
    }

    mod wayland {
        use super::x11_bgra_from_rgba;
        use honk_engine::tiny_skia::Pixmap;
        use honk_engine::{Rect, Vec2};
        use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState, Region};
        use smithay_client_toolkit::delegate_compositor;
        use smithay_client_toolkit::delegate_layer;
        use smithay_client_toolkit::delegate_output;
        use smithay_client_toolkit::delegate_registry;
        use smithay_client_toolkit::delegate_shm;
        use smithay_client_toolkit::output::{OutputHandler, OutputState};
        use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
        use smithay_client_toolkit::registry_handlers;
        use smithay_client_toolkit::shell::wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        };
        use smithay_client_toolkit::shell::WaylandSurface;
        use smithay_client_toolkit::shm::{slot::SlotPool, Shm, ShmHandler};
        use std::io;
        use std::num::NonZeroU32;
        use wayland_client::globals::registry_queue_init;
        use wayland_client::protocol::{wl_output, wl_shm, wl_surface};
        use wayland_client::{Connection, EventQueue, QueueHandle};

        const DEFAULT_WIDTH: u32 = 1280;
        const DEFAULT_HEIGHT: u32 = 720;

        pub struct WaylandOverlay {
            conn: Connection,
            event_queue: EventQueue<WaylandLayer>,
            qh: QueueHandle<WaylandLayer>,
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
                let surface = compositor.create_surface(&qh);
                let input_region = Region::new(&compositor).ok();
                if let Some(region) = &input_region {
                    surface.set_input_region(Some(region.wl_region()));
                }
                let layer = layer_shell.create_layer_surface(
                    &qh,
                    surface,
                    Layer::Top,
                    Some("honk300"),
                    None,
                );
                layer.set_anchor(Anchor::TOP | Anchor::LEFT);
                layer.set_keyboard_interactivity(KeyboardInteractivity::None);
                layer.set_size(DEFAULT_WIDTH, DEFAULT_HEIGHT);
                layer.commit();
                let pool = SlotPool::new((DEFAULT_WIDTH * DEFAULT_HEIGHT * 4) as usize, &shm)
                    .map_err(to_io)?;
                let mut state = WaylandLayer {
                    registry_state: RegistryState::new(&globals),
                    output_state: OutputState::new(&globals, &qh),
                    shm,
                    pool,
                    width: DEFAULT_WIDTH,
                    height: DEFAULT_HEIGHT,
                    configured: false,
                    closed: false,
                    layer,
                    _input_region: input_region,
                };
                let mut event_queue = event_queue;
                for _ in 0..16 {
                    event_queue.blocking_dispatch(&mut state).map_err(to_io)?;
                    if state.configured {
                        break;
                    }
                }
                Ok(Self {
                    conn,
                    event_queue,
                    qh,
                    state,
                })
            }

            pub fn bounds(&self) -> Rect {
                Rect::new(
                    Vec2::ZERO,
                    Vec2::new(self.state.width as f32, self.state.height as f32),
                )
            }

            pub fn set_input_region(&mut self, rect: Option<Rect>) -> io::Result<()> {
                if let Some(_rect) = rect {
                    // Native Wayland reduced mode intentionally remains click-through; the
                    // compositor still controls global input and pointer grabs.
                }
                Ok(())
            }

            pub fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
                self.state.present(&self.qh, dirty, pixmap)?;
                self.conn.flush().map_err(to_io)
            }

            pub fn pump(&mut self) -> io::Result<bool> {
                self.event_queue
                    .dispatch_pending(&mut self.state)
                    .map_err(to_io)?;
                self.conn.flush().map_err(to_io)?;
                Ok(!self.state.closed)
            }
        }

        struct WaylandLayer {
            registry_state: RegistryState,
            output_state: OutputState,
            shm: Shm,
            pool: SlotPool,
            width: u32,
            height: u32,
            configured: bool,
            closed: bool,
            layer: LayerSurface,
            _input_region: Option<Region>,
        }

        impl WaylandLayer {
            fn present(
                &mut self,
                _qh: &QueueHandle<Self>,
                dirty: Rect,
                pixmap: &Pixmap,
            ) -> io::Result<()> {
                let width = self.width.max(1);
                let height = self.height.max(1);
                let stride = width as i32 * 4;
                let (buffer, canvas) = self
                    .pool
                    .create_buffer(
                        width as i32,
                        height as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    )
                    .map_err(to_io)?;
                canvas.fill(0);
                blit_to_canvas(canvas, width, height, dirty, pixmap);
                self.layer
                    .wl_surface()
                    .damage_buffer(0, 0, width as i32, height as i32);
                buffer.attach_to(self.layer.wl_surface()).map_err(to_io)?;
                self.layer.commit();
                Ok(())
            }
        }

        fn blit_to_canvas(
            canvas: &mut [u8],
            width: u32,
            height: u32,
            dirty: Rect,
            pixmap: &Pixmap,
        ) {
            let src = x11_bgra_from_rgba(pixmap);
            let dst_x = dirty.min.x.round().max(0.0) as u32;
            let dst_y = dirty.min.y.round().max(0.0) as u32;
            for y in 0..pixmap.height() {
                let target_y = dst_y + y;
                if target_y >= height {
                    break;
                }
                for x in 0..pixmap.width() {
                    let target_x = dst_x + x;
                    if target_x >= width {
                        break;
                    }
                    let src_idx = ((y * pixmap.width() + x) * 4) as usize;
                    let dst_idx = ((target_y * width + target_x) * 4) as usize;
                    canvas[dst_idx..dst_idx + 4].copy_from_slice(&src[src_idx..src_idx + 4]);
                }
            }
        }

        impl CompositorHandler for WaylandLayer {
            fn scale_factor_changed(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _surface: &wl_surface::WlSurface,
                _new_factor: i32,
            ) {
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
                _qh: &QueueHandle<Self>,
                _output: wl_output::WlOutput,
            ) {
            }

            fn update_output(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _output: wl_output::WlOutput,
            ) {
            }

            fn output_destroyed(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _output: wl_output::WlOutput,
            ) {
            }
        }

        impl LayerShellHandler for WaylandLayer {
            fn closed(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _layer: &LayerSurface,
            ) {
                self.closed = true;
            }

            fn configure(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _layer: &LayerSurface,
                configure: LayerSurfaceConfigure,
                _serial: u32,
            ) {
                self.width = NonZeroU32::new(configure.new_size.0)
                    .map(NonZeroU32::get)
                    .unwrap_or(DEFAULT_WIDTH);
                self.height = NonZeroU32::new(configure.new_size.1)
                    .map(NonZeroU32::get)
                    .unwrap_or(DEFAULT_HEIGHT);
                self.configured = true;
            }
        }

        impl ShmHandler for WaylandLayer {
            fn shm_state(&mut self) -> &mut Shm {
                &mut self.shm
            }
        }

        delegate_compositor!(WaylandLayer);
        delegate_output!(WaylandLayer);
        delegate_shm!(WaylandLayer);
        delegate_layer!(WaylandLayer);
        delegate_registry!(WaylandLayer);

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
