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
                    | "com.openai.codex"
                    | "com.microsoft.vscode"
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
                    | "codex"
                    | "visual studio code"
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
        CollectWindowCloseOrigin, CollectWindowId, CollectWindowKind, CollectWindowRequestId,
        CollectWindowSnapshot,
    };
    use honk_engine::{
        ForeignWindowId, ForeignWindowSnapshot, LocalTime, PresenceSnapshot, Rect, Vec2,
    };
    use objc2::rc::{autoreleasepool, Retained};
    use objc2::MainThreadMarker;
    use objc2::{AnyThread, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSBitmapFormat,
        NSBitmapImageRep, NSColor, NSColorSpace, NSDeviceRGBColorSpace, NSEvent, NSEventMask,
        NSImage, NSImageAlignment, NSImageCacheMode, NSImageRep, NSImageScaling, NSImageView,
        NSRunningApplication, NSScreenSaverWindowLevel, NSTextField, NSView, NSWindow,
        NSWindowCollectionBehavior, NSWindowStyleMask, NSWorkspace,
    };
    use objc2_application_services::{
        kAXTrustedCheckOptionPrompt, AXError, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
        AXUIElement, AXValue, AXValueType,
    };
    use objc2_core_foundation::{
        CFBoolean, CFDictionary, CFRetained, CFString, CFType, CGPoint, CGRect, CGSize,
    };
    #[cfg(test)]
    use objc2_core_graphics::CGImage;
    use objc2_core_graphics::{
        CGDisplayBounds, CGError, CGEventSourceStateID, CGGetActiveDisplayList, CGMainDisplayID,
        CGMouseButton, CGWarpMouseCursorPosition,
    };
    use objc2_foundation::{
        NSBundle, NSDate, NSDefaultRunLoopMode, NSInteger, NSPoint, NSRect, NSSize, NSString, NSURL,
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
    const APPKIT_EVENT_PUMP_INTERVAL: Duration = Duration::from_nanos(16_666_667);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AccessibilityState {
        Trusted,
        Denied,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MacBundleReleaseMetadata {
        pub bundle_id: String,
        pub version: String,
        pub tag: String,
        pub commit: String,
    }

    enum BundleInfoValue {
        String(String),
        NonString,
    }

    fn bundle_release_metadata_from_values(
        mut value_for_key: impl FnMut(&str) -> Option<BundleInfoValue>,
    ) -> Option<MacBundleReleaseMetadata> {
        let mut string_for_key = |key| match value_for_key(key)? {
            BundleInfoValue::String(value) => Some(value),
            BundleInfoValue::NonString => None,
        };
        Some(MacBundleReleaseMetadata {
            bundle_id: string_for_key("CFBundleIdentifier")?,
            version: string_for_key("CFBundleShortVersionString")?,
            tag: string_for_key("Honk300ReleaseTag")?,
            commit: string_for_key("Honk300ReleaseCommit")?,
        })
    }

    pub fn main_bundle_release_metadata() -> Option<MacBundleReleaseMetadata> {
        let bundle = NSBundle::mainBundle();
        bundle_release_metadata_from_values(|key| {
            let value = bundle.objectForInfoDictionaryKey(&NSString::from_str(key))?;
            Some(match value.downcast_ref::<NSString>() {
                Some(value) => BundleInfoValue::String(value.to_string()),
                None => BundleInfoValue::NonString,
            })
        })
    }

    pub fn accessibility_state() -> AccessibilityState {
        if unsafe { AXIsProcessTrusted() } {
            AccessibilityState::Trusted
        } else {
            AccessibilityState::Denied
        }
    }

    fn request_accessibility_prompt_with(
        prompt: impl FnOnce() -> AccessibilityState,
    ) -> io::Result<AccessibilityState> {
        let _main_thread = MainThreadMarker::new().ok_or_else(|| {
            io::Error::other("macOS Accessibility prompt must be requested on the main thread")
        })?;
        Ok(autoreleasepool(|_| prompt()))
    }

    pub fn request_accessibility_prompt() -> io::Result<AccessibilityState> {
        request_accessibility_prompt_with(|| {
            let options = CFDictionary::<CFType, CFType>::from_slices(
                &[unsafe { kAXTrustedCheckOptionPrompt }.as_ref()],
                &[CFBoolean::new(true).as_ref()],
            );
            if unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) } {
                AccessibilityState::Trusted
            } else {
                AccessibilityState::Denied
            }
        })
    }

    fn accessibility_settings_urls() -> [&'static str; 2] {
        [
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension",
        ]
    }

    fn try_accessibility_settings_urls(mut open: impl FnMut(&str) -> bool) -> io::Result<()> {
        for url in accessibility_settings_urls() {
            if open(url) {
                return Ok(());
            }
        }
        Err(io::Error::other(
            "macOS rejected the Accessibility and Privacy & Security Settings URLs",
        ))
    }

    pub fn open_accessibility_settings() -> io::Result<()> {
        let _main_thread = MainThreadMarker::new().ok_or_else(|| {
            io::Error::other("macOS Accessibility settings must be opened on the main thread")
        })?;
        autoreleasepool(|_| {
            let workspace = NSWorkspace::sharedWorkspace();
            try_accessibility_settings_urls(|url_string| {
                NSURL::URLWithString(&NSString::from_str(url_string))
                    .is_some_and(|url| workspace.openURL(&url))
            })
        })
    }

    pub fn local_time() -> LocalTime {
        unsafe {
            let now = libc::time(ptr::null_mut());
            let mut out = std::mem::zeroed::<libc::tm>();
            let tm = if libc::localtime_r(&now, &mut out).is_null() {
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
        interactive: bool,
        next_event_pump: Instant,
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
                displays.push(DisplayWindow::new(mtm, info, coordinate_space, false)?);
            }
            Ok(Self {
                app,
                displays,
                primary_bounds,
                virtual_bounds,
                topology_changed: false,
                interactive: false,
                next_event_pump: Instant::now(),
                next_display_refresh: Instant::now() + DISPLAY_REFRESH_INTERVAL,
            })
        }

        pub fn pump(&mut self) -> bool {
            autoreleasepool(|_| self.pump_inner())
        }

        fn pump_inner(&mut self) -> bool {
            let now = Instant::now();
            if now >= self.next_event_pump {
                self.next_event_pump = now + APPKIT_EVENT_PUMP_INTERVAL;
                // Non-blocking drain of the AppKit event queue. Without this the queue is never
                // serviced, so window-chrome events — including the Titled|Closable
                // collect-window close button — are silently ignored. Driving the AppKit run loop
                // also commits Core Animation transactions, so cap it at the same 60 Hz maximum as
                // presentation instead of doing that work on every 120 Hz simulation tick.
                let mode = unsafe { NSDefaultRunLoopMode };
                while let Some(event) = self.app.nextEventMatchingMask_untilDate_inMode_dequeue(
                    NSEventMask::Any,
                    Some(&NSDate::distantPast()),
                    mode,
                    true,
                ) {
                    self.app.sendEvent(&event);
                }
            }
            if now >= self.next_display_refresh {
                self.next_display_refresh = now + DISPLAY_REFRESH_INTERVAL;
                if let Err(err) = self.refresh_display_topology() {
                    eprintln!(
                        "honk300: macOS display refresh failed; keeping prior topology ({err})"
                    );
                }
            }
            true
        }

        pub fn pointer_state(&self) -> (f32, f32, bool) {
            autoreleasepool(|_| {
                let coordinate_space =
                    appkit_coordinate_space(self.primary_bounds, self.virtual_bounds);
                let point = NSEvent::mouseLocation();
                let world = appkit_point_to_world((point.x, point.y), coordinate_space);
                let left_down = objc2_core_graphics::CGEventSource::button_state(
                    CGEventSourceStateID::CombinedSessionState,
                    CGMouseButton::Left,
                );
                (world.x, world.y, left_down)
            })
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
            autoreleasepool(|_| self.present_inner(dirty, pixmap))
        }

        fn present_inner(&mut self, dirty: Rect, pixmap: &Pixmap) -> io::Result<()> {
            for display in &mut self.displays {
                if let Some(clip) = dirty.intersection(display.info.bounds) {
                    display.present(dirty, clip, pixmap)?;
                } else {
                    display.clear();
                }
            }
            Ok(())
        }

        pub fn set_interactive(&mut self, over_goose: bool) {
            if self.interactive == over_goose {
                return;
            }
            for display in &mut self.displays {
                display.window.setIgnoresMouseEvents(!over_goose);
            }
            self.interactive = over_goose;
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
                    None => DisplayWindow::new(mtm, info, coordinate_space, self.interactive)?,
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
        surface: Option<BitmapSurface>,
        view_frame: Option<AppKitFrame>,
        visible: bool,
    }

    fn ignores_mouse_events_for_interactivity(interactive: bool) -> bool {
        !interactive
    }

    fn overlay_window_color_space() -> Retained<NSColorSpace> {
        NSColorSpace::sRGBColorSpace()
    }

    impl DisplayWindow {
        fn new(
            mtm: MainThreadMarker,
            info: DisplayInfo,
            desktop: Rect,
            interactive: bool,
        ) -> io::Result<Self> {
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
            // Use a stable standard-RGB destination instead of inheriting the physical screen's
            // wide-gamut profile. AppKit can preserve transparent backing-store capture while
            // avoiding a fresh Device RGB -> Display P3 transform on every presented frame;
            // WindowServer still performs final per-display composition.
            window.setColorSpace(Some(&overlay_window_color_space()));
            window.setHasShadow(false);
            unsafe {
                window.setReleasedWhenClosed(false);
            }
            window.setCanHide(false);
            window.setIgnoresMouseEvents(ignores_mouse_events_for_interactivity(interactive));
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
            let root_view = NSView::initWithFrame(NSView::alloc(mtm), view_frame);
            let image_view = NSImageView::initWithFrame(NSImageView::alloc(mtm), view_frame);
            image_view.setImageScaling(NSImageScaling::ScaleNone);
            image_view.setImageAlignment(NSImageAlignment::AlignTopLeft);
            // Keep the pixels in AppKit's ordinary window backing store. A custom child CALayer
            // looked correct on the physical display, but WindowServer capture and screen sharing
            // could omit it or composite its clear pixels as opaque black rectangles.
            root_view.addSubview(&image_view);
            window.setContentView(Some(&root_view));
            window.orderFrontRegardless();

            Ok(Self {
                info,
                window,
                image_view,
                surface: None,
                view_frame: None,
                visible: false,
            })
        }

        fn present(&mut self, dirty: Rect, clip: Rect, pixmap: &Pixmap) -> io::Result<()> {
            let clip = clip.pixel_aligned();
            let width = clip.width().ceil().max(1.0) as u32;
            let height = clip.height().ceil().max(1.0) as u32;
            let replacement = replacement_surface_extent(
                self.surface
                    .as_ref()
                    .map(|surface| (surface.width, surface.height)),
                width,
                height,
            );
            if let Some((capacity_width, capacity_height)) = replacement {
                self.surface = Some(BitmapSurface::new(capacity_width, capacity_height)?);
            }
            let surface = self
                .surface
                .as_ref()
                .expect("surface allocated for the requested dimensions");
            surface.copy_clipped_rgba(dirty, clip, pixmap, width, height)?;
            let local_frame = active_image_frame(self.info.bounds, clip, width, height);
            if self.view_frame != Some(local_frame) {
                if self.view_frame.is_some_and(|frame| {
                    frame.width == local_frame.width && frame.height == local_frame.height
                }) {
                    self.image_view.setFrameOrigin(NSPoint {
                        x: local_frame.x,
                        y: local_frame.y,
                    });
                } else {
                    self.image_view.setFrame(nsrect(local_frame));
                }
                self.view_frame = Some(local_frame);
            }
            if replacement.is_some() || !self.visible {
                self.image_view.setImage(Some(&surface.image));
            }
            // The NSImage and NSBitmapImageRep stay stable while their RGBA storage is reused.
            // Explicitly invalidate the image view because mutating that storage does not change
            // the NSImage object's identity, then flush just this window's pending AppKit draw.
            NSView::setNeedsDisplay(&self.image_view, true);
            self.window.displayIfNeeded();
            self.visible = true;

            Ok(())
        }

        fn clear(&mut self) {
            if self.visible {
                self.image_view.setImage(None);
                NSView::setNeedsDisplay(&self.image_view, true);
                self.window.displayIfNeeded();
                self.visible = false;
            }
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

    struct BitmapSurface {
        width: u32,
        height: u32,
        rep: Retained<NSBitmapImageRep>,
        image: Retained<NSImage>,
    }

    fn rounded_surface_extent(extent: u32) -> u32 {
        extent.saturating_add(31) / 32 * 32
    }

    fn replacement_surface_extent(
        current: Option<(u32, u32)>,
        active_width: u32,
        active_height: u32,
    ) -> Option<(u32, u32)> {
        let desired = (
            rounded_surface_extent(active_width),
            rounded_surface_extent(active_height),
        );
        let Some((current_width, current_height)) = current else {
            return Some(desired);
        };
        let must_grow = current_width < active_width || current_height < active_height;
        // A note, meme, or distant dirty region can temporarily make the union much larger than
        // the goose. Do not make every later frame redraw that stale capacity: shrink once either
        // axis exceeds twice the active bucket, while retaining small 32-pixel variations.
        let should_shrink = current_width > desired.0.saturating_mul(2)
            || current_height > desired.1.saturating_mul(2);
        (must_grow || should_shrink).then_some(desired)
    }

    fn active_image_frame(
        display_bounds: Rect,
        clip: Rect,
        active_width: u32,
        active_height: u32,
    ) -> AppKitFrame {
        AppKitFrame {
            x: (clip.min.x - display_bounds.min.x) as f64,
            y: (display_bounds.max.y - clip.max.y) as f64,
            width: active_width as f64,
            height: active_height as f64,
        }
    }

    impl BitmapSurface {
        fn new(width: u32, height: u32) -> io::Result<Self> {
            // Pass NULL planes so AppKit allocates and owns the pixel storage for the rep's
            // lifetime. Alpha-last device RGB matches tiny-skia's premultiplied RGBA byte
            // contract, so no channel swizzle is needed.
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
                    NSBitmapFormat::empty(),
                    (width * 4) as NSInteger,
                    32,
                )
            }
            .ok_or_else(|| io::Error::other("failed to create NSBitmapImageRep"))?;

            let image = NSImage::initWithSize(
                NSImage::alloc(),
                NSSize {
                    width: width as f64,
                    height: height as f64,
                },
            );
            image.setCacheMode(NSImageCacheMode::Never);
            let rep_ref: &NSImageRep = &rep;
            image.addRepresentation(rep_ref);
            Ok(Self {
                width,
                height,
                rep,
                image,
            })
        }

        fn copy_tight_rgba(&self, rgba: &[u8]) -> io::Result<()> {
            let expected = self.width as usize * self.height as usize * 4;
            if rgba.len() != expected {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("expected {expected} RGBA bytes, got {}", rgba.len()),
                ));
            }
            let dst = self.bitmap_data()?;
            let dst_stride = self.rep.bytesPerRow() as usize;
            let src_stride = self.width as usize * 4;
            for row in 0..self.height as usize {
                unsafe {
                    ptr::copy_nonoverlapping(
                        rgba.as_ptr().add(row * src_stride),
                        dst.add(row * dst_stride),
                        src_stride,
                    );
                }
            }
            Ok(())
        }

        fn copy_clipped_rgba(
            &self,
            dirty: Rect,
            clip: Rect,
            pixmap: &Pixmap,
            active_width: u32,
            active_height: u32,
        ) -> io::Result<()> {
            let dst = self.bitmap_data()?;
            let dst_stride = self.rep.bytesPerRow() as usize;
            let src_stride = pixmap.width() as usize * 4;
            let src_x = (clip.min.x - dirty.min.x).round() as isize;
            let src_y = (clip.min.y - dirty.min.y).round() as isize;
            let src = pixmap.data();

            for y in 0..self.height as usize {
                let dst_row = unsafe { dst.add(y * dst_stride) };
                unsafe { ptr::write_bytes(dst_row, 0, dst_stride) };
                if y >= active_height as usize {
                    continue;
                }
                let source_y = src_y + y as isize;
                if source_y < 0 || source_y >= pixmap.height() as isize {
                    continue;
                }
                let start_x = src_x.max(0) as usize;
                let skipped = (start_x as isize - src_x).max(0) as usize;
                if start_x >= pixmap.width() as usize || skipped >= active_width as usize {
                    continue;
                }
                let copy_pixels =
                    (active_width as usize - skipped).min(pixmap.width() as usize - start_x);
                let src_offset = source_y as usize * src_stride + start_x * 4;
                unsafe {
                    ptr::copy_nonoverlapping(
                        src.as_ptr().add(src_offset),
                        dst_row.add(skipped * 4),
                        copy_pixels * 4,
                    );
                }
            }
            Ok(())
        }

        fn bitmap_data(&self) -> io::Result<*mut u8> {
            let data = self.rep.bitmapData();
            if data.is_null() {
                Err(io::Error::other("NSBitmapImageRep provided no bitmap data"))
            } else {
                Ok(data)
            }
        }

        #[cfg(test)]
        fn direct_cg_image(&self) -> io::Result<Retained<CGImage>> {
            self.rep
                .CGImage()
                .ok_or_else(|| io::Error::other("AppKit did not expose the reusable RGBA bitmap"))
        }
    }

    fn image_from_rgba(buffer: &[u8], width: u32, height: u32) -> io::Result<Retained<NSImage>> {
        let surface = BitmapSurface::new(width, height)?;
        surface.copy_tight_rgba(buffer)?;
        Ok(surface.image)
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
    }

    fn note_text_color() -> Retained<NSColor> {
        NSColor::labelColor()
    }

    fn preferred_collect_window_id(
        active: Option<(CollectWindowRequestId, CollectWindowKind)>,
        candidates: impl IntoIterator<
            Item = (CollectWindowId, CollectWindowRequestId, CollectWindowKind),
        >,
    ) -> Option<CollectWindowId> {
        // A lingering note or meme may outlive the engine task that created it. Prefer the
        // currently spawned typed request so HashMap iteration order cannot starve a newer task.
        let mut fallback = None;
        for (id, request, kind) in candidates {
            if active == Some((request, kind)) {
                return Some(id);
            }
            fallback.get_or_insert(id);
        }
        fallback
    }

    pub struct CollectWindowController {
        mtm: Option<MainThreadMarker>,
        next_id: u64,
        windows: HashMap<CollectWindowId, ControlledWindow>,
        active_request: Option<(CollectWindowRequestId, CollectWindowKind)>,
        spawn_top_left: Vec2,
        coordinate_space: Rect,
    }

    impl CollectWindowController {
        pub fn new(primary_bounds: Rect, virtual_desktop_bounds: Rect) -> Self {
            Self {
                mtm: MainThreadMarker::new(),
                next_id: 1,
                windows: HashMap::new(),
                active_request: None,
                spawn_top_left: Vec2::new(primary_bounds.min.x + 40.0, primary_bounds.min.y + 80.0),
                coordinate_space: appkit_coordinate_space(primary_bounds, virtual_desktop_bounds),
            }
        }

        pub fn update_display_bounds(
            &mut self,
            primary_bounds: Rect,
            virtual_desktop_bounds: Rect,
        ) {
            self.spawn_top_left =
                Vec2::new(primary_bounds.min.x + 40.0, primary_bounds.min.y + 80.0);
            self.coordinate_space = appkit_coordinate_space(primary_bounds, virtual_desktop_bounds);
        }

        pub fn snapshot(&mut self) -> Option<CollectWindowSnapshot> {
            if let Some(id) = self.windows.iter().find_map(|(id, window)| {
                let window = window.window();
                (!window.isVisible() && !window.isMiniaturized()).then_some(*id)
            }) {
                let window = self.windows.remove(&id).expect("closed window id");
                if self.active_request == Some((window.request(), window.kind())) {
                    self.active_request = None;
                }
                return Some(CollectWindowSnapshot {
                    id,
                    request: window.request(),
                    kind: window.kind(),
                    rect: window.frame(self.coordinate_space),
                    alive: false,
                    close_origin: Some(CollectWindowCloseOrigin::User),
                });
            }
            let id = preferred_collect_window_id(
                self.active_request,
                self.windows
                    .iter()
                    .filter(|(_, window)| window.window().isVisible())
                    .map(|(id, window)| (*id, window.request(), window.kind())),
            )?;
            self.windows.get(&id).map(|window| CollectWindowSnapshot {
                id,
                request: window.request(),
                kind: window.kind(),
                rect: window.frame(self.coordinate_space),
                alive: true,
                close_origin: None,
            })
        }

        pub fn spawn_note(
            &mut self,
            request: CollectWindowRequestId,
        ) -> io::Result<CollectWindowId> {
            if let Some(id) = self.find_request(request, CollectWindowKind::Note) {
                self.active_request = Some((request, CollectWindowKind::Note));
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
            label.setTextColor(Some(&note_text_color()));
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
            self.active_request = Some((request, CollectWindowKind::Note));
            Ok(id)
        }

        pub fn spawn_image(
            &mut self,
            request: CollectWindowRequestId,
            title: &str,
            pixmap: &Pixmap,
        ) -> io::Result<CollectWindowId> {
            if let Some(id) = self.find_request(request, CollectWindowKind::Meme) {
                self.active_request = Some((request, CollectWindowKind::Meme));
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
            let image = image_from_rgba(pixmap.data(), pixmap.width(), pixmap.height())?;
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
                }),
            );
            self.active_request = Some((request, CollectWindowKind::Meme));
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
                if self.active_request == Some((window.request(), window.kind())) {
                    self.active_request = None;
                }
                window.window().orderOut(None);
            }
        }

        fn alloc_id(&mut self) -> CollectWindowId {
            let id = CollectWindowId(self.next_id);
            self.next_id += 1;
            id
        }

        fn find_request(
            &self,
            request: CollectWindowRequestId,
            kind: CollectWindowKind,
        ) -> Option<CollectWindowId> {
            self.windows.iter().find_map(|(id, window)| {
                (window.request() == request && window.kind() == kind).then_some(*id)
            })
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use objc2::runtime::NSObjectProtocol;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        #[test]
        fn accessibility_prompt_rejects_off_main_thread_before_calling_ax() {
            let ax_called = Arc::new(AtomicBool::new(false));
            let worker_ax_called = Arc::clone(&ax_called);
            let result = std::thread::spawn(move || {
                request_accessibility_prompt_with(|| {
                    worker_ax_called.store(true, Ordering::SeqCst);
                    AccessibilityState::Denied
                })
            })
            .join()
            .expect("worker thread must finish normally");

            let error = result.expect_err("off-main prompt requests must be rejected");
            assert_eq!(error.kind(), io::ErrorKind::Other);
            assert!(
                !ax_called.load(Ordering::SeqCst),
                "AX prompt closure must not run off the main thread"
            );
        }

        #[test]
        fn accessibility_settings_urls_prefer_direct_then_privacy_fallback() {
            assert_eq!(
                accessibility_settings_urls(),
                [
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                    "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension",
                ]
            );
        }

        #[test]
        fn accessibility_settings_rejects_off_main_thread() {
            let result = std::thread::spawn(open_accessibility_settings)
                .join()
                .expect("worker thread must finish normally");
            let error = result.expect_err("off-main settings requests must be rejected");
            assert_eq!(error.kind(), io::ErrorKind::Other);
        }

        #[test]
        fn accessibility_settings_failure_is_reported_only_after_both_urls_are_tried() {
            let mut attempted = Vec::new();
            let error = try_accessibility_settings_urls(|url| {
                attempted.push(url.to_owned());
                false
            })
            .expect_err("both rejected settings URLs must be actionable");

            assert_eq!(attempted, accessibility_settings_urls());
            assert_eq!(error.kind(), io::ErrorKind::Other);
        }

        #[test]
        fn bundle_release_metadata_rejects_missing_or_non_string_values() {
            let complete = bundle_release_metadata_from_values(|key| match key {
                "CFBundleIdentifier" => Some(BundleInfoValue::String("dev.emmetts.honk300".into())),
                "CFBundleShortVersionString" => Some(BundleInfoValue::String("0.3.3".into())),
                "Honk300ReleaseTag" => Some(BundleInfoValue::String("v0.3.3".into())),
                "Honk300ReleaseCommit" => Some(BundleInfoValue::String("abc123".into())),
                _ => None,
            });
            assert_eq!(
                complete,
                Some(MacBundleReleaseMetadata {
                    bundle_id: "dev.emmetts.honk300".into(),
                    version: "0.3.3".into(),
                    tag: "v0.3.3".into(),
                    commit: "abc123".into(),
                })
            );

            let missing = bundle_release_metadata_from_values(|key| match key {
                "CFBundleIdentifier" => Some(BundleInfoValue::String("dev.emmetts.honk300".into())),
                "CFBundleShortVersionString" => Some(BundleInfoValue::String("0.3.3".into())),
                "Honk300ReleaseTag" => None,
                "Honk300ReleaseCommit" => Some(BundleInfoValue::String("abc123".into())),
                _ => None,
            });
            assert_eq!(missing, None);

            let non_string = bundle_release_metadata_from_values(|key| match key {
                "CFBundleIdentifier" => Some(BundleInfoValue::String("dev.emmetts.honk300".into())),
                "CFBundleShortVersionString" => Some(BundleInfoValue::String("0.3.3".into())),
                "Honk300ReleaseTag" => Some(BundleInfoValue::String("v0.3.3".into())),
                "Honk300ReleaseCommit" => Some(BundleInfoValue::NonString),
                _ => None,
            });
            assert_eq!(non_string, None);
        }

        fn assert_component(actual: f64, byte: u8) {
            let expected = f64::from(byte) / 255.0;
            assert!(
                (actual - expected).abs() < 0.005,
                "expected {expected:.4} from byte {byte}, got {actual:.4}"
            );
        }

        fn assert_unpremultiplied_component(actual: f64, component: u8, alpha: u8) {
            let expected = f64::from(component) / f64::from(alpha);
            assert!(
                (actual - expected).abs() < 0.005,
                "expected {expected:.4} from premultiplied byte {component}/{alpha}, got {actual:.4}"
            );
        }

        #[test]
        fn note_text_uses_the_appearance_aware_system_label_color() {
            let actual = note_text_color();
            let expected = NSColor::labelColor();

            assert!(
                actual.isEqual(Some(&expected)),
                "note text must follow the active light, dark, and high-contrast appearance"
            );
        }

        #[test]
        fn collect_controller_prefers_the_active_typed_request_over_a_lingering_window() {
            let lingering = (
                CollectWindowId(1),
                CollectWindowRequestId(100),
                CollectWindowKind::Note,
            );
            let active = (
                CollectWindowId(2),
                CollectWindowRequestId(200),
                CollectWindowKind::Meme,
            );
            let same_request_wrong_kind = (CollectWindowId(3), active.1, CollectWindowKind::Note);

            assert_eq!(
                preferred_collect_window_id(
                    Some((active.1, active.2)),
                    [lingering, same_request_wrong_kind, active],
                ),
                Some(active.0),
                "an older lingering prop must not starve the currently active typed request"
            );
        }

        #[test]
        fn overlay_window_uses_a_stable_srgb_destination() {
            assert!(overlay_window_color_space().isEqual(Some(&NSColorSpace::sRGBColorSpace())));
        }

        #[test]
        fn appkit_interprets_tiny_skia_rgba_without_channel_or_alpha_swaps() {
            let rgba = [17_u8, 83, 149, 211];
            let image = image_from_rgba(&rgba, 1, 1).expect("create one-pixel AppKit image");
            let representations = image.representations();
            let rep = representations
                .firstObject()
                .expect("image owns its bitmap representation");
            let bitmap = rep
                .downcast_ref::<NSBitmapImageRep>()
                .expect("representation is an NSBitmapImageRep");
            let color = bitmap.colorAtX_y(0, 0).expect("read AppKit pixel color");

            assert_unpremultiplied_component(color.redComponent(), rgba[0], rgba[3]);
            assert_unpremultiplied_component(color.greenComponent(), rgba[1], rgba[3]);
            assert_unpremultiplied_component(color.blueComponent(), rgba[2], rgba[3]);
            assert_component(color.alphaComponent(), rgba[3]);
        }

        #[test]
        fn core_graphics_surface_declares_premultiplied_rgba_without_a_swizzle() {
            let rgba = [17_u8, 83, 149, 211];
            let surface = BitmapSurface::new(1, 1).expect("create one-pixel surface");
            surface.copy_tight_rgba(&rgba).expect("copy RGBA pixel");
            let image = surface.direct_cg_image().expect("create direct CGImage");

            assert_eq!(
                objc2_core_graphics::CGImage::alpha_info(Some(&image)),
                objc2_core_graphics::CGImageAlphaInfo::PremultipliedLast
            );
            assert_eq!(
                objc2_core_graphics::CGImage::bits_per_pixel(Some(&image)),
                32
            );
        }

        #[test]
        fn appkit_cgimage_observes_reused_bitmap_mutations() {
            let surface = BitmapSurface::new(1, 1).expect("create one-pixel surface");
            surface
                .copy_tight_rgba(&[17, 83, 149, 211])
                .expect("copy first pixel");
            let first = surface.direct_cg_image().expect("first image");
            let first_provider = CGImage::data_provider(Some(&first)).expect("first provider");
            let first_data = objc2_core_graphics::CGDataProvider::data(Some(&first_provider))
                .expect("first bytes");
            let first_bytes = unsafe { first_data.as_bytes_unchecked() };
            assert_eq!(&first_bytes[..4], &[17, 83, 149, 211]);

            surface
                .copy_tight_rgba(&[201, 71, 13, 223])
                .expect("mutate reused pixel");
            let second = surface.direct_cg_image().expect("second image");
            let second_provider = CGImage::data_provider(Some(&second)).expect("second provider");
            let second_data = objc2_core_graphics::CGDataProvider::data(Some(&second_provider))
                .expect("second bytes");
            let second_bytes = unsafe { second_data.as_bytes_unchecked() };
            assert_eq!(&second_bytes[..4], &[201, 71, 13, 223]);
        }

        #[test]
        fn bitmap_surfaces_grow_in_small_stable_steps() {
            assert_eq!(rounded_surface_extent(1), 32);
            assert_eq!(rounded_surface_extent(244), 256);
            assert_eq!(rounded_surface_extent(257), 288);
        }

        #[test]
        fn bitmap_surface_capacity_shrinks_after_a_large_transient_frame() {
            assert_eq!(replacement_surface_extent(None, 244, 193), Some((256, 224)));
            assert_eq!(replacement_surface_extent(Some((256, 224)), 247, 199), None);
            assert_eq!(
                replacement_surface_extent(Some((1_216, 928)), 244, 193),
                Some((256, 224))
            );
            assert_eq!(
                replacement_surface_extent(Some((256, 224)), 257, 193),
                Some((288, 224))
            );
        }

        #[test]
        fn image_view_frame_tracks_active_pixels_instead_of_surface_capacity() {
            let display = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(1440.0, 900.0));
            let clip = Rect::new(Vec2::new(100.0, 620.0), Vec2::new(344.0, 813.0));

            assert_eq!(
                active_image_frame(display, clip, 244, 193),
                AppKitFrame {
                    x: 100.0,
                    y: 87.0,
                    width: 244.0,
                    height: 193.0,
                }
            );
        }

        #[test]
        fn newly_reconciled_window_inherits_cached_interactive_state() {
            assert!(!ignores_mouse_events_for_interactivity(true));
            assert!(ignores_mouse_events_for_interactivity(false));
        }
    }
}

#[cfg(target_os = "macos")]
pub use platform::{
    accessibility_state, local_time, main_bundle_release_metadata, open_accessibility_settings,
    presence_state, request_accessibility_prompt, warp_cursor, AccessibilityState,
    CollectWindowController, ForeignWindowWatcher, MacBundleReleaseMetadata, Overlay,
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
    fn collect_window_space_handles_negative_and_below_main_monitors() {
        let main_display = Rect::new(Vec2::ZERO, Vec2::new(1440.0, 900.0));
        let virtual_desktop = Rect::new(Vec2::new(-1280.0, -600.0), Vec2::new(3360.0, 1800.0));
        let coordinate_space = appkit_coordinate_space(main_display, virtual_desktop);
        let world = Rect::new(Vec2::new(-900.0, 1050.0), Vec2::new(-560.0, 1230.0));
        let appkit = appkit_frame_for_world_rect(world, coordinate_space);

        assert_eq!(appkit.x, -900.0);
        assert_eq!(appkit.y, -330.0);
        assert_eq!(
            appkit_point_to_world((appkit.x, appkit.y + appkit.height), coordinate_space,),
            world.min
        );
        assert_eq!(
            appkit_point_to_world((appkit.x + appkit.width, appkit.y), coordinate_space),
            world.max
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
        assert!(is_protected_terminal_app(
            Some("com.openai.codex"),
            Some("Codex")
        ));
        assert!(is_protected_terminal_app(
            Some("com.microsoft.VSCode"),
            Some("Visual Studio Code")
        ));
        assert!(!is_protected_terminal_app(
            Some("com.apple.TextEdit"),
            Some("TextEdit")
        ));
    }
}
