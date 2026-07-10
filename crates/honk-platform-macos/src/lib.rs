//! macOS platform backend for honk300.
//!
//! The engine stays platform-free. This crate owns the AppKit agent/overlay identity,
//! CoreGraphics display and pointer primitives, and permission-gated desktop behavior.

use honk_engine::{Rect, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppKitFrame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn appkit_frame_for_world_rect(rect: Rect, desktop: Rect) -> AppKitFrame {
    AppKitFrame {
        x: rect.min.x as f64,
        y: (desktop.max.y - rect.max.y) as f64,
        width: rect.width().max(0.0) as f64,
        height: rect.height().max(0.0) as f64,
    }
}

pub fn appkit_point_to_world(point: (f64, f64), desktop: Rect) -> Vec2 {
    Vec2::new(point.0 as f32, (desktop.max.y as f64 - point.1) as f32)
}

/// AppKit's global Y axis is anchored to the main display, not the union of all displays.
pub fn appkit_coordinate_space(main_display: Rect, _virtual_desktop: Rect) -> Rect {
    main_display
}

#[cfg(any(test, target_os = "macos"))]
#[derive(Debug, Default)]
struct DragClassifier {
    candidate: Option<(u64, Vec2)>,
    active: Option<u64>,
}

#[cfg(any(test, target_os = "macos"))]
impl DragClassifier {
    const MIN_MOVEMENT_SQUARED: f32 = 4.0;

    fn release(&mut self) {
        self.candidate = None;
        self.active = None;
    }

    fn observe(&mut self, window_id: u64, origin: Vec2) -> bool {
        if self.active == Some(window_id) {
            return true;
        }
        match self.candidate {
            Some((candidate_id, start)) if candidate_id == window_id => {
                let delta = origin - start;
                if delta.x * delta.x + delta.y * delta.y >= Self::MIN_MOVEMENT_SQUARED {
                    self.active = Some(window_id);
                    true
                } else {
                    false
                }
            }
            _ => {
                self.candidate = Some((window_id, origin));
                self.active = None;
                false
            }
        }
    }
}

#[cfg(test)]
fn reconciled_display_ids(_existing: &[u32], active: &[u32]) -> Vec<u32> {
    active.to_vec()
}

pub fn is_protected_terminal_app(bundle_id: Option<&str>, app_name: Option<&str>) -> bool {
    let bundle_match = bundle_id
        .map(|id| id.to_ascii_lowercase())
        .is_some_and(|id| {
            matches!(
                id.as_str(),
                "com.apple.terminal"
                    | "com.googlecode.iterm2"
                    | "org.alacritty"
                    | "net.kovidgoyal.kitty"
                    | "dev.warp.warp-stable"
                    | "dev.warp.warp"
                    | "com.mitchellh.ghostty"
                    | "co.zeit.hyper"
            )
        });
    if bundle_match {
        return true;
    }

    app_name
        .map(|name| name.to_ascii_lowercase())
        .is_some_and(|name| {
            matches!(
                name.as_str(),
                "terminal"
                    | "iterm"
                    | "iterm2"
                    | "alacritty"
                    | "kitty"
                    | "wezterm"
                    | "warp"
                    | "ghostty"
                    | "hyper"
            )
        })
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{
        appkit_coordinate_space, appkit_frame_for_world_rect, appkit_point_to_world,
        is_protected_terminal_app, AppKitFrame, DragClassifier,
    };
    use honk_engine::collect_window::{
        CollectWindowId, CollectWindowKind, CollectWindowRequestId, CollectWindowSnapshot,
    };
    use honk_engine::{
        ForeignWindowId, ForeignWindowSnapshot, LocalTime, PresenceSnapshot, Rect, Vec2,
    };
    use objc2::rc::Retained;
    use objc2::MainThreadMarker;
    use objc2::{AnyThread, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSBitmapFormat,
        NSBitmapImageRep, NSColor, NSDeviceRGBColorSpace, NSEvent, NSEventMask, NSImage,
        NSImageRep, NSImageView, NSRunningApplication, NSScreenSaverWindowLevel, NSTextField,
        NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
    };
    use objc2_application_services::{
        AXError, AXIsProcessTrusted, AXUIElement, AXValue, AXValueType,
    };
    use objc2_core_foundation::{CFRetained, CFString, CFType, CGPoint, CGRect, CGSize};
    use objc2_core_graphics::{
        CGDisplayBounds, CGError, CGEventSourceStateID, CGGetActiveDisplayList, CGMainDisplayID,
        CGMouseButton, CGWarpMouseCursorPosition,
    };
    use objc2_foundation::{
        NSDate, NSDefaultRunLoopMode, NSInteger, NSPoint, NSRect, NSSize, NSString,
    };
    use std::collections::HashMap;
    use std::ffi::c_void;
    use std::io;
    use std::ptr;
    use std::ptr::NonNull;
    use std::time::{Duration, Instant};
    use tiny_skia::Pixmap;

    const MAX_DISPLAYS: usize = 16;
    const DISPLAY_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AccessibilityState {
        Trusted,
        Denied,
    }

    pub fn accessibility_state() -> AccessibilityState {
        if unsafe { AXIsProcessTrusted() } {
            AccessibilityState::Trusted
        } else {
            AccessibilityState::Denied
        }
    }

    pub fn pointer_state() -> (f32, f32, bool) {
        let coordinate_space = display_list()
            .map(|displays| {
                appkit_coordinate_space(primary_bounds(&displays), union_bounds(&displays))
            })
            .unwrap_or_else(|_| default_desktop_bounds());
        let point = NSEvent::mouseLocation();
        let world = appkit_point_to_world((point.x, point.y), coordinate_space);
        let left_down = objc2_core_graphics::CGEventSource::button_state(
            CGEventSourceStateID::CombinedSessionState,
            CGMouseButton::Left,
        );
        (world.x, world.y, left_down)
    }

    pub fn local_time() -> LocalTime {
        unsafe {
            let mut now = libc::time(ptr::null_mut());
            let mut out = std::mem::zeroed::<libc::tm>();
            let tm = if libc::localtime_r(&mut now, &mut out).is_null() {
                None
            } else {
                Some(out)
            };
            if let Some(tm) = tm {
                LocalTime {
                    day: ((tm.tm_year + 1900) * 10_000) + ((tm.tm_mon + 1) * 100) + tm.tm_mday,
                    hour: tm.tm_hour as u8,
                    minute: tm.tm_min as u8,
                    second: tm.tm_sec as u8,
                }
            } else {
                LocalTime {
                    day: 19700101,
                    hour: 0,
                    minute: 0,
                    second: 0,
                }
            }
        }
    }

    pub fn presence_state() -> io::Result<PresenceSnapshot> {
        Ok(PresenceSnapshot::unsupported())
    }

    pub fn warp_cursor(pos: Vec2) -> io::Result<()> {
        if accessibility_state() != AccessibilityState::Trusted {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "macOS Accessibility permission is required to warp the cursor",
            ));
        }
        let err = CGWarpMouseCursorPosition(CGPoint {
            x: pos.x as f64,
            y: pos.y as f64,
        });
        if err == CGError::Success {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "CGWarpMouseCursorPosition failed with {}",
                err.0
            )))
        }
    }

    pub struct Overlay {
        app: Retained<NSApplication>,
        displays: Vec<DisplayWindow>,
        primary_bounds: Rect,
        virtual_bounds: Rect,
        topology_changed: bool,
        next_display_refresh: Instant,
    }

    impl Overlay {
        pub fn new() -> io::Result<Self> {
            let mtm = MainThreadMarker::new().ok_or_else(|| {
                io::Error::other("macOS AppKit overlay must be created on the main thread")
            })?;
            let app = NSApplication::sharedApplication(mtm);
            let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
            app.finishLaunching();

            let display_infos = display_list()?;
            let primary_bounds = primary_bounds(&display_infos);
            let virtual_bounds = union_bounds(&display_infos);
            let coordinate_space = appkit_coordinate_space(primary_bounds, virtual_bounds);
            let mut displays = Vec::with_capacity(display_infos.len());
            for info in display_infos {
                displays.push(DisplayWindow::new(mtm, info, coordinate_space)?);
            }
            Ok(Self {
                app,
                displays,
                primary_bounds,
                virtual_bounds,
                topology_changed: false,
                next_display_refresh: Instant::now() + DISPLAY_REFRESH_INTERVAL,
            })
        }

        pub fn pump(&mut self) -> bool {
            // Non-blocking drain of the AppKit event queue. Without this the queue is never
            // serviced, so window-chrome events — including the Titled|Closable collect-window
            // close button — are silently ignored. `distantPast` makes each poll return
            // immediately, so we drain everything currently queued and stop as soon as it's empty.
            let mode = unsafe { NSDefaultRunLoopMode };
            while let Some(event) = self.app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                Some(&NSDate::distantPast()),
                mode,
                true,
            ) {
                self.app.sendEvent(&event);
            }
            self.app.updateWindows();
            if Instant::now() >= self.next_display_refresh {
                self.next_display_refresh = Instant::now() + DISPLAY_REFRESH_INTERVAL;
                if let Err(err) = self.refresh_display_topology() {
                    eprintln!(
                        "honk300: macOS display refresh failed; keeping prior topology ({err})"
                    );
                }
            }
            true
        }

        pub fn primary_monitor_bounds(&self) -> Rect {
            self.primary_bounds
        }

        pub fn virtual_desktop_bounds(&self) -> Rect {
            self.virtual_bounds
        }

        pub fn monitor_bounds(&self) -> Vec<Rect> {
            self.displays
                .iter()
                .map(|display| display.info.bounds)
                .collect()
        }

        pub fn take_topology_changed(&mut self) -> bool {
            std::mem::take(&mut self.topology_changed)
        }

        pub fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
            let coordinate_space =
                appkit_coordinate_space(self.primary_bounds, self.virtual_bounds);
            for display in &mut self.displays {
                if let Some(clip) = dirty.intersection(display.info.bounds) {
                    display.present(dirty, clip, pixmap, coordinate_space)?;
                } else {
                    display.clear();
                }
            }
            Ok(())
        }

        pub fn set_interactive(&mut self, over_goose: bool) {
            for display in &mut self.displays {
                display.window.setIgnoresMouseEvents(!over_goose);
            }
        }

        fn refresh_display_topology(&mut self) -> io::Result<()> {
            let infos = display_list()?;
            if infos
                == self
                    .displays
                    .iter()
                    .map(|display| display.info)
                    .collect::<Vec<_>>()
            {
                return Ok(());
            }

            let mtm = MainThreadMarker::new().ok_or_else(|| {
                io::Error::other("macOS display refresh must run on the main thread")
            })?;
            let primary_bounds = primary_bounds(&infos);
            let virtual_bounds = union_bounds(&infos);
            let coordinate_space = appkit_coordinate_space(primary_bounds, virtual_bounds);
            let mut existing = self
                .displays
                .drain(..)
                .map(|display| (display.info.id, display))
                .collect::<HashMap<_, _>>();
            let mut displays = Vec::with_capacity(infos.len());
            for info in infos {
                let mut display = match existing.remove(&info.id) {
                    Some(display) => display,
                    None => DisplayWindow::new(mtm, info, coordinate_space)?,
                };
                display.update_info(info, coordinate_space);
                displays.push(display);
            }
            // Removed displays are the only entries left. Their Drop impl closes just those
            // AppKit windows; surviving display windows retain identity and pixel ownership.
            drop(existing);
            self.displays = displays;
            self.primary_bounds = primary_bounds;
            self.virtual_bounds = virtual_bounds;
            self.topology_changed = true;
            Ok(())
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct DisplayInfo {
        id: u32,
        bounds: Rect,
        primary: bool,
    }

    struct DisplayWindow {
        info: DisplayInfo,
        window: Retained<NSWindow>,
        image_view: Retained<NSImageView>,
        image: Option<Retained<NSImage>>,
        buffer: Vec<u8>,
    }

    impl DisplayWindow {
        fn new(mtm: MainThreadMarker, info: DisplayInfo, desktop: Rect) -> io::Result<Self> {
            let frame = appkit_frame_for_world_rect(info.bounds, desktop);
            let ns_frame = nsrect(frame);
            let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    ns_frame,
                    style,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };
            window.setOpaque(false);
            window.setBackgroundColor(Some(&NSColor::clearColor()));
            window.setHasShadow(false);
            unsafe {
                window.setReleasedWhenClosed(false);
            }
            window.setCanHide(false);
            window.setIgnoresMouseEvents(true);
            window.setLevel(NSScreenSaverWindowLevel);
            window.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::Transient
                    | NSWindowCollectionBehavior::IgnoresCycle
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            );

            let view_frame = nsrect(AppKitFrame {
                x: 0.0,
                y: 0.0,
                width: frame.width,
                height: frame.height,
            });
            let image_view = NSImageView::initWithFrame(NSImageView::alloc(mtm), view_frame);
            window.setContentView(Some(&image_view));
            window.orderFrontRegardless();

            Ok(Self {
                info,
                window,
                image_view,
                image: None,
                buffer: Vec::new(),
            })
        }

        fn present(
            &mut self,
            dirty: Rect,
            clip: Rect,
            pixmap: &Pixmap,
            desktop: Rect,
        ) -> io::Result<()> {
            let clip = clip.pixel_aligned();
            let width = clip.width().ceil().max(1.0) as u32;
            let height = clip.height().ceil().max(1.0) as u32;
            self.buffer = clipped_bgra(dirty, clip, pixmap, width, height);
            let image = image_from_bgra(&self.buffer, width, height)?;
            let local_frame = AppKitFrame {
                x: (clip.min.x - self.info.bounds.min.x) as f64,
                y: (self.info.bounds.max.y - clip.max.y) as f64,
                width: width as f64,
                height: height as f64,
            };
            self.image_view.setFrame(nsrect(local_frame));
            self.image_view.setImage(Some(&image));
            self.image = Some(image);

            let window_frame = appkit_frame_for_world_rect(self.info.bounds, desktop);
            self.window.setFrame_display(nsrect(window_frame), false);
            self.window.orderFrontRegardless();
            Ok(())
        }

        fn clear(&mut self) {
            self.image_view.setImage(None);
            self.image = None;
            self.buffer.clear();
        }

        fn update_info(&mut self, info: DisplayInfo, coordinate_space: Rect) {
            self.info = info;
            let frame = appkit_frame_for_world_rect(info.bounds, coordinate_space);
            self.window.setFrame_display(nsrect(frame), false);
        }
    }

    impl Drop for DisplayWindow {
        fn drop(&mut self) {
            self.window.orderOut(None);
            self.window.close();
        }
    }

    fn display_list() -> io::Result<Vec<DisplayInfo>> {
        let mut ids = [0u32; MAX_DISPLAYS];
        let mut count = 0u32;
        let err =
            unsafe { CGGetActiveDisplayList(MAX_DISPLAYS as u32, ids.as_mut_ptr(), &mut count) };
        if err != CGError::Success {
            return Err(io::Error::other(format!(
                "CGGetActiveDisplayList failed with {}",
                err.0
            )));
        }
        let primary = CGMainDisplayID();
        let mut displays = ids
            .iter()
            .copied()
            .take(count as usize)
            .map(|id| DisplayInfo {
                id,
                bounds: cg_rect_to_world(CGDisplayBounds(id)),
                primary: id == primary,
            })
            .collect::<Vec<_>>();
        if displays.is_empty() {
            displays.push(DisplayInfo {
                id: primary,
                bounds: default_desktop_bounds(),
                primary: true,
            });
        }
        Ok(displays)
    }

    fn primary_bounds(displays: &[DisplayInfo]) -> Rect {
        displays
            .iter()
            .find(|display| display.primary)
            .or_else(|| displays.first())
            .map(|display| display.bounds)
            .unwrap_or_else(default_desktop_bounds)
    }

    fn union_bounds(displays: &[DisplayInfo]) -> Rect {
        displays
            .iter()
            .map(|display| display.bounds)
            .reduce(Rect::union)
            .unwrap_or_else(default_desktop_bounds)
    }

    fn default_desktop_bounds() -> Rect {
        Rect::new(Vec2::new(0.0, 0.0), Vec2::new(1440.0, 900.0))
    }

    fn cg_rect_to_world(rect: CGRect) -> Rect {
        Rect::new(
            Vec2::new(rect.origin.x as f32, rect.origin.y as f32),
            Vec2::new(
                (rect.origin.x + rect.size.width) as f32,
                (rect.origin.y + rect.size.height) as f32,
            ),
        )
    }

    fn nsrect(frame: AppKitFrame) -> NSRect {
        NSRect {
            origin: NSPoint {
                x: frame.x,
                y: frame.y,
            },
            size: NSSize {
                width: frame.width,
                height: frame.height,
            },
        }
    }

    fn image_from_bgra(buffer: &[u8], width: u32, height: u32) -> io::Result<Retained<NSImage>> {
        // Pass NULL planes so AppKit allocates and owns the pixel storage for the rep's lifetime.
        // The previous code handed AppKit a pointer into an external per-frame Vec that the caller
        // reuses and reallocates, so the rep aliased memory that mutated or freed underneath it
        // (tearing / use-after-free). With AppKit-owned storage we copy our rows in once.
        let rep = unsafe {
            NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bitmapFormat_bytesPerRow_bitsPerPixel(
                    NSBitmapImageRep::alloc(),
                    ptr::null_mut(),
                    width as NSInteger,
                    height as NSInteger,
                    8,
                    4,
                    true,
                    false,
                    NSDeviceRGBColorSpace,
                    NSBitmapFormat::AlphaFirst | NSBitmapFormat::ThirtyTwoBitLittleEndian,
                    (width * 4) as NSInteger,
                    32,
                )
        }
        .ok_or_else(|| io::Error::other("failed to create NSBitmapImageRep"))?;

        // Copy each row into AppKit's buffer, honoring its (possibly padded) bytesPerRow rather
        // than assuming a tight width*4 stride.
        let dst = rep.bitmapData();
        if dst.is_null() {
            return Err(io::Error::other("NSBitmapImageRep provided no bitmap data"));
        }
        let dst_stride = rep.bytesPerRow() as usize;
        let src_stride = width as usize * 4;
        let row_bytes = src_stride.min(dst_stride);
        for row in 0..height as usize {
            unsafe {
                ptr::copy_nonoverlapping(
                    buffer.as_ptr().add(row * src_stride),
                    dst.add(row * dst_stride),
                    row_bytes,
                );
            }
        }

        let image = NSImage::initWithSize(
            NSImage::alloc(),
            NSSize {
                width: width as f64,
                height: height as f64,
            },
        );
        let rep_ref: &NSImageRep = &rep;
        image.addRepresentation(rep_ref);
        Ok(image)
    }

    fn clipped_bgra(dirty: Rect, clip: Rect, pixmap: &Pixmap, width: u32, height: u32) -> Vec<u8> {
        let src_width = pixmap.width() as usize;
        let src_x = (clip.min.x - dirty.min.x).round().max(0.0) as usize;
        let src_y = (clip.min.y - dirty.min.y).round().max(0.0) as usize;
        let mut out = vec![0; width as usize * height as usize * 4];
        let src = pixmap.data();
        for y in 0..height as usize {
            for x in 0..width as usize {
                let src_idx = ((src_y + y) * src_width + src_x + x) * 4;
                let dst_idx = (y * width as usize + x) * 4;
                if src_idx + 3 < src.len() {
                    out[dst_idx] = src[src_idx + 2];
                    out[dst_idx + 1] = src[src_idx + 1];
                    out[dst_idx + 2] = src[src_idx];
                    out[dst_idx + 3] = src[src_idx + 3];
                }
            }
        }
        out
    }

    pub struct ForeignWindowWatcher {
        system: CFRetained<AXUIElement>,
        self_pid: libc::pid_t,
        drag: DragClassifier,
    }

    impl ForeignWindowWatcher {
        pub fn new(_overlay: &Overlay) -> io::Result<Self> {
            if accessibility_state() == AccessibilityState::Trusted {
                let system = unsafe { AXUIElement::new_system_wide() };
                let _ = unsafe { system.set_messaging_timeout(0.05) };
                Ok(Self {
                    system,
                    self_pid: std::process::id() as libc::pid_t,
                    drag: DragClassifier::default(),
                })
            } else {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "macOS Accessibility permission is required for foreign-window watch",
                ))
            }
        }

        pub fn active_drag(&mut self) -> io::Result<Option<ForeignWindowSnapshot>> {
            let left_down = objc2_core_graphics::CGEventSource::button_state(
                CGEventSourceStateID::CombinedSessionState,
                CGMouseButton::Left,
            );
            if !left_down {
                self.drag.release();
                return Ok(None);
            }

            let Some(window) = copy_ax_attribute(&self.system, "AXFocusedWindow")? else {
                return Ok(None);
            };
            let Ok(window) = window.downcast::<AXUIElement>() else {
                return Ok(None);
            };

            let Some(pid) = ax_pid(&window)? else {
                return Ok(None);
            };
            if pid == self.self_pid || protected_running_application(pid) {
                return Ok(None);
            }

            let Some(position) = copy_ax_attribute(&window, "AXPosition")? else {
                return Ok(None);
            };
            let Some(size) = copy_ax_attribute(&window, "AXSize")? else {
                return Ok(None);
            };
            let Some(origin) = ax_point(&position) else {
                return Ok(None);
            };
            let Some(size) = ax_size(&size) else {
                return Ok(None);
            };
            if size.width <= 1.0 || size.height <= 1.0 {
                return Ok(None);
            }

            let rect = Rect::new(
                Vec2::new(origin.x as f32, origin.y as f32),
                Vec2::new(
                    (origin.x + size.width) as f32,
                    (origin.y + size.height) as f32,
                ),
            );
            if !self.drag.observe(pid as u64, rect.min) {
                return Ok(None);
            }
            Ok(Some(ForeignWindowSnapshot::top_center(
                ForeignWindowId(pid as u64),
                rect,
            )))
        }
    }

    fn copy_ax_attribute(
        element: &AXUIElement,
        attribute: &'static str,
    ) -> io::Result<Option<CFRetained<CFType>>> {
        let name = CFString::from_static_str(attribute);
        let mut raw: *const CFType = ptr::null();
        let slot = NonNull::new(&mut raw as *mut *const CFType)
            .ok_or_else(|| io::Error::other("failed to allocate AX attribute slot"))?;
        let err = unsafe { element.copy_attribute_value(&name, slot) };
        match err {
            AXError::Success => {
                let Some(raw) = NonNull::new(raw.cast_mut()) else {
                    return Ok(None);
                };
                Ok(Some(unsafe { CFRetained::from_raw(raw) }))
            }
            AXError::NoValue | AXError::AttributeUnsupported => Ok(None),
            AXError::APIDisabled => Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "macOS Accessibility API is disabled",
            )),
            other => Err(io::Error::other(format!(
                "AX attribute {attribute} failed with {}",
                other.0
            ))),
        }
    }

    fn ax_pid(element: &AXUIElement) -> io::Result<Option<libc::pid_t>> {
        let mut pid: libc::pid_t = 0;
        let slot = NonNull::new(&mut pid as *mut libc::pid_t)
            .ok_or_else(|| io::Error::other("failed to allocate AX pid slot"))?;
        let err = unsafe { element.pid(slot) };
        match err {
            AXError::Success => Ok(Some(pid)),
            AXError::NoValue | AXError::InvalidUIElement => Ok(None),
            AXError::APIDisabled => Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "macOS Accessibility API is disabled",
            )),
            other => Err(io::Error::other(format!("AX pid failed with {}", other.0))),
        }
    }

    fn ax_point(value: &CFType) -> Option<CGPoint> {
        let value = value.downcast_ref::<AXValue>()?;
        if unsafe { value.r#type() } != AXValueType::CGPoint {
            return None;
        }
        let mut point = CGPoint { x: 0.0, y: 0.0 };
        let slot = NonNull::new((&mut point as *mut CGPoint).cast::<c_void>())?;
        unsafe { value.value(AXValueType::CGPoint, slot) }.then_some(point)
    }

    fn ax_size(value: &CFType) -> Option<CGSize> {
        let value = value.downcast_ref::<AXValue>()?;
        if unsafe { value.r#type() } != AXValueType::CGSize {
            return None;
        }
        let mut size = CGSize {
            width: 0.0,
            height: 0.0,
        };
        let slot = NonNull::new((&mut size as *mut CGSize).cast::<c_void>())?;
        unsafe { value.value(AXValueType::CGSize, slot) }.then_some(size)
    }

    fn protected_running_application(pid: libc::pid_t) -> bool {
        let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
            return false;
        };
        let bundle = app.bundleIdentifier().map(|value| value.to_string());
        let name = app.localizedName().map(|value| value.to_string());
        is_protected_terminal_app(bundle.as_deref(), name.as_deref())
    }

    enum ControlledWindow {
        Note(NoteWindow),
        Image(ImageWindow),
    }

    impl ControlledWindow {
        fn request(&self) -> CollectWindowRequestId {
            match self {
                Self::Note(window) => window.request,
                Self::Image(window) => window.request,
            }
        }

        fn kind(&self) -> CollectWindowKind {
            match self {
                Self::Note(_) => CollectWindowKind::Note,
                Self::Image(_) => CollectWindowKind::Meme,
            }
        }

        fn window(&self) -> &NSWindow {
            match self {
                Self::Note(window) => &window.window,
                Self::Image(window) => &window.window,
            }
        }

        fn frame(&self, desktop: Rect) -> Rect {
            world_rect_from_appkit_frame(self.window().frame(), desktop)
        }

        fn move_to(&mut self, top_left: Vec2, desktop: Rect) {
            let frame = appkit_frame_for_world_rect(world_rect_at(top_left, self.size()), desktop);
            self.window().setFrame_display(nsrect(frame), true);
        }

        fn size(&self) -> Vec2 {
            let frame = self.window().frame();
            Vec2::new(frame.size.width as f32, frame.size.height as f32)
        }

        fn set_passthrough(&self, passthrough: bool) {
            self.window().setIgnoresMouseEvents(passthrough);
        }

        fn focus(&self) {
            self.window().makeKeyAndOrderFront(None);
        }

        fn type_text(&self, text: &str) {
            if let Self::Note(window) = self {
                window.label.setStringValue(&NSString::from_str(text));
            }
        }
    }

    struct NoteWindow {
        request: CollectWindowRequestId,
        window: Retained<NSWindow>,
        label: Retained<NSTextField>,
    }

    struct ImageWindow {
        request: CollectWindowRequestId,
        window: Retained<NSWindow>,
        _image_view: Retained<NSImageView>,
        _image: Retained<NSImage>,
        _buffer: Vec<u8>,
    }

    pub struct CollectWindowController {
        mtm: Option<MainThreadMarker>,
        next_id: u64,
        windows: HashMap<CollectWindowId, ControlledWindow>,
        spawn_top_left: Vec2,
        coordinate_space: Rect,
    }

    impl CollectWindowController {
        pub fn new(primary_bounds: Rect, coordinate_space: Rect) -> Self {
            Self {
                mtm: MainThreadMarker::new(),
                next_id: 1,
                windows: HashMap::new(),
                spawn_top_left: Vec2::new(primary_bounds.min.x + 40.0, primary_bounds.min.y + 80.0),
                coordinate_space,
            }
        }

        pub fn update_display_bounds(&mut self, primary_bounds: Rect, coordinate_space: Rect) {
            self.spawn_top_left =
                Vec2::new(primary_bounds.min.x + 40.0, primary_bounds.min.y + 80.0);
            self.coordinate_space = coordinate_space;
        }

        pub fn snapshot(&self) -> Option<CollectWindowSnapshot> {
            self.windows
                .iter()
                .find(|(_, window)| window.window().isVisible())
                .map(|(id, window)| CollectWindowSnapshot {
                    id: *id,
                    request: window.request(),
                    kind: window.kind(),
                    rect: window.frame(self.coordinate_space),
                    alive: true,
                })
        }

        pub fn spawn_note(
            &mut self,
            request: CollectWindowRequestId,
        ) -> io::Result<CollectWindowId> {
            if let Some(id) = self.find_request(request) {
                return Ok(id);
            }
            let mtm = self
                .mtm
                .ok_or_else(|| io::Error::other("macOS collect windows require the main thread"))?;
            let id = self.alloc_id();
            let size = Vec2::new(340.0, 180.0);
            let window = create_prop_window(
                mtm,
                "Honk300 Note",
                world_rect_at(self.spawn_top_left, size),
                self.coordinate_space,
            );
            let label_frame = nsrect(AppKitFrame {
                x: 18.0,
                y: 18.0,
                width: size.x as f64 - 36.0,
                height: size.y as f64 - 36.0,
            });
            let label = NSTextField::labelWithString(&NSString::from_str(""), mtm);
            label.setFrame(label_frame);
            label.setEditable(false);
            label.setSelectable(false);
            label.setDrawsBackground(false);
            label.setMaximumNumberOfLines(0);
            label.setTextColor(Some(&NSColor::blackColor()));
            if let Some(content) = window.contentView() {
                content.addSubview(&label);
            }
            window.orderFrontRegardless();
            self.windows.insert(
                id,
                ControlledWindow::Note(NoteWindow {
                    request,
                    window,
                    label,
                }),
            );
            Ok(id)
        }

        pub fn spawn_image(
            &mut self,
            request: CollectWindowRequestId,
            title: &str,
            pixmap: &Pixmap,
        ) -> io::Result<CollectWindowId> {
            if let Some(id) = self.find_request(request) {
                return Ok(id);
            }
            let mtm = self
                .mtm
                .ok_or_else(|| io::Error::other("macOS collect windows require the main thread"))?;
            let id = self.alloc_id();
            let size = Vec2::new(pixmap.width() as f32, pixmap.height() as f32);
            let window = create_prop_window(
                mtm,
                title,
                world_rect_at(self.spawn_top_left, size),
                self.coordinate_space,
            );
            let view_frame = nsrect(AppKitFrame {
                x: 0.0,
                y: 0.0,
                width: size.x as f64,
                height: size.y as f64,
            });
            let image_view = NSImageView::initWithFrame(NSImageView::alloc(mtm), view_frame);
            let buffer = pixmap_bgra(pixmap);
            let image = image_from_bgra(&buffer, pixmap.width(), pixmap.height())?;
            image_view.setImage(Some(&image));
            window.setContentView(Some(&image_view));
            window.orderFrontRegardless();
            self.windows.insert(
                id,
                ControlledWindow::Image(ImageWindow {
                    request,
                    window,
                    _image_view: image_view,
                    _image: image,
                    _buffer: buffer,
                }),
            );
            Ok(id)
        }

        pub fn move_window(&mut self, id: CollectWindowId, top_left: Vec2) -> io::Result<()> {
            if let Some(window) = self.windows.get_mut(&id) {
                window.move_to(top_left, self.coordinate_space);
            }
            Ok(())
        }

        pub fn set_passthrough(
            &mut self,
            id: CollectWindowId,
            passthrough: bool,
        ) -> io::Result<()> {
            if let Some(window) = self.windows.get(&id) {
                window.set_passthrough(passthrough);
            }
            Ok(())
        }

        pub fn focus(&mut self, id: CollectWindowId) -> io::Result<()> {
            if let Some(window) = self.windows.get(&id) {
                window.focus();
            }
            Ok(())
        }

        pub fn type_text(&mut self, id: CollectWindowId, text: &str) -> io::Result<()> {
            if let Some(window) = self.windows.get(&id) {
                window.type_text(text);
            }
            Ok(())
        }

        pub fn close(&mut self, id: CollectWindowId) {
            if let Some(window) = self.windows.remove(&id) {
                window.window().orderOut(None);
            }
        }

        fn alloc_id(&mut self) -> CollectWindowId {
            let id = CollectWindowId(self.next_id);
            self.next_id += 1;
            id
        }

        fn find_request(&self, request: CollectWindowRequestId) -> Option<CollectWindowId> {
            self.windows
                .iter()
                .find_map(|(id, window)| (window.request() == request).then_some(*id))
        }
    }

    fn create_prop_window(
        mtm: MainThreadMarker,
        title: &str,
        rect: Rect,
        desktop: Rect,
    ) -> Retained<NSWindow> {
        let frame = nsrect(appkit_frame_for_world_rect(rect, desktop));
        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::UtilityWindow;
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                frame,
                style,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        window.setTitle(&NSString::from_str(title));
        unsafe {
            window.setReleasedWhenClosed(false);
        }
        window.setCanHide(false);
        window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
        window
    }

    fn world_rect_at(top_left: Vec2, size: Vec2) -> Rect {
        Rect::new(
            top_left,
            Vec2::new(top_left.x + size.x, top_left.y + size.y),
        )
    }

    fn world_rect_from_appkit_frame(frame: NSRect, desktop: Rect) -> Rect {
        let min = appkit_point_to_world(
            (frame.origin.x, frame.origin.y + frame.size.height),
            desktop,
        );
        let max = Vec2::new(
            min.x + frame.size.width as f32,
            min.y + frame.size.height as f32,
        );
        Rect::new(min, max)
    }

    fn pixmap_bgra(pixmap: &Pixmap) -> Vec<u8> {
        let mut out = Vec::with_capacity(pixmap.data().len());
        for pixel in pixmap.data().chunks_exact(4) {
            out.push(pixel[2]);
            out.push(pixel[1]);
            out.push(pixel[0]);
            out.push(pixel[3]);
        }
        out
    }
}

#[cfg(target_os = "macos")]
pub use platform::{
    accessibility_state, local_time, pointer_state, presence_state, warp_cursor,
    AccessibilityState, CollectWindowController, ForeignWindowWatcher, Overlay,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appkit_conversion_uses_main_display_not_virtual_desktop_height() {
        let main_display = Rect::new(Vec2::ZERO, Vec2::new(1440.0, 900.0));
        let virtual_desktop = Rect::new(Vec2::ZERO, Vec2::new(1440.0, 1500.0));
        let rect_on_display_below_main =
            Rect::new(Vec2::new(10.0, 1000.0), Vec2::new(110.0, 1100.0));

        let coordinate_space = appkit_coordinate_space(main_display, virtual_desktop);
        assert_eq!(coordinate_space, main_display);
        assert_eq!(
            appkit_frame_for_world_rect(rect_on_display_below_main, coordinate_space).y,
            -200.0
        );
    }

    #[test]
    fn a_stationary_click_is_not_classified_as_a_window_drag() {
        let mut classifier = DragClassifier::default();
        assert!(!classifier.observe(42, Vec2::new(100.0, 100.0)));
        assert!(!classifier.observe(42, Vec2::new(100.0, 100.0)));
        assert!(classifier.observe(42, Vec2::new(103.0, 100.0)));
        classifier.release();
        assert!(!classifier.observe(42, Vec2::new(103.0, 100.0)));
    }

    #[test]
    fn display_topology_refresh_keeps_one_window_per_active_display() {
        let active_display_ids = vec![10_u32, 20, 30];
        let current_window_display_ids = vec![10_u32];
        assert_eq!(
            reconciled_display_ids(&current_window_display_ids, &active_display_ids),
            active_display_ids
        );
    }

    #[test]
    fn appkit_frame_converts_y_down_world_to_y_up_appkit() {
        let desktop = Rect::new(Vec2::new(-1280.0, -900.0), Vec2::new(1920.0, 1080.0));
        let rect = Rect::new(Vec2::new(10.0, 20.0), Vec2::new(110.0, 70.0));
        assert_eq!(
            appkit_frame_for_world_rect(rect, desktop),
            AppKitFrame {
                x: 10.0,
                y: 1010.0,
                width: 100.0,
                height: 50.0,
            }
        );
    }

    #[test]
    fn appkit_point_conversion_handles_negative_monitor_origins() {
        let desktop = Rect::new(Vec2::new(-1280.0, -900.0), Vec2::new(1920.0, 1080.0));
        assert_eq!(
            appkit_point_to_world((-640.0, 1880.0), desktop),
            Vec2::new(-640.0, -800.0)
        );
    }

    #[test]
    fn terminal_app_classifier_covers_common_macos_terminals() {
        assert!(is_protected_terminal_app(Some("com.apple.Terminal"), None));
        assert!(is_protected_terminal_app(
            Some("com.googlecode.iterm2"),
            None
        ));
        assert!(is_protected_terminal_app(None, Some("Ghostty")));
        assert!(!is_protected_terminal_app(
            Some("com.apple.TextEdit"),
            Some("TextEdit")
        ));
    }
}
