//! Explicit GNOME Shell observations; no foreign-window or pointer commands.
use serde::Deserialize;
use std::collections::HashSet;
use std::time::Duration;

pub const MAX_AGE: Duration = Duration::from_millis(250);
pub const MAX_REPLY: usize = 65_536;
pub const BOUNDARY: &str = "gnome-observe-1";

#[cfg(target_os = "linux")]
mod transport;
#[cfg(target_os = "linux")]
pub use transport::{Connection, Observer};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub id: u32,
    pub pid: Option<u32>,
    pub app: Option<String>,
    pub title: Option<String>,
    pub geometry: [i32; 4],
    pub normal: bool,
    pub visible: bool,
    pub fullscreen: bool,
    pub dragging: bool,
}

impl Window {
    pub fn eligible(&self) -> bool {
        self.pid.is_some_and(|pid| pid > 0)
            && self.normal
            && self.visible
            && !self.fullscreen
            && self.app.as_ref().is_some_and(|app| !app.trim().is_empty())
            && self
                .title
                .as_ref()
                .is_some_and(|title| !title.trim().is_empty())
            && !super::is_protected_terminal_app(self.app.as_deref(), self.title.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub boundary: String,
    pub build: String,
    pub version: String,
    pub pid: u32,
    pub session_wayland: bool,
    pub grabbed: bool,
    pub overview: bool,
    pub windows: Vec<Window>,
}

impl Frame {
    pub fn decode(raw: &[u8], pid: u32, build: &str) -> Result<Self, &'static str> {
        if raw.len() > MAX_REPLY {
            return Err("GNOME observation exceeds its byte bound");
        }
        let frame: Self = serde_json::from_slice(raw).map_err(|_| "Invalid GNOME observation")?;
        if frame.boundary != BOUNDARY
            || frame.build != build
            || build.len() != 64
            || !build.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !matches!(frame.version.as_str(), "46.0" | "48.7")
            || frame.pid != pid
            || pid == 0
            || !frame.session_wayland
            || frame.windows.len() > 64
        {
            return Err("GNOME observation has an unqualified owner, version or boundary");
        }
        let mut identities = HashSet::new();
        let mut dragging = 0;
        for window in &frame.windows {
            if window.id == 0
                || !identities.insert(window.id)
                || window.pid == Some(0)
                || window.app.as_ref().is_some_and(|text| text.len() > 1024)
                || window.title.as_ref().is_some_and(|text| text.len() > 1024)
                || window.geometry[..2]
                    .iter()
                    .any(|v| v.abs_diff(0) > 1_000_000)
                || window.geometry[2..]
                    .iter()
                    .any(|v| !(1..=1_000_000).contains(v))
                || (window.dragging && !frame.grabbed)
            {
                return Err("Invalid GNOME window identity, bounds or grab state");
            }
            dragging += u8::from(window.dragging);
        }
        if dragging > 1 {
            return Err("Ambiguous GNOME user drag");
        }
        Ok(frame)
    }

    pub fn fullscreen(&self) -> bool {
        self.windows
            .iter()
            .any(|window| window.visible && window.fullscreen)
    }

    pub fn dragged_window(&self) -> Option<&Window> {
        if !self.grabbed || self.overview {
            return None;
        }
        self.windows
            .iter()
            .find(|window| window.dragging && window.eligible())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn native() -> Value {
        json!({"boundary": BOUNDARY, "build": "c".repeat(64), "version": "46.0", "pid": 500,
            "session_wayland": true, "grabbed": true, "overview": false,
            "windows": [{"id": 1, "pid": 600, "app": "org.example.Editor",
                "title": "An ordinary document", "geometry": [-300, 20, 250, 200],
                "normal": true, "visible": true, "fullscreen": false, "dragging": true}]})
    }
    fn decode(value: &Value) -> Result<Frame, &'static str> {
        Frame::decode(&serde_json::to_vec(value).unwrap(), 500, &"c".repeat(64))
    }
    #[test]
    fn native_drag_requires_live_eligible_identity_and_clear_overview() {
        let mut value = native();
        assert_eq!(decode(&value).unwrap().dragged_window().unwrap().id, 1);
        for (key, replacement) in [
            ("pid", Value::Null),
            ("title", json!("ChatGPT")),
            ("app", json!("code")),
            ("visible", json!(false)),
            ("normal", json!(false)),
            ("fullscreen", json!(true)),
        ] {
            let mut changed = value.clone();
            changed["windows"][0][key] = replacement;
            assert!(decode(&changed).unwrap().dragged_window().is_none());
        }
        value["overview"] = json!(true);
        assert!(decode(&value).unwrap().dragged_window().is_none());
    }
    #[test]
    fn optional_identity_does_not_erase_known_fullscreen() {
        let mut value = native();
        value["windows"][0]["pid"] = Value::Null;
        value["windows"][0]["title"] = Value::Null;
        value["windows"][0]["fullscreen"] = json!(true);
        assert!(decode(&value).unwrap().fullscreen());
        value["windows"][0]["visible"] = json!(false);
        assert!(!decode(&value).unwrap().fullscreen());
    }
    #[test]
    fn malformed_replaced_or_ambiguous_observations_fail_closed() {
        for (key, replacement) in [
            ("build", json!("d".repeat(64))),
            ("pid", json!(501)),
            ("version", json!("49.0")),
            ("boundary", json!("unknown")),
            ("session_wayland", json!(false)),
            ("grabbed", json!(false)),
        ] {
            let mut value = native();
            value[key] = replacement;
            assert!(decode(&value).is_err());
        }
        let mut value = native();
        value["windows"]
            .as_array_mut()
            .unwrap()
            .push(native()["windows"][0].clone());
        assert!(decode(&value).is_err());
        value["windows"][1]["id"] = json!(2);
        assert!(decode(&value).is_err());
        let mut raw = serde_json::to_vec(&native()).unwrap();
        raw.extend_from_slice(b"{}");
        assert!(Frame::decode(&raw, 500, &"c".repeat(64)).is_err());
        assert!(Frame::decode(&vec![b' '; MAX_REPLY + 1], 500, &"c".repeat(64)).is_err());
    }
}
