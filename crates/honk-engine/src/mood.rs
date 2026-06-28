//! Platform-free mood state and local-hour honk inputs (M13).
//!
//! The engine owns deterministic mood transitions and particles. The desktop runtime owns
//! sampling the local clock and feeds snapshots in through `World::set_local_time`.

use crate::math::Vec2;
use crate::rng::{RandomSource, SplitMix64};
use crate::sound::Sound;

const Z_LIFETIME: f32 = 1.6;
const Z_RISE: f32 = 42.0;

const UP: Vec2 = Vec2 { x: 0.0, y: -1.0 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoodKind {
    Content,
    Hyper,
    Sad,
    Sleepy,
    Mischievous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoodIntensity {
    Calm,
    Normal,
    Spicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoodOptions {
    pub dynamic_moods: bool,
    pub intensity: MoodIntensity,
}

impl Default for MoodOptions {
    fn default() -> Self {
        Self {
            dynamic_moods: true,
            intensity: MoodIntensity::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HourlyHonkOptions {
    pub on_hour_double_honk: bool,
}

impl Default for HourlyHonkOptions {
    fn default() -> Self {
        Self {
            on_hour_double_honk: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    pub day: i32,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl LocalTime {
    pub const fn hour_key(self) -> LocalHour {
        LocalHour {
            day: self.day,
            hour: self.hour,
        }
    }

    pub const fn is_top_of_hour(self) -> bool {
        self.minute == 0 && self.second == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalHour {
    pub day: i32,
    pub hour: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoodEvent {
    pub changed: bool,
    pub trigger_hyper: bool,
    pub sound: Option<Sound>,
    pub spawn_sleepy_particle: bool,
}

impl MoodEvent {
    const NONE: Self = Self {
        changed: false,
        trigger_hyper: false,
        sound: None,
        spawn_sleepy_particle: false,
    };
}

#[derive(Debug, Clone)]
pub struct MoodMachine {
    options: MoodOptions,
    current: MoodKind,
    next_transition: f32,
    next_sleepy_particle: f32,
}

impl MoodMachine {
    pub fn new(now: f32, options: MoodOptions, rng: &mut SplitMix64) -> Self {
        let mut machine = Self {
            options,
            current: MoodKind::Content,
            next_transition: now,
            next_sleepy_particle: now + 2.0,
        };
        machine.schedule_next_transition(now, rng);
        machine
    }

    pub fn current(&self) -> MoodKind {
        self.current
    }

    pub fn options(&self) -> MoodOptions {
        self.options
    }

    pub fn apply_options(&mut self, options: MoodOptions, now: f32, rng: &mut SplitMix64) {
        self.options = options;
        if !options.dynamic_moods {
            self.current = MoodKind::Content;
        }
        self.schedule_next_transition(now, rng);
    }

    pub fn tick(&mut self, now: f32, rng: &mut SplitMix64) -> MoodEvent {
        if !self.options.dynamic_moods {
            self.current = MoodKind::Content;
            return MoodEvent::NONE;
        }

        let mut event = MoodEvent::NONE;
        if now >= self.next_transition {
            self.current = self.choose_next(rng);
            self.schedule_next_transition(now, rng);
            event.changed = true;
            event.trigger_hyper = self.current == MoodKind::Hyper;
            event.sound = match self.current {
                MoodKind::Hyper => Some(Sound::high_honk()),
                MoodKind::Sad => Some(Sound::low_honk()),
                _ => None,
            };
        }

        if self.current == MoodKind::Sleepy && now >= self.next_sleepy_particle {
            event.spawn_sleepy_particle = true;
            self.next_sleepy_particle = now + self.sleepy_interval(rng);
        }
        event
    }

    fn choose_next(&self, rng: &mut SplitMix64) -> MoodKind {
        let weights: &[(MoodKind, u32)] = match self.options.intensity {
            MoodIntensity::Calm => &[
                (MoodKind::Content, 12),
                (MoodKind::Sleepy, 4),
                (MoodKind::Sad, 2),
                (MoodKind::Mischievous, 1),
                (MoodKind::Hyper, 1),
            ],
            MoodIntensity::Normal => &[
                (MoodKind::Content, 14),
                (MoodKind::Sleepy, 4),
                (MoodKind::Sad, 3),
                (MoodKind::Mischievous, 2),
                (MoodKind::Hyper, 1),
            ],
            MoodIntensity::Spicy => &[
                (MoodKind::Content, 8),
                (MoodKind::Sleepy, 2),
                (MoodKind::Sad, 2),
                (MoodKind::Mischievous, 5),
                (MoodKind::Hyper, 3),
            ],
        };
        let total: u32 = weights.iter().map(|(_, w)| *w).sum();
        let mut draw = (rng.next_f64() * total as f64).floor() as u32;
        for (kind, weight) in weights {
            if draw < *weight {
                return *kind;
            }
            draw -= *weight;
        }
        MoodKind::Content
    }

    fn schedule_next_transition(&mut self, now: f32, rng: &mut SplitMix64) {
        let (min, max) = match self.options.intensity {
            MoodIntensity::Calm => (90.0, 150.0),
            MoodIntensity::Normal => (60.0, 120.0),
            MoodIntensity::Spicy => (35.0, 85.0),
        };
        self.next_transition = now + rng.range(min, max);
    }

    fn sleepy_interval(&self, rng: &mut SplitMix64) -> f32 {
        let (min, max) = match self.options.intensity {
            MoodIntensity::Calm => (4.0, 7.0),
            MoodIntensity::Normal => (3.0, 6.0),
            MoodIntensity::Spicy => (2.0, 4.0),
        };
        rng.range(min, max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZParticle {
    pub origin: Vec2,
    pub created: f32,
}

impl ZParticle {
    fn age(self, now: f32) -> f32 {
        now - self.created
    }

    pub fn is_alive(self, now: f32) -> bool {
        self.age(now) <= Z_LIFETIME
    }

    pub fn alpha(self, now: f32) -> f32 {
        crate::math::clamp(1.0 - self.age(now) / Z_LIFETIME, 0.0, 1.0)
    }

    pub fn position(self, now: f32) -> Vec2 {
        let frac = crate::math::clamp(self.age(now) / Z_LIFETIME, 0.0, 1.0);
        self.origin + UP * (Z_RISE * frac) + Vec2::new(10.0 * frac, 0.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZParticles {
    items: Vec<ZParticle>,
}

impl ZParticles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, origin: Vec2, now: f32) {
        self.items.retain(|z| z.is_alive(now));
        self.items.push(ZParticle {
            origin,
            created: now,
        });
    }

    pub fn active(&self, now: f32) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        self.items
            .iter()
            .copied()
            .filter(move |z| z.is_alive(now))
            .map(move |z| (z.position(now), z.alpha(now)))
    }

    pub fn alive_count(&self, now: f32) -> usize {
        self.active(now).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_moods_stay_content() {
        let mut rng = SplitMix64::seed(1);
        let mut machine = MoodMachine::new(
            0.0,
            MoodOptions {
                dynamic_moods: false,
                intensity: MoodIntensity::Normal,
            },
            &mut rng,
        );
        for tick in 0..1_000 {
            machine.tick(tick as f32, &mut rng);
            assert_eq!(machine.current(), MoodKind::Content);
        }
    }

    #[test]
    fn transitions_are_deterministic_for_seed() {
        let mut a_rng = SplitMix64::seed(42);
        let mut b_rng = SplitMix64::seed(42);
        let mut a = MoodMachine::new(0.0, MoodOptions::default(), &mut a_rng);
        let mut b = MoodMachine::new(0.0, MoodOptions::default(), &mut b_rng);
        let mut a_seen = Vec::new();
        let mut b_seen = Vec::new();
        for second in 0..500 {
            if a.tick(second as f32, &mut a_rng).changed {
                a_seen.push(a.current());
            }
            if b.tick(second as f32, &mut b_rng).changed {
                b_seen.push(b.current());
            }
        }
        assert_eq!(a_seen, b_seen);
        assert!(!a_seen.is_empty());
    }

    #[test]
    fn z_particles_rise_and_fade() {
        let mut particles = ZParticles::new();
        particles.add(Vec2::new(10.0, 10.0), 0.0);
        assert_eq!(particles.alive_count(0.0), 1);
        let (pos, alpha) = particles.active(0.8).next().unwrap();
        assert!(pos.y < 10.0);
        assert!(alpha > 0.0 && alpha < 1.0);
        assert_eq!(particles.alive_count(2.0), 0);
    }
}
