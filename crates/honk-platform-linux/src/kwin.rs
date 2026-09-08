//! Bounded, fresh observations from an explicitly enabled KWin companion (ADR 0044).
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, Instant};

pub const MAX_AGE: Duration = Duration::from_millis(250);
const MAX_FRAME: usize = 65_536;

#[cfg(target_os = "linux")]
mod transport;
#[cfg(target_os = "linux")]
pub use transport::Bridge;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub id: String,
    pub pid: u32,
    pub app: String,
    pub title: String,
    pub geometry: [f64; 4],
    pub area: [f64; 4],
    pub normal: bool,
    pub deleted: bool,
    pub minimized: bool,
    pub fullscreen: bool,
    pub active: bool,
    pub moveable: bool,
    pub dragging: bool,
    pub drag_known: bool,
    pub on_desktop: bool,
    pub on_activity: bool,
    pub protected: bool,
}

impl Window {
    fn valid(&self) -> bool {
        let identity = self.id.trim_matches(['{', '}']);
        identity.len() == 36
            && identity.chars().enumerate().all(|(i, ch)| {
                if [8, 13, 18, 23].contains(&i) {
                    ch == '-'
                } else {
                    ch.is_ascii_hexdigit()
                }
            })
            && self.pid > 0
            && self.app.len() <= 1024
            && self.title.len() <= 1024
            && good_rect(self.geometry)
            && good_rect(self.area)
    }

    pub fn eligible(&self) -> bool {
        self.valid()
            && self.normal
            && !self.deleted
            && !self.minimized
            && !self.fullscreen
            && self.on_desktop
            && self.on_activity
            && self.moveable
            && !self.protected
            && !self.app.trim().is_empty()
            && !self.title.trim().is_empty()
            && !super::is_protected_terminal_app(Some(&self.app), Some(&self.title))
    }

    fn permits_move(&self, to: [f64; 2]) -> bool {
        let [x, y, width, height] = self.geometry;
        let [left, top, area_width, area_height] = self.area;
        self.eligible()
            && !self.dragging
            && to.into_iter().all(finite)
            && (to[0] - x).powi(2) + (to[1] - y).powi(2) <= 24.0 * 24.0
            && to[0] >= left
            && to[1] >= top
            && to[0] + width <= left + area_width
            && to[1] + height <= top + area_height
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub protocol: u8,
    pub sequence: u64,
    pub windows: Vec<Window>,
    pub stacking_order: bool,
    pub pointer: [f64; 2],
    pub result: String,
}

impl Frame {
    /// Bind a live window to an unreaped, caller-owned prop process and opaque id.
    /// Matching an application name alone never establishes ownership.
    pub fn owned_prop(&self, pid: u32, id: u64, size: [f64; 2]) -> Option<&Window> {
        let app = format!("honk300.prop.{id}");
        let mut matches = self.windows.iter().filter(|window| {
            window.pid == pid
                && window.app == app
                && window.eligible()
                && window.drag_known
                && !window.dragging
                && window.geometry[2..] == size
        });
        let window = matches.next()?;
        matches.next().is_none().then_some(window)
    }

    /// Conservative pointer exclusion requires a complete compositor ordering.
    /// Even an occluded protected window rejects an intersecting movement path.
    pub fn permits_pointer_motion(&self, target: [f64; 2]) -> bool {
        if !self.stacking_order
            || !self.pointer.into_iter().all(finite)
            || !target.into_iter().all(finite)
            || (self.pointer[0] - target[0]).powi(2) + (self.pointer[1] - target[1]).powi(2)
                > 24.0 * 24.0
        {
            return false;
        }
        let left = self.pointer[0].min(target[0]);
        let right = self.pointer[0].max(target[0]);
        let top = self.pointer[1].min(target[1]);
        let bottom = self.pointer[1].max(target[1]);
        self.windows.iter().all(|window| {
            if window.deleted || window.minimized || !window.on_desktop || !window.on_activity {
                return true;
            }
            if !window.valid() || window.dragging || !window.drag_known {
                return false;
            }
            // KWin includes our click-through layer surface in its window list
            // with an empty caption. Its live native PID establishes ownership;
            // an application-name lookalike never exempts a foreign window.
            // Keep checking every window beneath this transparent surface.
            if window.pid == std::process::id()
                && window.app == "honk300"
                && window.title.is_empty()
                && !window.moveable
                && !window.fullscreen
            {
                return true;
            }
            let [x, y, width, height] = window.geometry;
            let intersects = right >= x && left <= x + width && bottom >= y && top <= y + height;
            !intersects
                || (!window.fullscreen
                    && !window.protected
                    && !window.app.trim().is_empty()
                    && !window.title.trim().is_empty()
                    && !super::is_protected_terminal_app(Some(&window.app), Some(&window.title)))
        })
    }

    /// Fullscreen observation does not imply do-not-disturb observation.
    pub fn fullscreen(&self) -> bool {
        self.windows.iter().any(|window| {
            window.on_desktop
                && window.on_activity
                && !window.deleted
                && !window.minimized
                && window.fullscreen
        })
    }

    /// A single live user drag can supply a ride anchor without pointer injection.
    pub fn dragged_window(&self) -> Option<&Window> {
        let mut targets = self
            .windows
            .iter()
            .filter(|window| window.eligible() && window.drag_known && window.dragging);
        let target = targets.next()?;
        targets.next().is_none().then_some(target)
    }

    fn decode(raw: &str) -> Result<Self, &'static str> {
        if raw.len() > MAX_FRAME {
            return Err("oversized KWin frame");
        }
        let frame: Self = serde_json::from_str(raw).map_err(|_| "invalid KWin frame")?;
        if frame.protocol != 1
            || frame.sequence == 0
            || frame.windows.len() > 64
            || !frame.pointer.into_iter().all(finite)
            || frame.result.len() > 32
            || !frame.windows.iter().all(Window::valid)
        {
            return Err("invalid KWin snapshot");
        }
        let mut ids = HashSet::new();
        if !frame.windows.iter().all(|window| ids.insert(&window.id)) {
            return Err("duplicate KWin identity");
        }
        Ok(frame)
    }
}

fn finite(value: f64) -> bool {
    value.is_finite() && value.abs() <= 1_000_000.0
}
fn good_rect(value: [f64; 4]) -> bool {
    value.into_iter().all(finite) && value[2] > 0.0 && value[3] > 0.0
}

#[derive(Debug, Serialize)]
struct Move {
    kind: &'static str,
    id: String,
    pid: u32,
    app: String,
    from: [f64; 4],
    to: [f64; 2],
}

#[derive(Serialize)]
struct Reply {
    protocol: u8,
    sequence: u64,
    commands: Vec<Move>,
    stop: bool,
}

struct State {
    owner: String,
    frame: Option<(Instant, Frame)>,
    pending: Option<(Instant, Window, [f64; 2])>,
    sequence: u64,
    stopped: bool,
}

impl State {
    fn new(owner: String) -> Self {
        Self {
            owner,
            frame: None,
            pending: None,
            sequence: 0,
            stopped: false,
        }
    }

    fn stop(&mut self) {
        self.stopped = true;
        self.frame = None;
        self.pending = None;
    }

    fn snapshot(&self, now: Instant) -> Option<&Frame> {
        self.frame
            .as_ref()
            .filter(|(received, _)| {
                !self.stopped && now.saturating_duration_since(*received) < MAX_AGE
            })
            .map(|(_, frame)| frame)
    }

    fn queue_move(
        &mut self,
        window: &Window,
        to: [f64; 2],
        now: Instant,
    ) -> Result<(), &'static str> {
        let frame = self.snapshot(now).ok_or("KWin observation unavailable")?;
        if !window.permits_move(to) || !frame.windows.contains(window) {
            return Err("KWin target is protected, stale or out of bounds");
        }
        // The runtime keeps only its latest desired move; an old action never accumulates.
        self.pending = Some((now, window.clone(), to));
        Ok(())
    }

    fn exchange(
        &mut self,
        sender: &str,
        live_owner: &str,
        raw: &str,
        now: Instant,
    ) -> Result<String, &'static str> {
        if sender != self.owner {
            return Err("untrusted KWin sender");
        }
        if live_owner != self.owner {
            self.stop();
            return Err("KWin owner changed");
        }
        let frame = match Frame::decode(raw) {
            Ok(frame) if frame.sequence > self.sequence => frame,
            _ => {
                self.stop();
                return Err("invalid or repeated KWin snapshot");
            }
        };
        let mut commands = Vec::new();
        if !self.stopped && self.snapshot(now).is_some() {
            if let Some((queued, window, to)) = self.pending.take() {
                if now.saturating_duration_since(queued) < MAX_AGE
                    && window.permits_move(to)
                    && frame.windows.contains(&window)
                {
                    commands.push(Move {
                        kind: "move",
                        id: window.id,
                        pid: window.pid,
                        app: window.app,
                        from: window.geometry,
                        to,
                    });
                }
            }
        }
        self.pending = None;
        self.sequence = frame.sequence;
        let response = serde_json::to_string(&Reply {
            protocol: 1,
            sequence: frame.sequence,
            commands,
            stop: self.stopped,
        })
        .map_err(|_| "KWin reply encoding failed")?;
        if response.len() > 4096 {
            self.stop();
            return Err("KWin reply exceeded bound");
        }
        if !self.stopped {
            self.frame = Some((now, frame));
        }
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Frame {
        serde_json::from_str(r#"{"protocol":1,"sequence":1,"windows":[{"id":"{45316fac-9956-4bd3-adcd-fe8c1ecdf5cc}","pid":4307,"app":"honk300-native-probe","title":"Honk300 ordinary probe","geometry":[100,100,300,200],"area":[0,0,1280,900],"normal":true,"deleted":false,"minimized":false,"fullscreen":false,"active":true,"moveable":true,"dragging":false,"drag_known":true,"on_desktop":true,"on_activity":true,"protected":false}],"stacking_order":true,"pointer":[639,449],"result":"none"}"#).unwrap()
    }
    fn raw(frame: &Frame) -> String {
        serde_json::to_string(frame).unwrap()
    }
    #[test]
    fn owned_props_require_unique_process_token_geometry_and_live_eligibility() {
        let mut frame = fixture();
        frame.windows[0].app = "honk300.prop.7".into();
        assert!(frame.owned_prop(4307, 7, [300.0, 200.0]).is_some());
        assert!(frame.owned_prop(4308, 7, [300.0, 200.0]).is_none());
        assert!(frame.owned_prop(4307, 8, [300.0, 200.0]).is_none());
        assert!(frame.owned_prop(4307, 7, [301.0, 200.0]).is_none());
        for title in ["ChatGPT", "Konsole", ""] {
            frame.windows[0].title = title.into();
            assert!(frame.owned_prop(4307, 7, [300.0, 200.0]).is_none());
        }
        frame.windows[0].title = "Honk300 note".into();
        frame.windows[0].dragging = true;
        assert!(frame.owned_prop(4307, 7, [300.0, 200.0]).is_none());
        frame.windows[0].dragging = false;
        frame.windows.push(frame.windows[0].clone());
        assert!(frame.owned_prop(4307, 7, [300.0, 200.0]).is_none());
        frame.windows.clear();
        assert!(frame.owned_prop(4307, 7, [300.0, 200.0]).is_none());
    }
    #[test]
    fn pointer_motion_requires_bounded_complete_safe_observations() {
        let mut frame = fixture();
        frame.pointer = [110.0, 110.0];
        assert!(frame.permits_pointer_motion([116.0, 110.0]));
        assert!(!frame.permits_pointer_motion([140.0, 110.0]));
        assert!(!frame.permits_pointer_motion([f64::NAN, 110.0]));
        frame.stacking_order = false;
        assert!(!frame.permits_pointer_motion([116.0, 110.0]));
        frame.stacking_order = true;
        for title in ["ChatGPT", "Konsole", "Terminal", ""] {
            frame.windows[0].title = title.into();
            assert!(!frame.permits_pointer_motion([116.0, 110.0]));
        }
        frame.windows[0].title = "Ordinary document".into();
        frame.windows[0].dragging = true;
        assert!(!frame.permits_pointer_motion([116.0, 110.0]));
        frame.windows[0].dragging = false;
        frame.windows[0].fullscreen = true;
        assert!(!frame.permits_pointer_motion([116.0, 110.0]));
    }

    #[test]
    fn pointer_guard_recognizes_only_this_process_overlay_and_preserves_windows_below() {
        let mut frame = fixture();
        let mut overlay = frame.windows[0].clone();
        overlay.id = "{efec6f86-5174-4ff5-a84b-431aade70d25}".into();
        overlay.pid = std::process::id();
        overlay.app = "honk300".into();
        overlay.title.clear();
        overlay.geometry = [0.0, 0.0, 1280.0, 900.0];
        overlay.moveable = false;
        overlay.protected = true;
        frame.windows.push(overlay.clone());
        assert!(frame.permits_pointer_motion([645.0, 449.0]));
        for foreign in [true, false] {
            frame.windows[1] = overlay.clone();
            if foreign {
                frame.windows[1].pid = overlay.pid.checked_add(1).unwrap();
            } else {
                frame.windows[1].app = "another-app".into();
            }
            assert!(!frame.permits_pointer_motion([645.0, 449.0]));
        }
        frame.windows[1] = overlay;
        frame.windows[0].geometry = [630.0, 440.0, 100.0, 100.0];
        frame.windows[0].title = "ChatGPT Codex".into();
        assert!(!frame.permits_pointer_motion([645.0, 449.0]));
        frame.windows[0].title.clear();
        assert!(!frame.permits_pointer_motion([645.0, 449.0]));
    }

    #[test]
    fn live_desktop_presence_and_user_drag_remain_separate_from_movement() {
        let mut frame = fixture();
        frame.windows[0].dragging = true;
        assert_eq!(frame.dragged_window(), Some(&frame.windows[0]));
        assert!(!frame.windows[0].permits_move([110.0, 100.0]));
        frame.windows[0].drag_known = false;
        assert!(frame.dragged_window().is_none());
        frame.windows[0].drag_known = true;
        frame.windows[0].on_desktop = false;
        assert!(frame.dragged_window().is_none());
        frame.windows[0].fullscreen = true;
        assert!(!frame.fullscreen());
        frame.windows[0].on_desktop = true;
        assert!(frame.fullscreen());
        assert!(frame.dragged_window().is_none());
        frame.windows[0].on_activity = false;
        assert!(!frame.fullscreen());
        frame.windows[0].on_activity = true;
        frame.windows[0].fullscreen = false;
        frame.windows[0].title = "ChatGPT".into();
        assert!(frame.dragged_window().is_none());
    }

    fn receive(state: &mut State, frame: &Frame, now: Instant) -> serde_json::Value {
        serde_json::from_str(&state.exchange(":1.10", ":1.10", &raw(frame), now).unwrap()).unwrap()
    }

    #[test]
    fn native_frame_drives_a_single_bounded_move() {
        let now = Instant::now();
        let mut state = State::new(":1.10".into());
        let mut frame = fixture();
        receive(&mut state, &frame, now);
        state
            .queue_move(&frame.windows[0], [110.0, 100.0], now)
            .unwrap();
        frame.sequence += 1;
        let reply = receive(&mut state, &frame, now + Duration::from_millis(50));
        assert_eq!(reply["commands"].as_array().unwrap().len(), 1);
        assert_eq!(
            reply["commands"][0]["to"],
            serde_json::json!([110.0, 100.0])
        );
        frame.sequence += 1;
        assert_eq!(
            receive(&mut state, &frame, now)["commands"],
            serde_json::json!([])
        );
    }

    #[test]
    fn expiry_and_changed_identity_discard_queued_actions() {
        for mode in 0..10 {
            let now = Instant::now();
            let mut state = State::new(":1.10".into());
            let mut frame = fixture();
            receive(&mut state, &frame, now);
            state
                .queue_move(&frame.windows[0], [110.0, 100.0], now)
                .unwrap();
            frame.sequence += 1;
            match mode {
                1 => frame.windows[0].title = "ChatGPT".into(),
                2 => frame.windows[0].pid += 1,
                3 => frame.windows[0].geometry[0] += 1.0,
                4 => frame.windows.clear(),
                5 => frame.windows[0].area[2] = 50.0,
                6 => frame.windows[0].on_desktop = false,
                7 => frame.windows[0].on_activity = false,
                8 => frame.windows[0].dragging = true,
                9 => frame.windows[0].fullscreen = true,
                _ => {}
            }
            let later = now
                + if mode == 0 {
                    MAX_AGE
                } else {
                    Duration::from_millis(50)
                };
            if mode == 0 {
                assert!(state.snapshot(later).is_none());
            }
            assert_eq!(
                receive(&mut state, &frame, later)["commands"],
                serde_json::json!([])
            );
        }
    }

    #[test]
    fn protected_and_oversized_moves_cannot_be_queued() {
        let now = Instant::now();
        for title in ["ChatGPT", "Code", "Konsole", "Terminal", ""] {
            let mut frame = fixture();
            frame.windows[0].title = title.into();
            let mut state = State::new(":1.10".into());
            receive(&mut state, &frame, now);
            assert!(state
                .queue_move(&frame.windows[0], [110.0, 100.0], now)
                .is_err());
        }
        let frame = fixture();
        let mut state = State::new(":1.10".into());
        receive(&mut state, &frame, now);
        assert!(state
            .queue_move(&frame.windows[0], [125.0, 100.0], now)
            .is_err());
        assert!(state
            .queue_move(&frame.windows[0], [f64::NAN, 100.0], now)
            .is_err());
    }

    #[test]
    fn untrusted_peer_cannot_inject_or_revoke_a_valid_snapshot() {
        let now = Instant::now();
        let mut state = State::new(":1.10".into());
        let frame = fixture();
        receive(&mut state, &frame, now);
        assert!(state.exchange(":1.20", ":1.10", &raw(&frame), now).is_err());
        assert!(state.snapshot(now).is_some());
        assert!(state.exchange(":1.10", ":1.30", &raw(&frame), now).is_err());
        assert!(state.snapshot(now).is_none());
    }

    #[test]
    fn malformed_duplicate_and_replayed_frames_fail_closed() {
        let now = Instant::now();
        let mut duplicate = fixture();
        duplicate.windows.push(duplicate.windows[0].clone());
        let mut unknown = serde_json::to_value(fixture()).unwrap();
        unknown["arbitrary"] = true.into();
        for invalid in [
            "{".into(),
            " ".repeat(MAX_FRAME + 1),
            raw(&duplicate),
            unknown.to_string(),
            raw(&fixture()),
        ] {
            let mut state = State::new(":1.10".into());
            let frame = fixture();
            receive(&mut state, &frame, now);
            state
                .queue_move(&frame.windows[0], [110.0, 100.0], now)
                .unwrap();
            assert!(state.exchange(":1.10", ":1.10", &invalid, now).is_err());
            assert!(state.snapshot(now).is_none());
            assert!(state.pending.is_none());
        }
    }

    #[test]
    fn explicit_stop_never_restores_an_action_from_a_later_frame() {
        let now = Instant::now();
        let mut state = State::new(":1.10".into());
        let mut frame = fixture();
        receive(&mut state, &frame, now);
        state
            .queue_move(&frame.windows[0], [110.0, 100.0], now)
            .unwrap();
        state.stop();
        frame.sequence += 1;
        let reply = receive(&mut state, &frame, now);
        assert_eq!(reply["stop"], true);
        assert_eq!(reply["commands"], serde_json::json!([]));
        assert!(state.snapshot(now).is_none());
    }
}
