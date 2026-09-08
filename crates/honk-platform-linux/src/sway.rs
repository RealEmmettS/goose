//! Read-only Sway observations qualified independently from KWin (ADR 0046).
//!
//! Native IPC can move a window, but its public tree/seat schema does not expose
//! an authoritative active user drag. This adapter deliberately has no movement,
//! focus, keyboard, pointer, configuration, or general command operation.
use serde::Deserialize;
use std::collections::HashSet;

pub const MAX_REPLY: usize = 256 * 1024;
pub const MAX_AGE: std::time::Duration = std::time::Duration::from_millis(250);

#[cfg(target_os = "linux")]
mod transport;
#[cfg(target_os = "linux")]
pub use transport::{Connection, Peer};
#[cfg(target_os = "linux")]
mod observer;
#[cfg(target_os = "linux")]
pub use observer::Observer;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub variant: String,
}

impl Version {
    pub fn decode(raw: &[u8]) -> Result<Self, &'static str> {
        if raw.len() > 4096 {
            return Err("Sway version reply is too large");
        }
        let version: Self = serde_json::from_slice(raw).map_err(|_| "Invalid Sway version")?;
        if version.variant != "sway"
            || !matches!(
                (version.major, version.minor, version.patch),
                (1, 9, 0) | (1, 10, 1)
            )
        {
            return Err("This Sway version has not passed native qualification");
        }
        Ok(version)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Geometry {
    fn valid(self) -> bool {
        self.x.abs_diff(0) <= 1_000_000
            && self.y.abs_diff(0) <= 1_000_000
            && (1..=1_000_000).contains(&self.width)
            && (1..=1_000_000).contains(&self.height)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub id: u64,
    pub pid: Option<u32>,
    pub title: String,
    pub app: Option<String>,
    pub geometry: Geometry,
    pub visible: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub windows: Vec<Window>,
}

impl Frame {
    pub fn fullscreen(&self) -> bool {
        self.windows
            .iter()
            .any(|window| window.visible && window.fullscreen)
    }

    pub fn decode(raw: &[u8]) -> Result<Self, &'static str> {
        if raw.len() > MAX_REPLY {
            return Err("Sway tree reply is too large");
        }
        let root: serde_json::Value =
            serde_json::from_slice(raw).map_err(|_| "Invalid Sway tree JSON")?;
        if root.get("type").and_then(|value| value.as_str()) != Some("root") {
            return Err("Missing Sway root node");
        }
        let mut stack = vec![(&root, 0, false, false)];
        let mut identities = HashSet::new();
        let mut windows = Vec::new();
        let mut count = 0;
        let mut live_outputs = 0;
        while let Some((node, depth, inherited_fullscreen, inherited_output)) = stack.pop() {
            count += 1;
            if count > 512 || depth > 16 {
                return Err("Sway tree exceeds its node or depth bound");
            }
            let id = node
                .get("id")
                .and_then(|value| value.as_u64())
                .filter(|id| *id > 0)
                .ok_or("Invalid Sway node identity")?;
            if !identities.insert(id) {
                return Err("Repeated Sway node identity");
            }
            let kind = node
                .get("type")
                .and_then(|value| value.as_str())
                .ok_or("Missing Sway node type")?;
            if !matches!(
                kind,
                "root" | "output" | "workspace" | "con" | "floating_con"
            ) {
                return Err("Unknown Sway node type");
            }
            let live_output = if kind == "output" {
                // Sway includes i3's synthetic scratchpad output in GET_TREE.
                // It is not a real monitor and has no active/power fields.
                if id == i32::MAX as u64
                    && node.get("name").and_then(|value| value.as_str()) == Some("__i3")
                {
                    false
                } else {
                    let active = node
                        .get("active")
                        .and_then(|value| value.as_bool())
                        .ok_or("Missing Sway output activation state")?;
                    let powered = node
                        .get("dpms")
                        .and_then(|value| value.as_bool())
                        .ok_or("Missing Sway output power state")?;
                    if active && powered {
                        live_outputs += 1;
                    }
                    if live_outputs > 16 {
                        return Err("Too many active Sway outputs");
                    }
                    active && powered
                }
            } else {
                inherited_output
            };
            // Workspace fullscreen_mode is always 1 for i3 compatibility; only
            // real containers carry fullscreen authority, inherited by leaves.
            let own_fullscreen = if matches!(kind, "con" | "floating_con") {
                node.get("fullscreen_mode")
                    .and_then(|value| value.as_u64())
                    .filter(|mode| *mode <= 2)
                    .ok_or("Invalid Sway fullscreen mode")?
                    != 0
            } else {
                false
            };
            let fullscreen = inherited_fullscreen || own_fullscreen;
            for key in ["nodes", "floating_nodes"] {
                let children = node
                    .get(key)
                    .and_then(|value| value.as_array())
                    .ok_or("Missing Sway node children")?;
                if children.len() + stack.len() + count > 512 {
                    return Err("Too many Sway child nodes");
                }
                stack.extend(
                    children
                        .iter()
                        .map(|child| (child, depth + 1, fullscreen, live_output)),
                );
            }
            // Structural containers omit pid. Real views can explicitly report
            // null (notably XWayland clients without _NET_WM_PID); their visible
            // fullscreen state is still authoritative compositor information.
            let Some(raw_pid) = node.get("pid") else {
                continue;
            };
            let pid = match raw_pid {
                serde_json::Value::Null => None,
                value => Some(
                    value
                        .as_u64()
                        .filter(|pid| *pid > 0 && *pid <= u32::MAX as u64)
                        .ok_or("Invalid Sway window process")? as u32,
                ),
            };
            if !matches!(kind, "con" | "floating_con") || windows.len() >= 64 {
                return Err("Invalid or excessive Sway window inventory");
            }
            let title = match node.get("name") {
                Some(serde_json::Value::Null) => "",
                Some(serde_json::Value::String(title)) if title.len() <= 1024 => title,
                _ => return Err("Invalid Sway window title"),
            };
            let app = match node.get("app_id") {
                Some(serde_json::Value::String(app)) if app.len() <= 1024 => Some(app.clone()),
                Some(serde_json::Value::Null) => None,
                _ => return Err("Invalid Sway application identity"),
            };
            let geometry: Geometry = serde_json::from_value(
                node.get("rect")
                    .ok_or("Missing Sway window geometry")?
                    .clone(),
            )
            .map_err(|_| "Invalid Sway window geometry")?;
            if !geometry.valid() {
                return Err("Sway window geometry exceeds its bounds");
            }
            let visible = node
                .get("visible")
                .and_then(|value| value.as_bool())
                .ok_or("Missing Sway window visibility")?
                && live_output;
            windows.push(Window {
                id,
                pid,
                title: title.to_owned(),
                app,
                geometry,
                visible,
                fullscreen,
            });
        }
        if live_outputs == 0 {
            return Err("No active Sway output is available");
        }
        Ok(Self { windows })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tree() -> serde_json::Value {
        // The actual native Sway premise supplies these node/window fields.
        json!({"id":1,"type":"root","floating_nodes":[],"nodes":[{
            "id":2,"type":"output","name":"HEADLESS-1","active":true,"dpms":true,
            "rect":{"x":0,"y":0,"width":1280,"height":900},"floating_nodes":[],"nodes":[{
            "id":3,"type":"workspace","fullscreen_mode":1,"nodes":[],"floating_nodes":[{
            "id":6,"type":"floating_con","nodes":[],"floating_nodes":[],
            "pid":2489,"app_id":"honk300-sway-probe","name":"Honk300 ordinary Sway probe",
            "rect":{"x":490,"y":350,"width":300,"height":200},
            "visible":true,"fullscreen_mode":0}]}]}]})
    }

    fn window_mut(input: &mut serde_json::Value) -> &mut serde_json::Value {
        &mut input["nodes"][0]["nodes"][0]["floating_nodes"][0]
    }

    fn decode(value: &serde_json::Value) -> Result<Frame, &'static str> {
        Frame::decode(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn native_window_visibility_and_fullscreen_remain_separate() {
        let mut input = tree();
        let frame = decode(&input).unwrap();
        assert_eq!(frame.windows[0].pid, Some(2489));
        assert!(!frame.fullscreen());
        window_mut(&mut input)["fullscreen_mode"] = json!(2);
        assert!(decode(&input).unwrap().fullscreen());
        window_mut(&mut input)["visible"] = json!(false);
        assert!(!decode(&input).unwrap().fullscreen());
    }

    #[test]
    fn untitled_native_window_keeps_observation_alive() {
        let mut input = tree();
        window_mut(&mut input)["name"] = serde_json::Value::Null;
        let frame = decode(&input).unwrap();
        assert_eq!(frame.windows.len(), 1);
        assert!(frame.windows[0].title.is_empty());
        window_mut(&mut input)["name"] = json!(72);
        assert!(decode(&input).is_err());
    }

    #[test]
    fn fullscreen_without_a_client_pid_still_pauses_manners() {
        let mut input = tree();
        window_mut(&mut input)["pid"] = serde_json::Value::Null;
        window_mut(&mut input)["app_id"] = serde_json::Value::Null;
        window_mut(&mut input)["fullscreen_mode"] = json!(1);
        let frame = decode(&input).unwrap();
        assert!(frame.fullscreen());
        assert_eq!(frame.windows.len(), 1);
        assert_eq!(frame.windows[0].pid, None);
        window_mut(&mut input)["visible"] = json!(false);
        assert!(!decode(&input).unwrap().fullscreen());
    }

    #[test]
    fn malformed_ambiguous_or_oversized_inventory_fails_closed() {
        for field in ["pid", "visible", "rect", "fullscreen_mode", "app_id"] {
            let mut input = tree();
            window_mut(&mut input)[field] = if field == "app_id" {
                json!(7)
            } else {
                json!("invalid")
            };
            assert!(decode(&input).is_err(), "{field}");
        }
        let mut duplicate = tree();
        window_mut(&mut duplicate)["id"] = json!(1);
        assert!(decode(&duplicate).is_err());
        let mut excessive = tree();
        let node = window_mut(&mut excessive).clone();
        excessive["nodes"][0]["nodes"][0]["floating_nodes"] = json!((0..65)
            .map(|i| {
                let mut copy = node.clone();
                copy["id"] = json!(i + 6);
                copy
            })
            .collect::<Vec<_>>());
        assert!(decode(&excessive).is_err());
        assert!(Frame::decode(&vec![b' '; MAX_REPLY + 1]).is_err());
    }

    #[test]
    fn split_container_fullscreen_reaches_visible_descendants_only() {
        let mut input = tree();
        let window = window_mut(&mut input).take();
        *window_mut(&mut input) = json!({"id":5,"type":"con","fullscreen_mode":1,
            "floating_nodes":[],"nodes":[window]});
        assert!(decode(&input).unwrap().fullscreen());
        window_mut(&mut input)["nodes"][0]["visible"] = json!(false);
        assert!(!decode(&input).unwrap().fullscreen());
    }

    #[test]
    fn missing_or_powered_off_outputs_are_unknown_not_a_clear_desktop() {
        let mut input = tree();
        input["nodes"][0]["active"] = json!(false);
        assert!(decode(&input).is_err());
        input["nodes"][0]["active"] = json!(true);
        input["nodes"][0]["dpms"] = json!(false);
        assert!(decode(&input).is_err());
        input["nodes"] = json!([]);
        assert!(decode(&input).is_err());
    }

    #[test]
    fn only_native_qualified_versions_are_accepted() {
        for (major, minor, patch, accepted) in [
            (1, 9, 0, true),
            (1, 10, 1, true),
            (1, 10, 2, false),
            (1, 11, 0, false),
        ] {
            let raw = serde_json::to_vec(
                &json!({"variant":"sway","major":major,"minor":minor,"patch":patch}),
            )
            .unwrap();
            assert_eq!(Version::decode(&raw).is_ok(), accepted);
        }
    }
}
