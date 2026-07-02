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
