//! Independently qualified, read-only Hyprland observations (ADR 0047).
//! Public IPC has no authoritative active user-drag observation, so this
//! boundary exposes no movement, focus, pointer, or general command operation.
use serde::Deserialize;
use std::collections::HashSet;
pub const MAX_REPLY: usize = 256 * 1024;
pub const MAX_AGE: std::time::Duration = std::time::Duration::from_millis(250);

#[cfg(target_os = "linux")]
mod transport;
#[cfg(target_os = "linux")]
pub use transport::{Connection, Peer};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Version {
    pub version: String,
    pub commit: String,
}
impl Version {
    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > 4096 {
            return Err("Hyprland version reply is too large");
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| "Invalid Hyprland version")?;
        if !matches!(value.version.as_str(), "0.53.3" | "0.55.2")
            || value.commit.len() != 40
            || !value.commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("This Hyprland version has not passed native qualification");
        }
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Workspace {
    id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    id: i64,
    active_workspace: Workspace,
    special_workspace: Workspace,
    dpms_status: bool,
    disabled: bool,
}
impl Monitor {
    pub fn decode(bytes: &[u8]) -> Result<Vec<Self>, &'static str> {
        if bytes.len() > MAX_REPLY {
            return Err("Hyprland monitor reply is too large");
        }
        let monitors: Vec<Self> =
            serde_json::from_slice(bytes).map_err(|_| "Invalid Hyprland monitors")?;
        Self::validate(&monitors)?;
        Ok(monitors)
    }
    fn validate(monitors: &[Self]) -> Result<(), &'static str> {
        let mut ids = HashSet::new();
        if monitors.is_empty()
            || monitors.len() > 16
            || monitors
                .iter()
                .any(|monitor| monitor.id < 0 || !ids.insert(monitor.id))
        {
            return Err("Invalid Hyprland monitor inventory");
        }
        if !monitors
            .iter()
            .any(|monitor| !monitor.disabled && monitor.dpms_status)
        {
            return Err("No active Hyprland output is available");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Client {
    address: String,
    pid: Option<i64>,
    title: Option<String>,
    class: Option<String>,
    at: [i32; 2],
    size: [u32; 2],
    monitor: i64,
    workspace: Workspace,
    mapped: bool,
    hidden: bool,
    visible: Option<bool>,
    pinned: bool,
    fullscreen: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub id: String,
    pub pid: Option<u32>,
    pub title: String,
    pub app: Option<String>,
    pub geometry: [i64; 4],
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
    pub fn decode(
        version: &Version,
        monitors: &[Monitor],
        bytes: &[u8],
    ) -> Result<Self, &'static str> {
        if bytes.len() > MAX_REPLY {
            return Err("Hyprland client reply is too large");
        }
        let clients: Vec<Client> =
            serde_json::from_slice(bytes).map_err(|_| "Invalid Hyprland clients")?;
        Self::from_clients(version, monitors, clients)
    }
    /// The qualified compositor evaluates this fixed read-only batch in one
    /// event-loop dispatch. Parse exactly three JSON values and reject trailing
    /// data, incomplete replies and changes to the active output inventory.
    pub fn decode_snapshot(version: &Version, bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_REPLY {
            return Err("Hyprland snapshot is too large");
        }
        let mut decoder = serde_json::Deserializer::from_slice(bytes);
        let before = Vec::<Monitor>::deserialize(&mut decoder)
            .map_err(|_| "Invalid initial Hyprland monitors")?;
        let clients = Vec::<Client>::deserialize(&mut decoder)
            .map_err(|_| "Invalid Hyprland snapshot clients")?;
        let after = Vec::<Monitor>::deserialize(&mut decoder)
            .map_err(|_| "Invalid final Hyprland monitors")?;
        decoder
            .end()
            .map_err(|_| "Trailing Hyprland snapshot data")?;
        Monitor::validate(&before)?;
        Monitor::validate(&after)?;
        if before != after {
            return Err("Hyprland output state changed during observation");
        }
        Self::from_clients(version, &before, clients)
    }
    fn from_clients(
        version: &Version,
        monitors: &[Monitor],
        clients: Vec<Client>,
    ) -> Result<Self, &'static str> {
        if clients.len() > 64 {
            return Err("Too many Hyprland windows");
        }
        let mut ids = HashSet::new();
        let mut windows = Vec::new();
        for client in clients {
            let address = client
                .address
                .strip_prefix("0x")
                .ok_or("Invalid Hyprland window identity")?;
            if address.is_empty()
                || address.len() > 16
                || !address.bytes().all(|b| b.is_ascii_hexdigit())
                || u64::from_str_radix(address, 16)
                    .ok()
                    .is_none_or(|id| id == 0 || !ids.insert(id))
            {
                return Err("Invalid or repeated Hyprland window identity");
            }
            let title = client.title.unwrap_or_default();
            if title.len() > 1024
                || client.class.as_ref().is_some_and(|app| app.len() > 1024)
                || client.at.iter().any(|value| value.abs_diff(0) > 1_000_000)
                || client
                    .size
                    .iter()
                    .any(|value| !(1..=1_000_000).contains(value))
                || client.fullscreen > 3
            {
                return Err("Hyprland window fields exceed their bounds");
            }
            // Missing/zero PIDs carry no action authority but still contribute
            // compositor-owned fullscreen presence, just like untitled clients.
            let pid = match client.pid {
                None | Some(0) | Some(-1) => None,
                Some(pid) if pid > 0 && pid <= u32::MAX as i64 => Some(pid as u32),
                _ => return Err("Invalid Hyprland process identity"),
            };
            let monitor = monitors.iter().find(|monitor| monitor.id == client.monitor);
            let on_output = monitor.is_some_and(|monitor| {
                !monitor.disabled
                    && monitor.dpms_status
                    && (client.pinned
                        || client.workspace.id == monitor.active_workspace.id
                        || (monitor.special_workspace.id != 0
                            && client.workspace.id == monitor.special_workspace.id))
            });
            let native_visible = if version.version == "0.55.2" {
                client.visible.ok_or("Missing native Hyprland visibility")?
            } else {
                true
            };
            windows.push(Window {
                id: client.address,
                pid,
                title,
                app: client.class,
                geometry: [
                    client.at[0] as i64,
                    client.at[1] as i64,
                    client.size[0] as i64,
                    client.size[1] as i64,
                ],
                visible: client.mapped && !client.hidden && on_output && native_visible,
                fullscreen: client.fullscreen & 2 != 0,
            });
        }
        Ok(Self { windows })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn version() -> Version {
        Version {
            version: "0.55.2".into(),
            commit: "a".repeat(40),
        }
    }
    fn monitors() -> Vec<Monitor> {
        Monitor::decode(
            &serde_json::to_vec(&json!([{"id":1,"activeWorkspace":{"id":1},
            "specialWorkspace":{"id":0},"dpmsStatus":true,"disabled":false}]))
            .unwrap(),
        )
        .unwrap()
    }
    fn client() -> serde_json::Value {
        json!({"address":"0x1234","pid":72,"title":"Ordinary","class":"fixture",
            "at":[-120,20],"size":[300,200],"monitor":1,"workspace":{"id":1},
            "mapped":true,"hidden":false,"visible":true,"pinned":false,"fullscreen":2})
    }
    fn decode(value: serde_json::Value) -> Result<Frame, &'static str> {
        Frame::decode(
            &version(),
            &monitors(),
            &serde_json::to_vec(&value).unwrap(),
        )
    }
    #[test]
    fn visible_fullscreen_does_not_require_a_title_or_process_id() {
        let mut value = client();
        value["pid"] = json!(null);
        value["title"] = json!(null);
        let frame = decode(json!([value])).unwrap();
        assert!(frame.fullscreen());
        assert_eq!(frame.windows[0].pid, None);
        assert_eq!(frame.windows[0].geometry, [-120, 20, 300, 200]);
    }
    #[test]
    fn hidden_unmapped_other_workspace_and_maximized_are_not_fullscreen() {
        for (field, value) in [
            ("hidden", json!(true)),
            ("mapped", json!(false)),
            ("workspace", json!({"id":2})),
            ("visible", json!(false)),
            ("fullscreen", json!(1)),
        ] {
            let mut window = client();
            window[field] = value;
            assert!(!decode(json!([window])).unwrap().fullscreen(), "{field}");
        }
    }
    #[test]
    fn duplicate_malformed_oversized_and_unqualified_inputs_fail_closed() {
        assert!(decode(json!([client(), client()])).is_err());
        for (field, value) in [
            ("address", json!("0x0")),
            ("pid", json!(-200)),
            ("size", json!([0, 2])),
            ("title", json!("x".repeat(1025))),
            ("fullscreen", json!(9)),
        ] {
            let mut window = client();
            window[field] = value;
            assert!(decode(json!([window])).is_err(), "{field}");
        }
        assert!(Monitor::decode(b"[]").is_err());
        assert!(Version::decode(
            br#"{"version":"0.55.3","commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#
        )
        .is_err());
        assert!(Frame::decode(&version(), &monitors(), &vec![b' '; MAX_REPLY + 1]).is_err());
    }
    #[test]
    fn snapshot_requires_exactly_three_complete_matching_inventories() {
        let monitor = json!([{"id":1,"activeWorkspace":{"id":1},
            "specialWorkspace":{"id":0},"dpmsStatus":true,"disabled":false}]);
        let clients = json!([client()]);
        let snapshot = format!("{monitor}\n\n\n{clients}\n\n\n{monitor}");
        assert!(Frame::decode_snapshot(&version(), snapshot.as_bytes())
            .unwrap()
            .fullscreen());
        for invalid in [
            format!("{monitor}\n{clients}"),
            format!("{snapshot} []"),
            format!("{monitor}\n{clients}\n[]"),
            format!(
                "{monitor}\n{clients}\n{}",
                monitor.to_string().replace("\"id\":1", "\"id\":2")
            ),
            format!("{snapshot} garbage"),
        ] {
            assert!(Frame::decode_snapshot(&version(), invalid.as_bytes()).is_err());
        }
        assert!(Frame::decode_snapshot(&version(), &vec![b' '; MAX_REPLY + 1]).is_err());
    }
}
