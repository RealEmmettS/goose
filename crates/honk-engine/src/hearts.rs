//! Heart particles emitted while the goose is being patted (M6, plan §5.9).
//!
//! Petting = repeated cursor hover-sweeps over the goose; each registered pat spawns a heart
//! that rises and fades. Clean-room procedural (the original ships a `heart.png`; we draw it).
//! A small growable buffer that self-prunes dead hearts on insert, so it stays bounded.

use crate::math::Vec2;

/// How long a heart lives before it has fully faded, in seconds.
pub const LIFETIME: f32 = 1.3;
/// How far (pixels) a heart drifts upward over its lifetime.
pub const RISE: f32 = 34.0;

/// Screen-up unit vector (hearts float up).
const UP: Vec2 = Vec2 { x: 0.0, y: -1.0 };

/// A single heart: where it was spawned and when.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heart {
    /// World position the heart was spawned at.
    pub origin: Vec2,
    /// Wall-clock time the heart was created.
    pub created: f32,
}

impl Heart {
    /// Seconds since the heart was spawned.
    fn age(&self, now: f32) -> f32 {
        now - self.created
    }

    /// Whether the heart is still within its lifetime at `now`.
    pub fn is_alive(&self, now: f32) -> bool {
        self.age(now) <= LIFETIME
    }

    /// Opacity scale in `[0, 1]`: `1.0` fresh, fading linearly to `0.0` at end of life.
    pub fn alpha(&self, now: f32) -> f32 {
        crate::math::clamp(1.0 - self.age(now) / LIFETIME, 0.0, 1.0)
    }

    /// Current world position at `now` (origin, drifted upward by elapsed-life fraction).
    pub fn position(&self, now: f32) -> Vec2 {
        let frac = crate::math::clamp(self.age(now) / LIFETIME, 0.0, 1.0);
        self.origin + UP * (RISE * frac)
    }
}

/// A small buffer of rising/fading hearts. Dead hearts are pruned on [`Hearts::add`].
#[derive(Debug, Clone, Default)]
pub struct Hearts {
    items: Vec<Heart>,
}

impl Hearts {
    /// An empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a heart at `origin`, created at `now`. Prunes hearts already dead at `now` first.
    pub fn add(&mut self, origin: Vec2, now: f32) {
        self.items.retain(|h| h.is_alive(now));
        self.items.push(Heart {
            origin,
            created: now,
        });
    }

    /// All hearts alive at `now`, with their drifted position and current alpha.
    pub fn active(&self, now: f32) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        self.items
            .iter()
            .filter(move |h| h.is_alive(now))
            .map(move |h| (h.position(now), h.alpha(now)))
    }

    /// Count of hearts alive at `now`.
    pub fn alive_count(&self, now: f32) -> usize {
        self.active(now).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifetime_constants_are_sane() {
        assert_eq!(LIFETIME, 1.3);
        assert_eq!(RISE, 34.0);
    }

    #[test]
    fn add_then_alive_until_lifetime() {
        let mut h = Hearts::new();
        h.add(Vec2::new(10.0, 10.0), 0.0);
        assert_eq!(h.alive_count(0.0), 1);
        assert_eq!(h.alive_count(1.0), 1);
        assert_eq!(h.alive_count(1.4), 0, "past LIFETIME it is gone");
    }

    #[test]
    fn rises_upward_and_fades() {
        let heart = Heart {
            origin: Vec2::new(0.0, 0.0),
            created: 0.0,
        };
        // Fresh: full alpha, at origin.
        assert!((heart.alpha(0.0) - 1.0).abs() < 1e-6);
        assert_eq!(heart.position(0.0), Vec2::new(0.0, 0.0));
        // Mid-life: dimmer and higher on screen (smaller y).
        assert!(heart.alpha(0.65) < 1.0 && heart.alpha(0.65) > 0.0);
        assert!(heart.position(0.65).y < 0.0, "should have risen up");
        // Dead: zero alpha.
        assert_eq!(heart.alpha(2.0), 0.0);
    }

    #[test]
    fn prunes_dead_hearts_on_add() {
        let mut h = Hearts::new();
        h.add(Vec2::new(1.0, 1.0), 0.0);
        // Long after the first heart died, adding another drops the corpse.
        h.add(Vec2::new(2.0, 2.0), 2.0);
        assert_eq!(h.alive_count(2.0), 1, "the dead first heart was pruned");
    }
}
