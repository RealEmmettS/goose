//! Built-in Autumn leaf piles (M14).
//!
//! The original shipped Autumn as a DLL mod. honk300 keeps the behavior built into the
//! platform-free engine and renders leaves procedurally.

use crate::entity::GooseEntity;
use crate::math::{clamp, Rect, Vec2};
use crate::rng::{RandomSource, SplitMix64};
use crate::time::DT;

pub const FIRST_PILE_SECONDS: f32 = 10.0;
pub const PILE_INTERVAL_MIN: f32 = 4.8;
pub const PILE_INTERVAL_MAX: f32 = 72.0;
pub const MAX_LEAF_PILES: usize = 6;
pub const LEAVES_PER_PILE: usize = 128;
pub const SPAWN_ANIM_LENGTH: f32 = 1.0;
pub const LIFETIME_AFTER_KICKED: f32 = 10.0;
pub const GRAVITY: f32 = -900.0;
pub const MAX_VEL_XY: f32 = 200.0;
pub const KICK_MIN_VERT_VEL: f32 = 10.0;
pub const KICK_MAX_VERT_VEL: f32 = 500.0;
pub const RENDER_Z_SCALE_VERTICAL: f32 = 0.6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutumnPileId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutumnPileTarget {
    pub id: AutumnPileId,
    pub position: Vec2,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutumnLeaf {
    pub planar: Vec2,
    pub z: f32,
    pub vel_planar: Vec2,
    pub vel_z: f32,
    pub color: AutumnLeafColor,
}

impl AutumnLeaf {
    pub fn screen_offset(self) -> Vec2 {
        self.planar + Vec2::new(0.0, -self.z * RENDER_Z_SCALE_VERTICAL)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutumnLeafColor {
    Gold,
    Orange,
    Red,
    Brown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutumnPile {
    pub id: AutumnPileId,
    pub position: Vec2,
    pub radius: f32,
    pub height: f32,
    pub created_at: f32,
    pub kicked_at: Option<f32>,
    pub leaves: Vec<AutumnLeaf>,
}

impl AutumnPile {
    fn new(
        id: AutumnPileId,
        position: Vec2,
        radius: f32,
        height: f32,
        now: f32,
        rng: &mut SplitMix64,
    ) -> Self {
        let mut leaves = Vec::with_capacity(LEAVES_PER_PILE);
        for i in 0..LEAVES_PER_PILE {
            let unit = random_in_unit_circle(rng);
            let mut height_frac = rng.range(0.0, 1.0);
            height_frac *= height_frac;
            let spread = unit * radius * (1.0 - height_frac);
            let planar = Vec2::new(spread.x, spread.y * 0.6);
            leaves.push(AutumnLeaf {
                planar,
                z: height_frac * height,
                vel_planar: Vec2::ZERO,
                vel_z: 0.0,
                color: match i % 4 {
                    0 => AutumnLeafColor::Gold,
                    1 => AutumnLeafColor::Orange,
                    2 => AutumnLeafColor::Red,
                    _ => AutumnLeafColor::Brown,
                },
            });
        }
        Self {
            id,
            position,
            radius,
            height,
            created_at: now,
            kicked_at: None,
            leaves,
        }
    }

    fn kick(
        &mut self,
        kick_velocity: Vec2,
        goose_speed_percentage: f32,
        now: f32,
        rng: &mut SplitMix64,
    ) {
        self.kicked_at = Some(now);
        let speed_scale = lerp(0.6, 1.1, clamp(goose_speed_percentage, 0.0, 1.0));
        let kick_dir = normalized_or(kick_velocity, Vec2::new(1.0, 0.0));
        for leaf in &mut self.leaves {
            let radial = normalized_or(leaf.planar, random_in_unit_circle(rng));
            let cross = 1.0 - Vec2::dot(radial, kick_dir).abs();
            let mut planar = radial * rng.range(0.0, MAX_VEL_XY);
            planar = planar + radial * cross * MAX_VEL_XY * 0.2;
            planar = lerp_vec(planar, kick_velocity, 0.3);
            let mut vel_z = rng.range(KICK_MIN_VERT_VEL, KICK_MAX_VERT_VEL);
            vel_z *= lerp(0.9, 1.1, cross);
            leaf.vel_planar = planar * speed_scale;
            leaf.vel_z = vel_z * speed_scale;
        }
    }

    fn tick(&mut self) {
        if self.kicked_at.is_none() {
            return;
        }
        for leaf in &mut self.leaves {
            leaf.planar = leaf.planar + leaf.vel_planar * DT;
            leaf.z += leaf.vel_z * DT;
            leaf.vel_z += GRAVITY * DT;
            if leaf.z < 0.0 {
                leaf.z = 0.0;
                leaf.vel_z *= -0.3;
                leaf.vel_planar = leaf.vel_planar * 0.2;
            }
        }
    }

    pub fn spawn_scale(&self, now: f32) -> f32 {
        ease_out_bounce(clamp((now - self.created_at) / SPAWN_ANIM_LENGTH, 0.0, 1.0))
    }

    pub fn fade_out(&self, now: f32) -> f32 {
        let Some(kicked_at) = self.kicked_at else {
            return 0.0;
        };
        clamp(((now - kicked_at) - 8.0) / 2.0, 0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct AutumnState {
    piles: Vec<AutumnPile>,
    next_pile_at: Option<f32>,
    next_id: u64,
}

impl Default for AutumnState {
    fn default() -> Self {
        Self::new()
    }
}

impl AutumnState {
    pub fn new() -> Self {
        Self {
            piles: Vec::new(),
            next_pile_at: None,
            next_id: 1,
        }
    }

    pub fn piles(&self) -> &[AutumnPile] {
        &self.piles
    }

    pub fn targets(&self) -> Vec<AutumnPileTarget> {
        self.piles
            .iter()
            .filter(|pile| pile.kicked_at.is_none())
            .map(|pile| AutumnPileTarget {
                id: pile.id,
                position: pile.position,
                radius: pile.radius,
            })
            .collect()
    }

    pub fn has_unkicked_piles(&self) -> bool {
        self.piles.iter().any(|pile| pile.kicked_at.is_none())
    }

    pub fn clear(&mut self) {
        self.piles.clear();
        self.next_pile_at = None;
    }

    pub fn tick(
        &mut self,
        now: f32,
        active: bool,
        bounds: Rect,
        goose: &GooseEntity,
        rng: &mut SplitMix64,
    ) {
        if !active {
            self.clear();
            return;
        }
        let next = *self.next_pile_at.get_or_insert(now + FIRST_PILE_SECONDS);
        if now >= next {
            self.next_pile_at = Some(now + rng.range(PILE_INTERVAL_MIN, PILE_INTERVAL_MAX));
            if self.piles.len() < MAX_LEAF_PILES {
                self.spawn_pile(now, bounds, rng);
            }
        }

        let walk = goose.parameters.walk_speed;
        let charge = goose.parameters.charge_speed;
        let denom = (charge - walk).max(1.0);
        let speed_pct = clamp((goose.velocity.magnitude() - walk) / denom, 0.0, 1.0);
        for pile in &mut self.piles {
            if pile.kicked_at.is_none()
                && Vec2::distance(goose.position, pile.position) < pile.radius + 4.0
            {
                pile.kick(goose.velocity, speed_pct, now, rng);
            }
            pile.tick();
        }
        self.piles.retain(|pile| {
            pile.kicked_at
                .is_none_or(|kicked_at| now - kicked_at <= LIFETIME_AFTER_KICKED)
        });
    }

    fn spawn_pile(&mut self, now: f32, bounds: Rect, rng: &mut SplitMix64) {
        let width = bounds.width().max(1.0);
        let height = bounds.height().max(1.0);
        let position = Vec2::new(
            bounds.min.x + width * rng.range(0.2, 0.8),
            bounds.min.y + height * rng.range(0.2, 0.8),
        );
        let radius = rng.range(30.0, 50.0);
        let pile_height = rng.range(30.0, 50.0);
        let pile = AutumnPile::new(
            AutumnPileId(self.next_id),
            position,
            radius,
            pile_height,
            now,
            rng,
        );
        self.next_id += 1;
        self.piles.push(pile);
    }
}

fn random_in_unit_circle(rng: &mut SplitMix64) -> Vec2 {
    let angle = rng.range(0.0, std::f32::consts::TAU);
    let radius = rng.range(0.0, 1.0);
    Vec2::new(radius * angle.cos(), radius * angle.sin())
}

fn normalized_or(v: Vec2, fallback: Vec2) -> Vec2 {
    if v.magnitude() <= f32::EPSILON {
        fallback
    } else {
        v.normalize()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_vec(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}

fn ease_out_bounce(x: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if x < 1.0 / d1 {
        n1 * x * x
    } else if x < 2.0 / d1 {
        let x = x - 1.5 / d1;
        n1 * x * x + 0.75
    } else if x < 2.5 / d1 {
        let x = x - 2.25 / d1;
        n1 * x * x + 0.9375
    } else {
        let x = x - 2.625 / d1;
        n1 * x * x + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{GooseEntity, ParametersTable};

    fn bounds() -> Rect {
        Rect {
            min: Vec2::ZERO,
            max: Vec2::new(800.0, 600.0),
        }
    }

    #[test]
    fn autumn_spawns_first_pile_after_ten_seconds() {
        let mut autumn = AutumnState::new();
        let goose = GooseEntity::new();
        let mut rng = SplitMix64::seed(1);
        autumn.tick(0.0, true, bounds(), &goose, &mut rng);
        autumn.tick(9.9, true, bounds(), &goose, &mut rng);
        assert!(autumn.piles().is_empty());
        autumn.tick(10.1, true, bounds(), &goose, &mut rng);
        assert_eq!(autumn.piles().len(), 1);
        assert_eq!(autumn.piles()[0].leaves.len(), LEAVES_PER_PILE);
    }

    #[test]
    fn autumn_pile_cap_is_enforced() {
        let mut autumn = AutumnState::new();
        let goose = GooseEntity::new();
        let mut rng = SplitMix64::seed(2);
        for i in 0..20 {
            autumn.next_pile_at = Some(i as f32);
            autumn.tick(i as f32, true, bounds(), &goose, &mut rng);
        }
        assert_eq!(autumn.piles().len(), MAX_LEAF_PILES);
    }

    #[test]
    fn autumn_clears_when_inactive() {
        let mut autumn = AutumnState::new();
        let goose = GooseEntity::new();
        let mut rng = SplitMix64::seed(3);
        autumn.tick(0.0, true, bounds(), &goose, &mut rng);
        autumn.tick(10.1, true, bounds(), &goose, &mut rng);
        assert!(!autumn.piles().is_empty());
        autumn.tick(10.2, false, bounds(), &goose, &mut rng);
        assert!(autumn.piles().is_empty());
    }

    #[test]
    fn goose_kicks_pile_and_it_expires() {
        let mut autumn = AutumnState::new();
        let mut goose = GooseEntity::new();
        goose.parameters = ParametersTable::default();
        let mut rng = SplitMix64::seed(4);
        autumn.tick(0.0, true, bounds(), &goose, &mut rng);
        autumn.tick(10.1, true, bounds(), &goose, &mut rng);
        let pos = autumn.piles()[0].position;
        goose.position = pos;
        goose.velocity = Vec2::new(goose.parameters.charge_speed, 0.0);
        autumn.tick(10.2, true, bounds(), &goose, &mut rng);
        assert!(autumn.piles()[0].kicked_at.is_some());
        autumn.tick(21.0, true, bounds(), &goose, &mut rng);
        assert!(autumn.piles().is_empty());
    }
}
