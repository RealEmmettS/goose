//! One continuous, ground-anchored projected goose (ADR 0041).
//!
//! Heading rotates a shared anatomy rather than blending different silhouettes. Foot
//! contacts remain world-space truth; animation only changes the body above them.
use crate::feet::{Feet, FeetState, FootPose};
use crate::math::{Rect, Vec2};

const NECK_SMOOTH_RATE: f32 = 10.0;
const TURN_RATE: f32 = 11.0;
const BLINK_CLOSE: f64 = 0.06;
const BLINK_OPEN: f64 = 0.09;

/// A single anatomy, including the anchors used by interaction and delivery tasks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rig {
    pub ground: Vec2,
    pub forward: Vec2,
    pub neck_lerp_percent: f32,
    pub body_center: Vec2,
    pub underbody_center: Vec2,
    pub neck_base: Vec2,
    pub neck_c1: Vec2,
    pub neck_c2: Vec2,
    pub neck_head: Vec2,
    pub snout_center: Vec2,
    pub beak_tip: Vec2,
    pub eye: Vec2,
    pub head_tilt: f32,
    pub feet: Feet,
    pub feet_pose: [FootPose; 2],
    pub bob: f32,
    pub breath: f32,
    pub blink: f32,
    pub tail_flick: f32,
    pub beak_open: f32,
    pub pleased: f32,
    pub anticipation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GoosePose {
    pub primary: Rig,
}
impl GoosePose {
    pub fn bounding_box(&self) -> Rect {
        self.primary.bounding_box()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RigInput {
    pub center: Vec2,
    pub direction_deg: f32,
    pub neck_target: f32,
    pub speed: f32,
    pub velocity: Vec2,
    pub step_time: f32,
    pub now: f64,
    pub dt: f32,
}
impl RigInput {
    pub fn static_pose(center: Vec2, direction_deg: f32, neck_target: f32) -> Self {
        Self {
            center,
            direction_deg,
            neck_target,
            speed: 0.0,
            velocity: Vec2::ZERO,
            step_time: 0.2,
            now: 0.0,
            dt: 0.0,
        }
    }
}

/// Finite reaction channels, with no script engine or unbounded animation queue.
#[derive(Debug, Clone, PartialEq)]
pub struct RigAnim {
    pub feet: FeetState,
    heading: f32,
    neck: f32,
    breath_phase: f32,
    blink_started: f64,
    pub next_blink: f64,
    tail_flick: f32,
    honk: f32,
    pleased: f32,
    anticipation: f32,
    anticipation_target: f32,
    lean: f32,
    last_speed: f32,
    expressions: bool,
    reduced_motion: bool,
}
impl RigAnim {
    pub fn new(center: Vec2, direction_deg: f32) -> Self {
        Self {
            feet: FeetState::new(center, Vec2::from_angle_degrees(direction_deg)),
            heading: direction_deg.to_radians(),
            neck: 0.0,
            breath_phase: 0.0,
            blink_started: f64::NEG_INFINITY,
            next_blink: 2.0,
            tail_flick: 0.0,
            honk: 0.0,
            pleased: 0.0,
            anticipation: 0.0,
            anticipation_target: 0.0,
            lean: 0.0,
            last_speed: 0.0,
            expressions: true,
            reduced_motion: false,
        }
    }
    pub fn start_blink(&mut self, now: f64) {
        self.blink_started = now;
    }
    /// Sound event: the jaw opens once and the tail gives a small answering kick.
    pub fn flick_tail(&mut self) {
        self.tail_flick = 1.0;
        self.honk = 1.0;
    }
    pub fn pet(&mut self) {
        self.pleased = 1.0;
    }
    pub fn anticipate(&mut self, active: bool) {
        self.anticipation_target = f32::from(active);
    }
    pub fn set_expression_options(&mut self, enabled: bool, reduced_motion: bool) {
        self.expressions = enabled;
        self.reduced_motion = reduced_motion;
    }
    fn blink_amount(&self, now: f64) -> f32 {
        let t = now - self.blink_started;
        if t < 0.0 {
            0.0
        } else if t < BLINK_CLOSE {
            (t / BLINK_CLOSE) as f32
        } else if t < BLINK_CLOSE + BLINK_OPEN {
            (1.0 - (t - BLINK_CLOSE) / BLINK_OPEN) as f32
        } else {
            0.0
        }
    }
    pub fn update(&mut self, input: &RigInput) -> GoosePose {
        let dt = input.dt.max(0.0);
        let target = input.direction_deg.to_radians();
        let delta = (target - self.heading + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        self.heading = if dt == 0.0 {
            target
        } else {
            self.heading + delta * (1.0 - (-TURN_RATE * dt).exp())
        }
        .rem_euclid(std::f32::consts::TAU);
        let forward = Vec2::new(self.heading.cos(), self.heading.sin());
        self.feet
            .tick(dt, input.center, forward, input.velocity, input.step_time);
        let ease = if dt == 0.0 {
            1.0
        } else {
            1.0 - (-NECK_SMOOTH_RATE * dt).exp()
        };
        self.neck += (input.neck_target.clamp(0.0, 1.0) - self.neck) * ease;
        self.anticipation += (self.anticipation_target - self.anticipation) * ease;
        let acceleration = if dt > 0.0 {
            (input.speed - self.last_speed) / dt
        } else {
            0.0
        };
        let lean_target = (acceleration / 800.0).clamp(-1.0, 1.0);
        self.lean += (lean_target - self.lean) * ease;
        self.last_speed = input.speed;
        self.breath_phase = (self.breath_phase + dt * std::f32::consts::TAU * 0.48)
            .rem_euclid(std::f32::consts::TAU);
        self.tail_flick *= (-dt / 0.18).exp();
        self.honk = (self.honk - dt / 0.22).max(0.0);
        self.pleased = (self.pleased - dt / 1.1).max(0.0);
        let decoration = if self.expressions && !self.reduced_motion {
            1.0
        } else {
            0.0
        };
        let expressions = f32::from(self.expressions);
        let breath =
            self.breath_phase.sin() * (1.0 - input.speed / 40.0).clamp(0.0, 1.0) * decoration;
        let poses = self.feet.poses();
        // The body rises over the supporting leg during recovery. A stopped goose has
        // no free-running gait wave; both feet and body settle together.
        let lift = poses[0].lift.max(poses[1].lift);
        let bob = lift * 0.30 + breath * 0.24;
        let weight = (poses[0].lift - poses[1].lift) * 0.14;
        let projected = |f: f32, side: f32, up: f32| input.center + project(forward, f, side, up);
        let neck = self.neck * self.neck * (3.0 - 2.0 * self.neck);
        let lean = self.lean * 2.0 * decoration;
        let anticipation = self.anticipation * expressions;
        let pleased = self.pleased * expressions;
        // Keep the rounded belly above the webbed feet through front/rear stance.
        // This is anatomical clearance, shared by the body and neck, not an offset
        // applied to the planted foot contacts.
        let body_center = projected(-2.0 + lean, weight, 28.0 + bob);
        let neck_base = projected(10.0 + lean, weight, 35.0 + bob);
        let head_tilt = (self.lean * 0.10 + anticipation * 0.13 - pleased * 0.08) * decoration;
        let neck_head = projected(
            18.0 + neck * 3.0 + anticipation * 3.0 + lean,
            weight + pleased * 1.3 * decoration,
            57.0 + neck * 11.0 + bob + pleased * 1.3,
        );
        let neck_c1 = neck_base + project(forward, -2.0, 0.0, 10.0);
        let neck_c2 = neck_head + project(forward, -4.0, 0.0, -9.0);
        let beak_open = (self.honk * std::f32::consts::PI).sin().max(0.0) * expressions;
        let primary = Rig {
            ground: input.center,
            forward,
            neck_lerp_percent: self.neck,
            body_center,
            underbody_center: body_center + Vec2::new(0.0, 10.0),
            neck_base,
            neck_c1,
            neck_c2,
            neck_head,
            snout_center: neck_head + project(forward, 14.0, 0.0, -3.0 - head_tilt * 12.0),
            beak_tip: neck_head + project(forward, 23.0, 0.0, -3.0 - head_tilt * 23.0),
            eye: neck_head + project(forward, 4.0, 10.0, 3.0),
            head_tilt,
            feet: self.feet.positions(),
            feet_pose: poses,
            bob,
            breath,
            blink: self.blink_amount(input.now) * expressions,
            tail_flick: self.tail_flick * decoration,
            beak_open,
            pleased,
            anticipation,
        };
        GoosePose { primary }
    }
    pub fn neck(&self) -> f32 {
        self.neck
    }
}

/// Orthographic ground-plane basis and screen-up elevation. Forward and side are
/// anatomical distances, never independently rotated or mirrored image layers.
pub(crate) fn project(forward: Vec2, distance: f32, side: f32, height: f32) -> Vec2 {
    Vec2::new(
        forward.x * distance - forward.y * side,
        (forward.y * distance + forward.x * side) * 0.45 - height,
    )
}
impl Default for Rig {
    fn default() -> Self {
        Self::update(Vec2::new(300.0, 300.0), 0.0, 0.0, 0.0)
    }
}
impl Rig {
    pub fn update(center: Vec2, direction_deg: f32, neck: f32, _gait: f32) -> Self {
        RigAnim::new(center, direction_deg)
            .update(&RigInput::static_pose(center, direction_deg, neck))
            .primary
    }
    pub fn bounding_box(&self) -> Rect {
        let mut min = self.body_center - Vec2::new(35.0, 26.0);
        let mut max = self.body_center + Vec2::new(35.0, 26.0);
        for (point, radius) in [
            (self.neck_head, 17.0),
            (self.beak_tip, 12.0),
            (self.neck_base, 12.0),
            (self.feet.left, 13.0),
            (self.feet.right, 13.0),
            (self.ground, 28.0),
        ] {
            min.x = min.x.min(point.x - radius);
            min.y = min.y.min(point.y - radius);
            max.x = max.x.max(point.x + radius);
            max.y = max.y.max(point.y + radius);
        }
        Rect { min, max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::DT;
    #[test]
    fn anchors_remain_anatomical_through_a_complete_turn() {
        for degree in 0..360 {
            let rig = Rig::update(Vec2::new(-200.0, 300.0), degree as f32, 0.5, 0.0);
            assert!(rig.neck_head.y < rig.body_center.y - 10.0);
            let leading = rig.beak_tip - rig.neck_head;
            assert!(leading.x * rig.forward.x + leading.y * rig.forward.y > 6.0);
            assert!(rig.bounding_box().contains(rig.beak_tip));
        }
    }
    #[test]
    fn heading_wrap_and_reversal_keep_every_anchor_continuous() {
        let mut anim = RigAnim::new(Vec2::ZERO, 359.0);
        let mut input = RigInput::static_pose(Vec2::ZERO, 1.0, 0.4);
        input.dt = DT;
        let first = anim.update(&input).primary;
        assert!(first.forward.x > 0.99);
        input.direction_deg = 181.0;
        let mut previous = first;
        for _ in 0..180 {
            let next = anim.update(&input).primary;
            assert!(Vec2::distance(previous.beak_tip, next.beak_tip) < 13.0);
            assert!(Vec2::distance(previous.body_center, next.body_center) < 3.0);
            previous = next;
        }
        assert!(previous.forward.x < -0.99);
    }
    #[test]
    fn neck_eases_and_raises_head() {
        let mut anim = RigAnim::new(Vec2::ZERO, 0.0);
        let tucked = anim
            .update(&RigInput::static_pose(Vec2::ZERO, 0.0, 0.0))
            .primary;
        let mut input = RigInput::static_pose(Vec2::ZERO, 0.0, 1.0);
        input.dt = DT;
        assert!(anim.update(&input).primary.neck_lerp_percent < 0.2);
        for _ in 0..60 {
            anim.update(&input);
        }
        assert!(anim.update(&input).primary.neck_head.y < tucked.neck_head.y - 9.0);
    }
    #[test]
    fn reactions_expire_and_reduced_motion_keeps_honk_readable() {
        let mut anim = RigAnim::new(Vec2::ZERO, 0.0);
        anim.set_expression_options(true, true);
        anim.pet();
        anim.flick_tail();
        let mut input = RigInput::static_pose(Vec2::ZERO, 0.0, 0.0);
        input.dt = DT;
        let pose = anim.update(&input).primary;
        assert!(pose.beak_open > 0.0 && pose.pleased > 0.0);
        assert_eq!(
            (pose.breath, pose.tail_flick, pose.head_tilt),
            (0.0, 0.0, 0.0)
        );
        for _ in 0..150 {
            anim.update(&input);
        }
        let pose = anim.update(&input).primary;
        assert_eq!((pose.beak_open, pose.pleased), (0.0, 0.0));
        anim.set_expression_options(false, false);
        anim.flick_tail();
        anim.pet();
        let pose = anim.update(&input).primary;
        assert_eq!((pose.beak_open, pose.pleased), (0.0, 0.0));
    }
    #[test]
    fn blink_closes_and_reopens() {
        let mut anim = RigAnim::new(Vec2::ZERO, 0.0);
        anim.start_blink(1.0);
        let mut input = RigInput::static_pose(Vec2::ZERO, 0.0, 0.0);
        input.now = 1.05;
        assert!(anim.update(&input).primary.blink > 0.5);
        input.now = 1.3;
        assert_eq!(anim.update(&input).primary.blink, 0.0);
    }
    #[test]
    fn long_lived_phase_is_bounded() {
        let mut anim = RigAnim::new(Vec2::ZERO, 0.0);
        anim.breath_phase = std::f32::consts::TAU * 100_000.0;
        let mut input = RigInput::static_pose(Vec2::ZERO, 0.0, 0.45);
        input.dt = DT;
        anim.update(&input);
        assert!(anim.breath_phase < std::f32::consts::TAU);
    }
}
