//! 2D vector math, reimplemented clean-room from the engine's `SamMath` / `Vector2`.
//!
//! All engine geometry and physics use `f32` to match the original's `float`
//! parameter table; keeping the numeric type identical avoids subtle drift when we
//! assert ported constants against their verified source values.

/// Degrees → radians multiplier (`π / 180`).
pub const DEG2RAD: f32 = std::f32::consts::PI / 180.0;
/// Radians → degrees multiplier (`180 / π`).
pub const RAD2DEG: f32 = 180.0 / std::f32::consts::PI;

/// Linear interpolation: `a` at `p = 0`, `b` at `p = 1`. `p` is not clamped.
pub fn lerp(a: f32, b: f32, p: f32) -> f32 {
    a * (1.0 - p) + b * p
}

/// Clamp `value` into `[min, max]`.
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// A 2D point/vector in screen space (x right, y down).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// The zero vector.
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    /// Construct from components.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Unit vector pointing along `angle` (degrees), measured from +x toward +y.
    pub fn from_angle_degrees(angle: f32) -> Self {
        let r = angle * DEG2RAD;
        Self::new(r.cos(), r.sin())
    }

    /// Euclidean distance between two points.
    pub fn distance(a: Vec2, b: Vec2) -> f32 {
        (a - b).magnitude()
    }

    /// Component-wise linear interpolation.
    pub fn lerp(a: Vec2, b: Vec2, p: f32) -> Vec2 {
        Vec2::new(lerp(a.x, b.x, p), lerp(a.y, b.y, p))
    }

    /// Dot product.
    pub fn dot(a: Vec2, b: Vec2) -> f32 {
        a.x * b.x + a.y * b.y
    }

    /// Vector length.
    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Unit vector in the same direction; returns [`Vec2::ZERO`] for the zero vector
    /// (matching the original's guard).
    pub fn normalize(self) -> Vec2 {
        if self.x == 0.0 && self.y == 0.0 {
            return Vec2::ZERO;
        }
        let d = self.magnitude();
        Vec2::new(self.x / d, self.y / d)
    }

    /// Same direction, length capped at `max`.
    pub fn clamp_magnitude(self, max: f32) -> Vec2 {
        if self.magnitude() > max {
            self.normalize() * max
        } else {
            self
        }
    }

    /// Unit vector 90° clockwise from `self` (used for lateral offsets: eyes, feet).
    pub fn perpendicular(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }
}

/// An axis-aligned rectangle in world space, used for dirty-rect / bounding-box work.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    /// Smallest rect containing all `points`, then grown by `pad` on every side.
    /// Returns `None` for an empty point set.
    pub fn bounding(points: impl IntoIterator<Item = Vec2>, pad: f32) -> Option<Rect> {
        let mut it = points.into_iter();
        let first = it.next()?;
        let mut min = first;
        let mut max = first;
        for p in it {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Some(Rect {
            min: Vec2::new(min.x - pad, min.y - pad),
            max: Vec2::new(max.x + pad, max.y + pad),
        })
    }

    /// Width of the rect.
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height of the rect.
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn deg2rad_roundtrips() {
        assert!(approx(180.0 * DEG2RAD, std::f32::consts::PI));
        assert!(approx(std::f32::consts::PI * RAD2DEG, 180.0));
    }

    #[test]
    fn from_angle_degrees_cardinals() {
        let right = Vec2::from_angle_degrees(0.0);
        assert!(approx(right.x, 1.0) && approx(right.y, 0.0));
        // 90° points toward +y (screen-down), matching the goose's default direction.
        let down = Vec2::from_angle_degrees(90.0);
        assert!(approx(down.x, 0.0) && approx(down.y, 1.0));
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        assert!(approx(lerp(10.0, 20.0, 0.0), 10.0));
        assert!(approx(lerp(10.0, 20.0, 1.0), 20.0));
        assert!(approx(lerp(10.0, 20.0, 0.5), 15.0));
    }

    #[test]
    fn clamp_bounds() {
        assert!(approx(clamp(5.0, 0.0, 10.0), 5.0));
        assert!(approx(clamp(-1.0, 0.0, 10.0), 0.0));
        assert!(approx(clamp(99.0, 0.0, 10.0), 10.0));
    }

    #[test]
    fn vector_ops() {
        let a = Vec2::new(3.0, 4.0);
        assert!(approx(a.magnitude(), 5.0));
        assert!(approx(Vec2::distance(Vec2::ZERO, a), 5.0));
        let n = a.normalize();
        assert!(approx(n.magnitude(), 1.0));
        assert_eq!(Vec2::ZERO.normalize(), Vec2::ZERO);
        let c = a.clamp_magnitude(2.5);
        assert!(approx(c.magnitude(), 2.5));
        assert!(approx(
            Vec2::dot(Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)),
            0.0
        ));
        assert_eq!(Vec2::new(1.0, 2.0).perpendicular(), Vec2::new(-2.0, 1.0));
    }
}
