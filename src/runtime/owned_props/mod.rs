//! Bounded private transport and ownership for the Linux native prop companion.
// The wire/registry tests also run on Windows; OS process I/O remains Linux-only.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use honk_engine::collect_window::MAX_OWNED_COLLECT_WINDOWS;
use honk_engine::{
    CollectWindowCloseOrigin, CollectWindowId, CollectWindowKind, CollectWindowRequestId,
    CollectWindowSnapshot, Rect, Vec2,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::io::{self, Write};

#[cfg(target_os = "linux")]
mod process;
#[cfg(target_os = "linux")]
pub(super) use process::Controller;

const MAX_COMMAND_BYTES: usize = 4 * 1024 * 1024;
const MAX_EVENT_BYTES: usize = 512;
const MAX_QUEUED_COMMANDS: usize = 64;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Command<'a> {
    Note {
        id: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        title: &'a str,
    },
    Image {
        id: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        pixel_width: u32,
        pixel_height: u32,
        title: &'a str,
        pixels: &'a str,
    },
    Move {
        id: u64,
        x: i32,
        y: i32,
    },
    Passthrough {
        id: u64,
        passthrough: bool,
    },
    Focus {
        id: u64,
    },
    Text {
        id: u64,
        text: &'a str,
    },
    Close {
        id: u64,
    },
}

#[derive(Serialize)]
struct Envelope<'a> {
    v: u8,
    #[serde(flatten)]
    command: &'a Command<'a>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
enum Event {
    Ready {
        v: u8,
        positioning: bool,
    },
    Window {
        v: u8,
        id: u64,
        kind: u8,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        alive: bool,
        origin: Option<Origin>,
    },
    Busy {
        v: u8,
        id: u64,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Origin {
    User,
    Program,
}

struct Entry {
    request: CollectWindowRequestId,
    kind: CollectWindowKind,
    width_limit: u32,
    height_limit: u32,
    snapshot: Option<CollectWindowSnapshot>,
}

struct Registry {
    expected_positioning: bool,
    ready: bool,
    next_id: u64,
    latest_id: Option<u64>,
    busy: bool,
    entries: BTreeMap<u64, Entry>,
    closed: VecDeque<CollectWindowSnapshot>,
}

impl Registry {
    fn new(expected_positioning: bool) -> Self {
        Self {
            expected_positioning,
            ready: false,
            next_id: 1,
            latest_id: None,
            busy: false,
            entries: BTreeMap::new(),
            closed: VecDeque::new(),
        }
    }

    fn has_capacity(&self) -> bool {
        self.ready && !self.busy && self.entries.len() < MAX_OWNED_COLLECT_WINDOWS
    }

    fn contains_request(&self, request: CollectWindowRequestId) -> bool {
        self.entries.values().any(|entry| entry.request == request)
    }

    fn reserve(
        &mut self,
        request: CollectWindowRequestId,
        kind: CollectWindowKind,
        width: u32,
        height: u32,
    ) -> io::Result<Option<u64>> {
        if !self.has_capacity() {
            return Ok(None);
        }
        if width == 0
            || width > 900
            || height == 0
            || height > 700
            || self.contains_request(request)
        {
            return Err(invalid("invalid or duplicate owned prop reservation"));
        }
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or_else(|| invalid("owned prop identities exhausted"))?;
        self.entries.insert(
            id,
            Entry {
                request,
                kind,
                width_limit: width,
                height_limit: height,
                snapshot: None,
            },
        );
        self.latest_id = Some(id);
        Ok(Some(id))
    }

    fn accept(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.is_empty() || bytes.len() >= MAX_EVENT_BYTES {
            return Err(invalid("oversized or empty owned prop event"));
        }
        let event: Event =
            serde_json::from_slice(bytes).map_err(|_| invalid("invalid owned prop response"))?;
        match event {
            Event::Ready { v: 1, positioning }
                if !self.ready && positioning == self.expected_positioning =>
            {
                self.ready = true
            }
            Event::Window {
                v: 1,
                id,
                kind,
                x,
                y,
                width,
                height,
                alive,
                origin,
            } if self.ready => {
                let entry = self
                    .entries
                    .get_mut(&id)
                    .ok_or_else(|| invalid("unknown owned prop identity"))?;
                let expected_kind = match entry.kind {
                    CollectWindowKind::Note => 0,
                    CollectWindowKind::Meme => 1,
                };
                if kind != expected_kind
                    || width == 0
                    || height == 0
                    || width > entry.width_limit
                    || height > entry.height_limit
                    || !(-1_000_000..=1_000_000).contains(&x)
                    || !(-1_000_000..=1_000_000).contains(&y)
                    || (!self.expected_positioning && (x != 0 || y != 0))
                    || alive == origin.is_some()
                {
                    return Err(invalid("invalid owned prop geometry or close origin"));
                }
                let snapshot = CollectWindowSnapshot {
                    id: CollectWindowId(id),
                    request: entry.request,
                    kind: entry.kind,
                    rect: Rect::new(
                        Vec2::new(x as f32, y as f32),
                        Vec2::new(x as f32 + width as f32, y as f32 + height as f32),
                    ),
                    alive,
                    close_origin: origin.map(|origin| match origin {
                        Origin::User => CollectWindowCloseOrigin::User,
                        Origin::Program => CollectWindowCloseOrigin::Program,
                    }),
                };
                if alive {
                    entry.snapshot = Some(snapshot);
                } else {
                    if self.closed.len() >= MAX_OWNED_COLLECT_WINDOWS {
                        return Err(invalid("owned prop close queue overflow"));
                    }
                    self.closed.push_back(snapshot);
                    self.entries.remove(&id);
                    self.busy = false;
                    if self.latest_id == Some(id) {
                        self.latest_id = None;
                    }
                }
            }
            Event::Busy { v: 1, id } if self.ready => {
                if self
                    .entries
                    .get(&id)
                    .is_none_or(|entry| entry.snapshot.is_some())
                {
                    return Err(invalid("busy response did not identify a pending prop"));
                }
                if self.entries.remove(&id).is_none() {
                    return Err(invalid("unknown busy prop reservation"));
                }
                if self.latest_id == Some(id) {
                    self.latest_id = None;
                }
                self.busy = true;
            }
            _ => {
                return Err(invalid(
                    "unexpected owned prop handshake or protocol version",
                ))
            }
        }
        Ok(())
    }

    fn snapshot(&mut self) -> Option<CollectWindowSnapshot> {
        self.closed.pop_front().or_else(|| {
            self.latest_id
                .and_then(|id| self.entries.get(&id))
                .and_then(|entry| entry.snapshot)
        })
    }
}

struct Pending {
    bytes: Vec<u8>,
    offset: usize,
    movement: Option<u64>,
}

#[derive(Default)]
struct Outbox {
    commands: VecDeque<Pending>,
    bytes: usize,
}

impl Outbox {
    fn remaining(&self) -> usize {
        self.bytes - self.commands.front().map_or(0, |head| head.offset)
    }

    fn enqueue(&mut self, command: Command<'_>) -> io::Result<()> {
        let movement = if let Command::Move { id, .. } = command {
            Some(id)
        } else {
            None
        };
        let mut bytes = serde_json::to_vec(&Envelope {
            v: 1,
            command: &command,
        })?;
        bytes.push(b'\n');
        // Coalesce only an entirely unsent trailing move for the same prop.
        // Spawn/text/close and partially written commands remain ordering barriers.
        if movement.is_some()
            && self
                .commands
                .back()
                .is_some_and(|last| last.offset == 0 && last.movement == movement)
        {
            let old = self.commands.pop_back().expect("checked back");
            self.bytes -= old.bytes.len();
        }
        if self.commands.len() >= MAX_QUEUED_COMMANDS
            || bytes.len() > MAX_COMMAND_BYTES
            || self.bytes + bytes.len() > MAX_COMMAND_BYTES
        {
            return Err(invalid("owned prop command queue overflow"));
        }
        self.bytes += bytes.len();
        self.commands.push_back(Pending {
            bytes,
            offset: 0,
            movement,
        });
        Ok(())
    }

    fn flush(&mut self, writer: &mut impl Write) -> io::Result<()> {
        let mut written_this_frame = 0;
        while let Some(command) = self.commands.front_mut() {
            let end = command
                .bytes
                .len()
                .min(command.offset + 256 * 1024 - written_this_frame);
            match writer.write(&command.bytes[command.offset..end]) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "closed owned prop input",
                    ))
                }
                Ok(count) => {
                    command.offset += count;
                    written_this_frame += count;
                    if command.offset == command.bytes.len() {
                        self.bytes -= command.bytes.len();
                        self.commands.pop_front();
                    }
                    if written_this_frame >= 256 * 1024 {
                        break;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}

fn coordinates(point: Vec2) -> io::Result<(i32, i32)> {
    if !point.x.is_finite()
        || !point.y.is_finite()
        || point.x.abs() > 1_000_000.0
        || point.y.abs() > 1_000_000.0
    {
        return Err(invalid("invalid owned prop position"));
    }
    Ok((point.x.round() as i32, point.y.round() as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready(positioning: bool) -> Registry {
        let mut registry = Registry::new(positioning);
        registry
            .accept(format!(r#"{{"v":1,"event":"ready","positioning":{positioning}}}"#).as_bytes())
            .unwrap();
        registry
    }

    fn event(id: u64, alive: bool, origin: Option<&str>) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"v":1,"event":"window","id":id,"kind":0,"x":0,"y":0,"width":400,"height":250,"alive":alive,"origin":origin})).unwrap()
    }

    #[test]
    fn pending_and_live_props_share_capacity_and_old_close_preserves_current_delivery() {
        let mut registry = ready(true);
        for request in 1..=8 {
            let id = registry
                .reserve(
                    CollectWindowRequestId(request),
                    CollectWindowKind::Note,
                    400,
                    250,
                )
                .unwrap()
                .unwrap();
            registry.accept(&event(id, true, None)).unwrap();
        }
        assert!(!registry.has_capacity());
        assert_eq!(
            registry
                .reserve(CollectWindowRequestId(9), CollectWindowKind::Note, 400, 250)
                .unwrap(),
            None
        );
        registry.accept(&event(1, false, Some("user"))).unwrap();
        assert!(registry.has_capacity());
        let close = registry.snapshot().unwrap();
        assert_eq!(close.close_origin, Some(CollectWindowCloseOrigin::User));
        assert_eq!(
            registry.snapshot().unwrap().request,
            CollectWindowRequestId(8)
        );
        assert_eq!(
            registry
                .reserve(
                    CollectWindowRequestId(100),
                    CollectWindowKind::Note,
                    400,
                    250
                )
                .unwrap(),
            Some(9)
        );
    }

    #[test]
    fn wrong_identity_geometry_and_capability_never_enter_the_engine() {
        let mut registry = Registry::new(false);
        assert!(registry
            .accept(br#"{"v":1,"event":"ready","positioning":true}"#)
            .is_err());
        registry
            .accept(br#"{"v":1,"event":"ready","positioning":false}"#)
            .unwrap();
        assert!(registry
            .accept(br#"{"v":1,"event":"ready","positioning":false}"#)
            .is_err());
        registry
            .reserve(
                CollectWindowRequestId(200),
                CollectWindowKind::Note,
                400,
                250,
            )
            .unwrap();
        for invalid_event in [
            serde_json::json!({"v":1,"event":"window","id":1,"kind":0,"x":5,"y":0,"width":400,"height":250,"alive":true,"origin":null}),
            serde_json::json!({"v":1,"event":"window","id":1,"kind":0,"x":0,"y":0,"width":401,"height":250,"alive":true,"origin":null}),
            serde_json::json!({"v":1,"event":"window","id":1,"kind":0,"x":0,"y":0,"width":400,"height":250,"alive":false,"origin":null}),
            serde_json::json!({"v":1,"event":"window","id":99,"kind":0,"x":0,"y":0,"width":400,"height":250,"alive":true,"origin":null}),
            serde_json::json!({"v":1,"event":"ready","positioning":false,"window_id":123}),
        ] {
            assert!(registry
                .accept(&serde_json::to_vec(&invalid_event).unwrap())
                .is_err());
        }
        registry.accept(&event(1, true, None)).unwrap();
        assert!(registry
            .accept(br#"{"v":1,"event":"busy","id":1}"#)
            .is_err());
        assert!(registry.snapshot().unwrap().alive);
    }

    #[test]
    fn move_coalescing_keeps_text_close_order_and_partial_writes() {
        let mut outbox = Outbox::default();
        outbox
            .enqueue(Command::Move {
                id: 1,
                x: 10,
                y: 20,
            })
            .unwrap();
        outbox
            .enqueue(Command::Move {
                id: 1,
                x: 30,
                y: 40,
            })
            .unwrap();
        outbox
            .enqueue(Command::Text {
                id: 1,
                text: "Café 🦆",
            })
            .unwrap();
        outbox
            .enqueue(Command::Move {
                id: 1,
                x: 50,
                y: 60,
            })
            .unwrap();
        outbox.enqueue(Command::Close { id: 1 }).unwrap();
        struct Partial {
            bytes: Vec<u8>,
            blocked: bool,
        }
        impl Write for Partial {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if self.blocked {
                    return Err(io::ErrorKind::WouldBlock.into());
                }
                let size = bytes.len().min(7);
                self.bytes.extend_from_slice(&bytes[..size]);
                self.blocked = true;
                Ok(size)
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut writer = Partial {
            bytes: vec![],
            blocked: false,
        };
        while !outbox.commands.is_empty() {
            outbox.flush(&mut writer).unwrap();
            writer.blocked = false;
        }
        let values: Vec<serde_json::Value> = writer
            .bytes
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).unwrap())
            .collect();
        assert_eq!(values.len(), 4);
        assert_eq!(values[0]["x"], 30);
        assert_eq!(values[1]["text"], "Café 🦆");
        assert_eq!(values[2]["x"], 50);
        assert_eq!(values[3]["op"], "close");
        assert_eq!(outbox.bytes, 0);
    }

    #[test]
    fn queue_bounds_and_nonfinite_coordinates_fail_closed() {
        let mut outbox = Outbox::default();
        for _ in 0..MAX_QUEUED_COMMANDS {
            outbox.enqueue(Command::Focus { id: 1 }).unwrap();
        }
        assert!(outbox.enqueue(Command::Close { id: 1 }).is_err());
        assert!(coordinates(Vec2::new(f32::NAN, 0.0)).is_err());
        assert!(coordinates(Vec2::new(1_000_001.0, 0.0)).is_err());
        assert_eq!(
            coordinates(Vec2::new(-1400.0, 200.0)).unwrap(),
            (-1400, 200)
        );
    }
}
