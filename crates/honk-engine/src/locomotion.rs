//! Auto-locomotion: each tick the goose accelerates toward `target_pos`, capped at its
//! current speed tier, and stops cleanly on arrival.
//!
//! Clean-room: the original `tick` integration lives only in the closed binary. This is a
//! straightforward, constant-driven reconstruction consistent with the verified parameter
//! table (`WalkSpeed`/`RunSpeed`/`ChargeSpeed`, `AccelerationNormal`/`Charged`). Tasks
//! (M4+) only set `target_pos` / `current_speed` / `current_acceleration`; this is the one
//! place position and velocity change.

use crate::entity::GooseEntity;
use crate::math::{Vec2, RAD2DEG};

/// Advance `goose` by one tick of `dt` seconds toward `target_pos`.
pub fn step(goose: &mut GooseEntity, dt: f32) {
    let to_target = goose.target_pos - goose.position;
    let dist = to_target.magnitude();

    // Distance coverable this tick at full speed; once we're within it (and allowed to
    // stop sharply), snap to the target and halt rather than orbit it.
    let reach = (goose.current_speed * dt).max(0.5);
    if dist <= reach && goose.can_decelerate_immediately {
        goose.position = goose.target_pos;
        goose.velocity = Vec2::ZERO;
        return;
    }

    // Steer velocity toward "full speed straight at the target", limited by acceleration.
    let desired = if dist > 1e-4 {
        to_target.normalize() * goose.current_speed
    } else {
        Vec2::ZERO
    };
    let max_dv = goose.current_acceleration * dt;
    goose.velocity = goose.velocity + (desired - goose.velocity).clamp_magnitude(max_dv);

    // Never exceed the current speed tier.
    goose.velocity = goose.velocity.clamp_magnitude(goose.current_speed);

    // Integrate and face the direction of travel.
    goose.position = goose.position + goose.velocity * dt;
    if goose.velocity.magnitude() > 1e-3 {
        goose.direction = goose.velocity.y.atan2(goose.velocity.x) * RAD2DEG;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::SpeedTier;
    use crate::time::DT;

    fn walker() -> GooseEntity {
        let mut g = GooseEntity::new();
        g.position = Vec2::ZERO;
        g.current_speed = g.parameters.speed(SpeedTier::Walk); // 80
        g.current_acceleration = g.parameters.acceleration(SpeedTier::Walk); // 1300
        g
    }

    #[test]
    fn walks_at_correct_speed() {
        let mut g = walker();
        g.target_pos = Vec2::new(10_000.0, 0.0); // far enough to reach cruising speed
        for _ in 0..120 {
            step(&mut g, DT); // one simulated second
        }
        // Cruising velocity settles at WalkSpeed and never exceeds it.
        assert!(g.velocity.magnitude() <= 80.0 + 1e-3);
        assert!((g.velocity.magnitude() - 80.0).abs() < 0.5);
        // ~80 px travelled in 1 s, minus the small acceleration-ramp deficit.
        assert!(
            g.position.x > 76.0 && g.position.x <= 80.5,
            "x = {}",
            g.position.x
        );
        // Heading is +x (≈ 0°).
        assert!(g.direction.abs() < 1.0, "dir = {}", g.direction);
    }

    #[test]
    fn never_exceeds_higher_tiers() {
        for (speed, accel) in [(200.0, 1300.0), (400.0, 2300.0)] {
            let mut g = walker();
            g.current_speed = speed;
            g.current_acceleration = accel;
            g.target_pos = Vec2::new(100_000.0, 0.0);
            for _ in 0..240 {
                step(&mut g, DT);
                assert!(g.velocity.magnitude() <= speed + 1e-2);
            }
        }
    }

    #[test]
    fn arrives_and_stops() {
        let mut g = walker();
        g.position = Vec2::new(0.0, 0.0);
        g.target_pos = Vec2::new(50.0, 0.0);
        for _ in 0..600 {
            step(&mut g, DT);
        }
        assert!(Vec2::distance(g.position, g.target_pos) < 0.5);
        assert!(
            g.velocity.magnitude() < 1e-3,
            "should be at rest at the target"
        );
    }
}
