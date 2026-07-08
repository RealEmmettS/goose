//! The goose rig: verified geometry constants, the persistent animation state, and the
//! per-frame dual-view pose computation.
//!
//! **V2 (Procedural Vector, ADR 0014).** The goose is drawn from two flat-illustration
//! views adapted from the project's own reference art (`docs/art-reference/`):
//!
//! - **Side profile** for shallow headings and idle/interaction — mirrored left/right,
//!   never rotated, with an S-curve neck spine, layered wing, and two-tone legs driven
//!   by the stateful plant-and-swing gait ([`crate::feet::FeetState`]).
//! - **Top-down** for steep headings — rotates freely to any heading with a subtle
//!   waddle roll.
//!
//! A ~125 ms crossfade with hysteresis (enter top-down past 55° from horizontal, leave
//! below 45°) switches between them; [`GoosePose`] carries the primary rig plus the
//! optional fading one so the renderer composites both and dirty rects union both.
//!
//! The geometry **constants** remain verbatim from the verified source (`Exports.cs`,
//! `Rig`/`ProceduralFeets`) and are pinned by test; the assembly math is clean-room,
//! tuned against the reference art via the preview harness.

use crate::feet::{Feet, FeetState, FootPose};
use crate::math::{Rect, Vec2};

// Verified rig constants (Exports.cs `Rig`) — pinned by test. V2 uses them as the
// authoritative proportions they encode (neck travel, head/eye layout) scaled into the
// reference-art coordinate frame.
pub const UNDERBODY_RADIUS: f32 = 15.0;
pub const UNDERBODY_LENGTH: f32 = 7.0;
pub const UNDERBODY_ELEVATION: f32 = 9.0;
pub const BODY_RADIUS: f32 = 22.0;
pub const BODY_LENGTH: f32 = 11.0;
pub const BODY_ELEVATION: f32 = 14.0;
pub const NECC_RADIUS: f32 = 13.0;
pub const NECC_HEIGHT_1: f32 = 20.0;
pub const NECC_FORWARD_1: f32 = 3.0;
pub const NECC_HEIGHT_2: f32 = 10.0;
pub const NECC_FORWARD_2: f32 = 16.0;
pub const HEAD_RADIUS_1: f32 = 15.0;
pub const HEAD_LENGTH_1: f32 = 3.0;
pub const HEAD_RADIUS_2: f32 = 10.0;
pub const HEAD_LENGTH_2: f32 = 5.0;
pub const EYE_RADIUS: f32 = 2.0;
pub const EYE_ELEVATION: f32 = 3.0;
pub const IPD: f32 = 5.0;
pub const EYES_FORWARD: f32 = 5.0;

/// Screen-up unit vector.
const UP: Vec2 = Vec2 { x: 0.0, y: -1.0 };

/// Reference-art → screen scale for the side view (`docs/art-reference/goose-side-main.svg`,
/// 196-unit-tall art → ~74 px standing goose).
pub const SIDE_SCALE: f32 = 0.42;
/// Reference-art → screen scale for the top-down view.
pub const TOP_SCALE: f32 = 0.40;
/// Ground anchor of the side reference art (between the feet, at the foot line).
const REF_GROUND: Vec2 = Vec2 { x: 72.0, y: 183.0 };

/// Heading steepness (degrees from horizontal) past which the top-down view engages…
pub const TOPDOWN_ENTER_DEG: f32 = 55.0;
/// …and below which the side view re-engages (hysteresis band).
pub const TOPDOWN_EXIT_DEG: f32 = 45.0;
/// View crossfade duration (seconds).
pub const VIEW_FADE_SECS: f32 = 0.125;

/// How quickly the smoothed neck chases its target (1/s).
const NECK_SMOOTH_RATE: f32 = 10.0;
/// Blink close/open times (seconds).
const BLINK_CLOSE: f32 = 0.06;
const BLINK_OPEN: f32 = 0.09;
/// Idle breathing rate (Hz) and how strongly speed suppresses it.
const BREATH_HZ: f32 = 0.55;
/// Tail-flick decay time constant (seconds).
const TAIL_FLICK_DECAY: f32 = 0.35;

/// Which flat-illustration view a [`Rig`] was assembled for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RigView {
    /// Side profile. `mirror` is `-1.0` facing screen-left (art-native) or `+1.0`
    /// facing screen-right (horizontally flipped).
    Side { mirror: f32 },
    /// Top-down, rotated to `heading` (unit vector), with a small `waddle` roll
    /// (radians) synced to the gait.
    TopDown { heading: Vec2, waddle: f32 },
}

/// Computed positions of every attach point for one frame of one view. Field names and
/// semantics are kept from V1 — tasks grab the cursor at `beak_tip`, hearts rise from
/// `neck_head`, anchors use `body_center` — so the task layer is untouched by V2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rig {
    /// Ground point (feet level / shadow centre) — the entity position.
    pub ground: Vec2,
    /// Facing/travel unit vector (raw heading, both views).
    pub forward: Vec2,
    /// Smoothed neck blend actually shown: `0` tucked → `1` raised/reaching.
    pub neck_lerp_percent: f32,
    /// Which view this rig assembles, with its view parameters.
    pub view: RigView,

    pub body_center: Vec2,
    /// Belly / under-body anchor (drives the underbody shading in the side view).
    pub underbody_center: Vec2,
    /// Neck spine root (shoulder).
    pub neck_base: Vec2,
    /// Neck spine control points (cubic bezier `neck_base → c1 → c2 → neck_head`).
    pub neck_c1: Vec2,
    pub neck_c2: Vec2,
    /// Head anchor (skull centre) — spine end.
    pub neck_head: Vec2,
    /// Mid-beak point.
    pub snout_center: Vec2,
    /// Beak tip — the cursor-nab / collect grab point.
    pub beak_tip: Vec2,
    pub eye: Vec2,
    /// Head tilt (radians; side view only, positive = beak dips).
    pub head_tilt: f32,

    /// Ground positions of both feet (footmarks, tests).
    pub feet: Feet,
    /// Full render poses `[left, right]` (lift, heading, swing state).
    pub feet_pose: [FootPose; 2],

    /// Body bob (pixels, already applied to body/neck/head — kept for tests/tools).
    pub bob: f32,
    /// Idle-breathing scale for the body (`0` = none; renderer scales body ry by `1+0.02*breath`).
    pub breath: f32,
    /// Blink amount `0` open → `1` closed.
    pub blink: f32,
    /// Honk tail-flick `0..1`, decaying.
    pub tail_flick: f32,
}

/// One frame's full drawable pose: the active view plus, during a view crossfade, the
/// outgoing view with its remaining opacity (`1 → 0`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GoosePose {
    pub primary: Rig,
    pub fading: Option<(Rig, f32)>,
}

impl GoosePose {
    /// Union bounding box of every drawn view (dirty-rect present must cover the fade).
    pub fn bounding_box(&self) -> Rect {
        let bb = self.primary.bounding_box();
        match &self.fading {
            Some((rig, _)) => bb.union(rig.bounding_box()),
            None => bb,
        }
    }
}

/// Per-frame inputs to the pose computation (world → rig).
#[derive(Debug, Clone, Copy)]
pub struct RigInput {
    /// Entity ground position.
    pub center: Vec2,
    /// Facing/travel direction (degrees).
    pub direction_deg: f32,
    /// Desired neck blend `0..1` (mood/extend target; the rig eases toward it).
    pub neck_target: f32,
    /// Current speed (px/s) and velocity (px/s).
    pub speed: f32,
    pub velocity: Vec2,
    /// Per-step interval (seconds; `<=0` falls back to the walk default).
    pub step_time: f32,
    /// World clock (seconds) and tick delta.
    pub now: f32,
    pub dt: f32,
}

impl RigInput {
    /// A deterministic, motion-free input for tests/tools: standing at `center`,
    /// facing `direction_deg`, neck at `neck_target`, at `t = 0`.
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

/// Persistent animation state carried across ticks on the entity.
#[derive(Debug, Clone, PartialEq)]
pub struct RigAnim {
    /// Stateful plant-and-swing feet.
    pub feet: FeetState,
    /// `0` side ↔ `1` top-down: which view is active (target of the fade).
    top_active: bool,
    /// Fade progress `0..1` toward the active view (1 = fade complete).
    fade: f32,
    /// Side-view facing: `-1` left, `+1` right.
    mirror: f32,
    /// Smoothed neck blend.
    neck: f32,
    /// Breathing phase (radians).
    breath_phase: f32,
    /// When the current/last blink started; `f32::NEG_INFINITY` = none yet.
    blink_started: f32,
    /// When the next blink is due (world seeds this from its RNG).
    pub next_blink: f32,
    /// Honk tail-flick energy `0..1`.
    tail_flick: f32,
}

impl RigAnim {
    /// Fresh state for a goose standing at `center` facing `direction_deg`.
    pub fn new(center: Vec2, direction_deg: f32) -> Self {
        let forward = Vec2::from_angle_degrees(direction_deg);
        Self {
            feet: FeetState::new(center, forward),
            top_active: false,
            fade: 1.0,
            mirror: side_mirror(forward).unwrap_or(-1.0),
            neck: 0.0,
            breath_phase: 0.0,
            blink_started: f32::NEG_INFINITY,
            next_blink: 2.0,
            tail_flick: 0.0,
        }
    }

    /// Begin a blink now (world calls this when `now >= next_blink`, then reschedules
    /// `next_blink` from its RNG so blinking stays deterministic per seed).
    pub fn start_blink(&mut self, now: f32) {
        self.blink_started = now;
    }

    /// Kick the tail (honk reaction).
    pub fn flick_tail(&mut self) {
        self.tail_flick = 1.0;
    }

    fn blink_amount(&self, now: f32) -> f32 {
        let t = now - self.blink_started;
        if t < 0.0 {
            0.0
        } else if t < BLINK_CLOSE {
            t / BLINK_CLOSE
        } else if t < BLINK_CLOSE + BLINK_OPEN {
            1.0 - (t - BLINK_CLOSE) / BLINK_OPEN
        } else {
            0.0
        }
    }

    /// Advance every animation channel one tick and assemble the drawable pose.
    pub fn update(&mut self, input: &RigInput) -> GoosePose {
        let forward = Vec2::from_angle_degrees(input.direction_deg);

        // Feet first: the gait is the ground truth the body reacts to.
        self.feet.tick(
            input.dt,
            input.center,
            forward,
            input.velocity,
            input.step_time,
        );

        // Eased neck (no more mood snapping). A zero-dt update is a static pose
        // request (tests/tools): snap straight to the target so poses are exact.
        if input.dt > 0.0 {
            let k = 1.0 - (-NECK_SMOOTH_RATE * input.dt).exp();
            self.neck += (input.neck_target.clamp(0.0, 1.0) - self.neck) * k;
        } else {
            self.neck = input.neck_target.clamp(0.0, 1.0);
        }

        // Side-facing mirror: update only when the heading is meaningfully horizontal.
        if input.speed > 1.0 {
            if let Some(m) = side_mirror(forward) {
                self.mirror = m;
            }
        }

        // View selection with hysteresis; only a moving goose can change views.
        if input.speed > 1.0 {
            let steep_deg = forward.y.abs().asin().to_degrees();
            if self.top_active && steep_deg < TOPDOWN_EXIT_DEG {
                self.top_active = false;
                self.fade = 1.0 - self.fade.min(1.0);
            } else if !self.top_active && steep_deg > TOPDOWN_ENTER_DEG {
                self.top_active = true;
                self.fade = 1.0 - self.fade.min(1.0);
            }
        }
        if input.dt > 0.0 {
            self.fade = (self.fade + input.dt / VIEW_FADE_SECS).min(1.0);
        }

        // Breathing fades out with speed; tail flick decays exponentially.
        let idle = (1.0 - input.speed / 40.0).clamp(0.0, 1.0);
        self.breath_phase += input.dt * std::f32::consts::TAU * BREATH_HZ;
        let breath = self.breath_phase.sin() * idle;
        if input.dt > 0.0 {
            self.tail_flick *= (-input.dt / TAIL_FLICK_DECAY).exp();
            if self.tail_flick < 0.01 {
                self.tail_flick = 0.0;
            }
        }

        let blink = self.blink_amount(input.now);
        let shared = SharedChannels {
            forward,
            neck: self.neck,
            breath,
            blink,
            tail_flick: self.tail_flick,
        };

        let primary_view = self.top_active;
        let primary = assemble(input, &self.feet, &shared, primary_view, self.mirror);
        let fading = if self.fade < 1.0 {
            let rig = assemble(input, &self.feet, &shared, !primary_view, self.mirror);
            Some((rig, 1.0 - self.fade))
        } else {
            None
        };
        GoosePose { primary, fading }
    }

    /// Current smoothed neck value (tests/status).
    pub fn neck(&self) -> f32 {
        self.neck
    }

    /// True while the top-down view is the active one.
    pub fn top_view_active(&self) -> bool {
        self.top_active
    }
}

/// Side-view mirror for a heading, if the heading is horizontal enough to say
/// (`None` inside the dead zone so the last facing sticks).
fn side_mirror(forward: Vec2) -> Option<f32> {
    if forward.x > 0.2 {
        Some(1.0)
    } else if forward.x < -0.2 {
        Some(-1.0)
    } else {
        None
    }
}

/// Animation channels shared by both view assemblies in one frame.
struct SharedChannels {
    forward: Vec2,
    neck: f32,
    breath: f32,
    blink: f32,
    tail_flick: f32,
}

fn assemble(
    input: &RigInput,
    feet: &FeetState,
    ch: &SharedChannels,
    top: bool,
    mirror: f32,
) -> Rig {
    if top {
        assemble_top(input, feet, ch)
    } else {
        assemble_side(input, feet, ch, mirror)
    }
}

/// Map a side-reference-art point (`goose-side-main.svg` coordinates, art faces
/// screen-left) onto the screen for a goose standing at `ground` with facing `mirror`.
pub(crate) fn side_ref_pt(ground: Vec2, mirror: f32, rx: f32, ry: f32) -> Vec2 {
    let fwd = (REF_GROUND.x - rx) * SIDE_SCALE; // + = toward the beak
    let up = (REF_GROUND.y - ry) * SIDE_SCALE; // + = above the ground line
    Vec2::new(ground.x + fwd * mirror, ground.y - up)
}

fn assemble_side(input: &RigInput, feet: &FeetState, ch: &SharedChannels, mirror: f32) -> Rig {
    let ground = input.center;
    // Screen-forward for the side view: strictly horizontal, per the mirror.
    let fwd = Vec2::new(mirror, 0.0);
    // `side_ref_pt` treats mirror=-1 as art-native (facing left).
    let rp = |rx: f32, ry: f32| side_ref_pt(ground, mirror, rx, ry);

    // Body bob: rises mid-swing, plus a whisper of idle breathing.
    let bob = feet.swing_progress().sin_bob() + ch.breath * 0.5;
    let bob_v = UP * bob;

    let body_center = rp(80.0, 112.0) + bob_v;
    let underbody_center = rp(58.0, 132.0) + bob_v;

    // Neck spine: shoulder → head anchor, eased between tucked and raised.
    let p = ch.neck;
    let eased = p * p * (3.0 - 2.0 * p);
    let neck_base = rp(50.0, 80.0) + bob_v;
    let head_tucked = rp(51.0, 46.0);
    let head_raised = rp(44.0, 20.0);
    // Arc the travel slightly forward so the raise reads organic, not linear.
    let arc = fwd * (10.0 * (eased * (1.0 - eased)) * 4.0 * SIDE_SCALE);
    let neck_head = Vec2::lerp(head_tucked, head_raised, eased) + arc + bob_v * 0.6;

    let len = Vec2::distance(neck_base, neck_head);
    // S-curve: lower control bows backward, upper control bows forward.
    let neck_c1 = neck_base + UP * (len * 0.40) - fwd * (len * 0.16);
    let neck_c2 = neck_head - UP * (len * 0.32) + fwd * (len * 0.18);

    // Head tilt: level when raised, dipping slightly when tucked.
    let head_tilt = (1.0 - eased) * 0.16;

    // Beak/eye offsets from the head anchor (44,20) in ref units, tilt-rotated.
    let head_off = |rx: f32, ry: f32| {
        let dx = (44.0 - rx) * SIDE_SCALE; // + toward beak tip
        let dy = (20.0 - ry) * SIDE_SCALE; // + above head anchor
        let (s, c) = head_tilt.sin_cos();
        let fx = dx * c - (-dy) * s * 0.0 - 0.0; // rotation of (dx, up dy) about anchor
                                                 // Rotate (forward=dx, up=dy) by -tilt around the anchor: forward stays forward,
                                                 // up tips toward forward as the head dips.
        let f2 = dx * c + dy * s;
        let u2 = dy * c - dx * s;
        let _ = fx;
        neck_head + fwd * f2 + UP * u2
    };
    let beak_tip = head_off(13.0, 26.0);
    let snout_center = head_off(28.0, 22.0);
    let eye = head_off(41.2, 17.0);

    Rig {
        ground,
        forward: ch.forward,
        neck_lerp_percent: p,
        view: RigView::Side { mirror },
        body_center,
        underbody_center,
        neck_base,
        neck_c1,
        neck_c2,
        neck_head,
        snout_center,
        beak_tip,
        eye,
        head_tilt,
        feet: feet.positions(),
        feet_pose: feet.poses(),
        bob,
        breath: ch.breath,
        blink: ch.blink,
        tail_flick: ch.tail_flick,
    }
}

fn assemble_top(input: &RigInput, feet: &FeetState, ch: &SharedChannels) -> Rig {
    let ground = input.center;
    let heading = if ch.forward.magnitude() > 1e-3 {
        ch.forward
    } else {
        Vec2::new(0.0, -1.0)
    };
    // Waddle roll synced to the gait.
    let waddle = (feet.swing_progress() * std::f32::consts::PI).sin() * 0.05;

    // Neck reach translates the head/beak forward along the heading.
    let p = ch.neck;
    let eased = p * p * (3.0 - 2.0 * p);
    let reach = heading * (eased * 8.0);

    let body_center = ground;
    let neck_base = ground + heading * (43.0 * TOP_SCALE);
    let neck_head = ground + heading * (57.5 * TOP_SCALE) + reach;
    let beak_tip = ground + heading * (85.7 * TOP_SCALE) + reach;
    let snout_center = ground + heading * (76.0 * TOP_SCALE) + reach;
    let mid = Vec2::lerp(neck_base, neck_head, 0.5);

    Rig {
        ground,
        forward: ch.forward,
        neck_lerp_percent: p,
        view: RigView::TopDown { heading, waddle },
        body_center,
        underbody_center: body_center,
        neck_base,
        neck_c1: mid,
        neck_c2: mid,
        neck_head,
        snout_center,
        beak_tip,
        // Eyes sit on the head from above; expose the leading edge point.
        eye: neck_head + heading * (6.0 * TOP_SCALE),
        head_tilt: 0.0,
        feet: feet.positions(),
        feet_pose: feet.poses(),
        bob: 0.0,
        breath: ch.breath,
        blink: ch.blink,
        tail_flick: ch.tail_flick,
    }
}

/// Small helper: turn a swing progress `0..1` into the body-bob height.
trait SinBob {
    fn sin_bob(self) -> f32;
}
impl SinBob for f32 {
    fn sin_bob(self) -> f32 {
        (self * std::f32::consts::PI).sin() * 1.2
    }
}

impl Default for Rig {
    fn default() -> Self {
        let mut anim = RigAnim::new(Vec2::new(300.0, 300.0), 0.0);
        anim.update(&RigInput::static_pose(Vec2::new(300.0, 300.0), 0.0, 0.0))
            .primary
    }
}

impl Rig {
    /// Compute a single-view pose statically (tests/tools). Prefer holding a
    /// [`RigAnim`] and calling [`RigAnim::update`] in real simulation loops.
    pub fn update(center: Vec2, direction_deg: f32, neck_lerp_percent: f32, _gait: f32) -> Self {
        let mut anim = RigAnim::new(center, direction_deg);
        anim.update(&RigInput::static_pose(
            center,
            direction_deg,
            neck_lerp_percent,
        ))
        .primary
    }

    /// World-space bounding box covering this view of the goose, padded for outlines
    /// and the soft shadow.
    pub fn bounding_box(&self) -> Rect {
        match self.view {
            RigView::Side { mirror } => {
                let rp = |rx: f32, ry: f32| side_ref_pt(self.ground, mirror, rx, ry);
                let points = [
                    self.ground,
                    rp(11.9, 12.8),  // beak tip region
                    rp(30.5, 8.8),   // head top
                    rp(136.8, 62.0), // back/tail top
                    rp(136.8, 117.0),
                    rp(31.0, 185.0), // toes
                    rp(105.0, 185.0),
                    self.neck_head,
                    self.beak_tip,
                    self.feet.left,
                    self.feet.right,
                ];
                Rect::bounding(points, 12.0).expect("rig always has points")
            }
            RigView::TopDown { heading, .. } => {
                let r = 90.0 * TOP_SCALE + 10.0;
                let tip = self.beak_tip + heading * 6.0;
                Rect::bounding(
                    [
                        self.ground + Vec2::new(r, r),
                        self.ground - Vec2::new(r, r),
                        tip,
                    ],
                    12.0,
                )
                .expect("rig always has points")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::DT;

    #[test]
    fn geometry_constants_match_verified_source() {
        assert_eq!(BODY_RADIUS, 22.0);
        assert_eq!(BODY_LENGTH, 11.0);
        assert_eq!(BODY_ELEVATION, 14.0);
        assert_eq!(UNDERBODY_RADIUS, 15.0);
        assert_eq!(NECC_RADIUS, 13.0);
        assert_eq!((NECC_HEIGHT_1, NECC_FORWARD_1), (20.0, 3.0));
        assert_eq!((NECC_HEIGHT_2, NECC_FORWARD_2), (10.0, 16.0));
        assert_eq!((HEAD_RADIUS_1, HEAD_LENGTH_1), (15.0, 3.0));
        assert_eq!((HEAD_RADIUS_2, HEAD_LENGTH_2), (10.0, 5.0));
        assert_eq!(
            (EYE_RADIUS, EYE_ELEVATION, IPD, EYES_FORWARD),
            (2.0, 3.0, 5.0, 5.0)
        );
    }

    #[test]
    fn side_goose_is_assembled_sanely() {
        // Facing +x (right): head above body, beak leading forward.
        let rig = Rig::update(Vec2::new(300.0, 300.0), 0.0, 0.0, 0.0);
        assert!(matches!(rig.view, RigView::Side { mirror } if mirror == 1.0));
        assert!(rig.body_center.y < rig.ground.y);
        assert!(rig.neck_head.y < rig.body_center.y);
        assert!(rig.snout_center.x > rig.body_center.x);
        assert!(rig.beak_tip.x > rig.snout_center.x);
        assert!(rig.feet.left.y > rig.body_center.y);
        assert!(Vec2::distance(rig.feet.left, rig.feet.right) > 1.0);
    }

    #[test]
    fn side_view_mirrors_left() {
        let rig = Rig::update(Vec2::new(300.0, 300.0), 180.0, 0.0, 0.0);
        assert!(matches!(rig.view, RigView::Side { mirror } if mirror == -1.0));
        assert!(rig.beak_tip.x < rig.body_center.x);
    }

    #[test]
    fn neck_raises_the_head_over_time() {
        let center = Vec2::new(300.0, 300.0);
        let mut anim = RigAnim::new(center, 0.0);
        let tucked = anim
            .update(&RigInput::static_pose(center, 0.0, 0.0))
            .primary;
        // Ease toward a raised neck for half a second of ticks.
        let mut input = RigInput::static_pose(center, 0.0, 1.0);
        input.dt = DT;
        let mut raised = tucked;
        for i in 0..60 {
            input.now = i as f32 * DT;
            raised = anim.update(&input).primary;
        }
        assert!(raised.neck_head.y < tucked.neck_head.y - 5.0);
        assert!(raised.neck_lerp_percent > 0.9);
    }

    #[test]
    fn neck_eases_instead_of_snapping() {
        let center = Vec2::new(300.0, 300.0);
        let mut anim = RigAnim::new(center, 0.0);
        let mut input = RigInput::static_pose(center, 0.0, 1.0);
        input.dt = DT;
        input.now = DT;
        let after_one_tick = anim.update(&input).primary;
        // One tick toward a raised neck moves only a fraction of the way.
        assert!(after_one_tick.neck_lerp_percent < 0.2);
    }

    #[test]
    fn steep_heading_switches_to_top_down_with_fade() {
        let center = Vec2::new(300.0, 300.0);
        let mut anim = RigAnim::new(center, 0.0);
        // Establish the side view.
        let mut input = RigInput::static_pose(center, 0.0, 0.0);
        input.dt = DT;
        anim.update(&input);
        assert!(!anim.top_view_active());
        // Dive straight down at speed: top-down should engage with a crossfade.
        let mut input = RigInput {
            center,
            direction_deg: 90.0,
            neck_target: 0.0,
            speed: 80.0,
            velocity: Vec2::new(0.0, 80.0),
            step_time: 0.2,
            now: DT,
            dt: DT,
        };
        let pose = anim.update(&input);
        assert!(anim.top_view_active());
        assert!(matches!(pose.primary.view, RigView::TopDown { .. }));
        let (fading, alpha) = pose.fading.expect("crossfade in progress");
        assert!(matches!(fading.view, RigView::Side { .. }));
        assert!(alpha > 0.0 && alpha < 1.0);
        // The fade completes within VIEW_FADE_SECS.
        for i in 2..30 {
            input.now = i as f32 * DT;
            input.center = input.center + input.velocity * DT;
            let pose = anim.update(&input);
            if pose.fading.is_none() {
                assert!(input.now <= VIEW_FADE_SECS + 3.0 * DT);
                return;
            }
        }
        panic!("crossfade never completed");
    }

    #[test]
    fn view_has_hysteresis() {
        let center = Vec2::new(300.0, 300.0);
        let mut anim = RigAnim::new(center, 0.0);
        let tick = |deg: f32, anim: &mut RigAnim| {
            let v = Vec2::from_angle_degrees(deg) * 80.0;
            anim.update(&RigInput {
                center,
                direction_deg: deg,
                neck_target: 0.0,
                speed: 80.0,
                velocity: v,
                step_time: 0.2,
                now: 0.0,
                dt: DT,
            });
        };
        // 50° is inside the hysteresis band: entering from the side view, no switch.
        tick(50.0, &mut anim);
        assert!(!anim.top_view_active());
        // 60° switches in; falling back to 50° must NOT switch out (exit is 45°).
        tick(60.0, &mut anim);
        assert!(anim.top_view_active());
        tick(50.0, &mut anim);
        assert!(anim.top_view_active());
        tick(30.0, &mut anim);
        assert!(!anim.top_view_active());
    }

    #[test]
    fn pose_bounding_box_unions_both_views_during_fade() {
        let center = Vec2::new(300.0, 300.0);
        let mut anim = RigAnim::new(center, 0.0);
        anim.update(&RigInput::static_pose(center, 0.0, 0.0));
        let pose = anim.update(&RigInput {
            center,
            direction_deg: 90.0,
            neck_target: 0.0,
            speed: 80.0,
            velocity: Vec2::new(0.0, 80.0),
            step_time: 0.2,
            now: DT,
            dt: DT,
        });
        let (fading, _) = pose.fading.expect("fade active");
        let union = pose.bounding_box();
        let a = pose.primary.bounding_box();
        let b = fading.bounding_box();
        assert!(union.min.x <= a.min.x.min(b.min.x));
        assert!(union.max.y >= a.max.y.max(b.max.y));
    }

    #[test]
    fn bounding_box_contains_the_goose() {
        let rig = Rig::default();
        let bb = rig.bounding_box();
        assert!(bb.width() > 0.0 && bb.height() > 0.0);
        assert!(bb.min.y <= rig.neck_head.y && bb.max.y >= rig.feet.left.y);
        assert!(bb.min.x <= rig.beak_tip.x && bb.max.x >= rig.beak_tip.x);
    }

    #[test]
    fn blink_closes_and_reopens() {
        let center = Vec2::new(300.0, 300.0);
        let mut anim = RigAnim::new(center, 0.0);
        anim.start_blink(1.0);
        let mut input = RigInput::static_pose(center, 0.0, 0.0);
        input.now = 1.05;
        let mid = anim.update(&input).primary;
        assert!(mid.blink > 0.5, "blink should be closing at t+50ms");
        input.now = 1.3;
        let after = anim.update(&input).primary;
        assert_eq!(after.blink, 0.0, "blink should be over at t+300ms");
    }
}
