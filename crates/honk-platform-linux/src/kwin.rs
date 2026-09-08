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
    pub pointer: [f64; 2],
    pub result: String,
}

impl Frame {
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
        serde_json::from_str(r#"{"protocol":1,"sequence":1,"windows":[{"id":"{45316fac-9956-4bd3-adcd-fe8c1ecdf5cc}","pid":4307,"app":"honk300-native-probe","title":"Honk300 ordinary probe","geometry":[100,100,300,200],"area":[0,0,1280,900],"normal":true,"deleted":false,"minimized":false,"fullscreen":false,"active":true,"moveable":true,"dragging":false,"protected":false}],"pointer":[639,449],"result":"none"}"#).unwrap()
    }
    fn raw(frame: &Frame) -> String {
        serde_json::to_string(frame).unwrap()
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
        for mode in 0..6 {
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
