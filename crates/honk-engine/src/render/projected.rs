//! Round, continuous anatomy. All parts share the rig projection and depth order.
use super::{geom::*, mix, RenderPalette};
use crate::{
    math::Vec2,
    rig::{project, Rig},
};
use tiny_skia::{Path, PathBuilder};

struct Draw<'a, C: VectorCanvas> {
    pixels: &'a mut C,
    origin: Vec2,
    scale: f32,
}
impl<C: VectorCanvas> Draw<'_, C> {
    fn pt(&self, p: Vec2) -> Vec2 {
        (p - self.origin) * self.scale
    }
    fn path(&self, points: &[Vec2]) -> Path {
        let mut p = PathBuilder::new();
        let start = self.pt(points[0]);
        p.move_to(start.x, start.y);
        for curve in points[1..].chunks_exact(3) {
            let a = self.pt(curve[0]);
            let b = self.pt(curve[1]);
            let c = self.pt(curve[2]);
            p.cubic_to(a.x, a.y, b.x, b.y, c.x, c.y);
        }
        p.close();
        p.finish().expect("finite projected shape")
    }
    fn oval(&self, center: Vec2, rx: f32, ry: f32) -> Path {
        let c = self.pt(center);
        PathBuilder::from_oval(
            tiny_skia::Rect::from_xywh(
                c.x - rx * self.scale,
                c.y - ry * self.scale,
                rx * 2.0 * self.scale,
                ry * 2.0 * self.scale,
            )
            .expect("positive ellipse"),
        )
        .expect("oval")
    }
    fn fill(&mut self, path: &Path, color: (u8, u8, u8)) {
        fill(self.pixels, path, &paint(color, 255));
    }
    fn line(&mut self, path: &Path, color: (u8, u8, u8), width: f32) {
        stroke(self.pixels, path, &paint(color, 255), width * self.scale);
    }
    fn segment(&mut self, a: Vec2, b: Vec2, color: (u8, u8, u8), width: f32) {
        let a = self.pt(a);
        let b = self.pt(b);
        let mut p = PathBuilder::new();
        p.move_to(a.x, a.y);
        p.line_to(b.x, b.y);
        self.line(&p.finish().expect("segment"), color, width);
    }
}

pub(super) fn paint_goose(
    pixels: &mut impl VectorCanvas,
    rig: &Rig,
    origin: Vec2,
    scale: f32,
    pal: &RenderPalette,
) {
    let mut d = Draw {
        pixels,
        origin,
        scale,
    };
    let f = rig.forward;
    let local = |forward, side, up| project(f, forward, side, up);
    let shade = mix(pal.goose_white, pal.goose_shade, 0.45);
    let wing = mix(pal.goose_white, pal.goose_wing, 0.12);
    let edge = mix(pal.goose_outline, pal.goose_shade, 0.35);
    // Three translucent ellipses form a soft, bounded contact shadow without a cache.
    for (rx, ry, alpha) in [(27.0, 6.4, 10), (22.0, 4.9, 14), (15.0, 3.0, 15)] {
        let center = d.pt(rig.ground + Vec2::new(0.0, 2.0));
        ellipse(
            d.pixels,
            center,
            rx * scale,
            ry * scale,
            &paint((40, 44, 42), alpha),
        );
    }
    let mut feet = rig.feet_pose;
    if feet[0].pos.y > feet[1].pos.y {
        feet.swap(0, 1);
    }
    for foot in feet {
        let ankle = foot.pos - Vec2::new(0.0, foot.lift);
        let hip = Vec2::new(
            rig.body_center.x + (foot.pos.x - rig.ground.x) * 0.4,
            rig.body_center.y + 13.0,
        );
        let knee = Vec2::lerp(hip, ankle, 0.55) + local(-2.0, 0.0, 0.0);
        d.segment(hip, knee, pal.goose_orange_dark, 2.3);
        d.segment(knee, ankle, pal.goose_orange, 2.3);
        let v = foot.heading;
        let side = Vec2::new(-v.y, v.x);
        let point = |a, b| ankle + Vec2::new((v * a + side * b).x, (v * a + side * b).y * 0.65);
        let web = d.path(&[
            point(-2.0, -1.7),
            point(-1.0, -2.3),
            point(3.0, -4.1),
            point(6.0, -4.0),
            point(6.6, -3.5),
            point(5.5, -1.6),
            point(8.0, -1.0),
            point(8.6, 0.1),
            point(7.0, 0.7),
            point(5.6, 1.4),
            point(5.7, 3.0),
            point(5.1, 3.8),
            point(3.8, 3.1),
            point(1.2, 2.4),
            point(-2.2, 2.5),
            point(-2.0, -1.7),
        ]);
        d.line(&web, pal.goose_orange_dark, 1.2);
        d.fill(&web, pal.goose_orange);
    }
    let tail_base = rig.body_center + local(-18.0, 0.0, 2.0);
    let tail_tip = rig.body_center + local(-31.0, 0.0, 8.0 + rig.tail_flick * 5.0);
    let tail = d.path(&[
        tail_base + Vec2::new(0.0, -7.0),
        tail_base + local(-5.0, 0.0, 5.0),
        tail_tip + Vec2::new(0.0, -3.0),
        tail_tip,
        tail_tip + Vec2::new(0.0, 5.0),
        tail_base + Vec2::new(0.0, 9.0),
        tail_base + Vec2::new(0.0, 8.0),
        tail_base,
        tail_base,
        tail_base + Vec2::new(0.0, -7.0),
    ]);
    let body_rx = (25.5_f32.powi(2) * f.x * f.x + 18.0_f32.powi(2) * f.y * f.y).sqrt();
    let body_ry = 19.5 + f.y.abs() * 1.0 + rig.breath * 0.25;
    let body = d.oval(rig.body_center, body_rx, body_ry);
    let neck = d.path(&[
        rig.neck_base + Vec2::new(-8.4, 2.0),
        rig.neck_c1 + Vec2::new(-6.6, 0.0),
        rig.neck_c2 + Vec2::new(-6.8, 0.0),
        rig.neck_head + Vec2::new(-7.5, 0.0),
        rig.neck_head,
        rig.neck_head,
        rig.neck_head + Vec2::new(7.5, 0.0),
        rig.neck_c2 + Vec2::new(6.8, 0.0),
        rig.neck_c1 + Vec2::new(7.4, 0.0),
        rig.neck_base + Vec2::new(10.0, 2.0),
        rig.neck_base,
        rig.neck_base,
        rig.neck_base + Vec2::new(-8.4, 2.0),
    ]);
    let head = d.oval(rig.neck_head, 12.6 + f.x.abs() * 0.8, 12.3);
    // Shared silhouette pass covers internal part outlines; the neck grows out of the body.
    for path in [&tail, &body, &neck, &head] {
        d.line(path, edge, 2.0);
    }
    for path in [&tail, &body, &neck, &head] {
        d.fill(path, pal.goose_white);
    }
    // A restrained crescent under the belly, followed by the same body inset, leaves
    // the broad white silhouette intact on light and dark desktops.
    let belly = d.path(&[
        rig.body_center + Vec2::new(-body_rx * 0.9, 5.0),
        rig.body_center + Vec2::new(-body_rx * 0.65, 23.0),
        rig.body_center + Vec2::new(body_rx * 0.72, 23.0),
        rig.body_center + Vec2::new(body_rx * 0.98, 1.0),
        rig.body_center + Vec2::new(body_rx * 0.62, 16.0),
        rig.body_center + Vec2::new(-body_rx * 0.50, 18.5),
        rig.body_center + Vec2::new(-body_rx * 0.9, 5.0),
    ]);
    d.fill(&belly, shade);
    for sign in [-1.0, 1.0] {
        if sign * f.x < -0.20 {
            continue;
        }
        let center = rig.body_center + local(-2.0, sign * 14.0, 0.0);
        let p = |a, b| center + local(a, 0.0, b);
        let wing_path = d.path(&[
            p(11.0, 5.0),
            p(5.0, 9.0),
            p(-9.0, 9.0),
            p(-16.0, 3.0),
            p(-13.0, -4.0),
            p(4.0, -10.0),
            p(11.0, 5.0),
        ]);
        d.fill(&wing_path, wing);
        // Open crease (do not close its curve into a second outlined wing).
        let mut b = PathBuilder::new();
        let points = [p(-11.0, 0.0), p(-4.0, -5.0), p(5.0, -5.0), p(9.0, 2.0)].map(|p| d.pt(p));
        b.move_to(points[0].x, points[0].y);
        b.cubic_to(
            points[1].x,
            points[1].y,
            points[2].x,
            points[2].y,
            points[3].x,
            points[3].y,
        );
        d.line(
            &b.finish().expect("wing crease"),
            mix(wing, pal.goose_shade, 0.5),
            0.85,
        );
    }
    // The beak sits behind the skull when looking away, and in front when facing us.
    // Its base is buried in the skull; foreshortening is the same projection as the rig.
    let beak = |side, along, up| rig.neck_head + local(along, side, up - rig.head_tilt * along);
    // Project a solid ellipsoid bill. Unlike an overlapping dorsal/underside path,
    // this remains filled at front and rear headings and cannot make a hollow smile.
    let center = d.pt(beak(0.0, 15.0, -2.0));
    let xx = 64.0 * f.x * f.x + 36.0 * f.y * f.y;
    let yy = (64.0 * f.y * f.y + 36.0 * f.x * f.x) * 0.45 * 0.45 + 2.8 * 2.8;
    let xy = (64.0 - 36.0) * 0.45 * f.x * f.y;
    let angle = 0.5 * (2.0 * xy).atan2(xx - yy);
    let spread = ((xx - yy).powi(2) + 4.0 * xy * xy).sqrt();
    let rx = ((xx + yy + spread) * 0.5).sqrt() * scale;
    let ry = ((xx + yy - spread) * 0.5).sqrt() * scale;
    let (sin, cos) = angle.sin_cos();
    let bill = PathBuilder::from_circle(0.0, 0.0, 1.0)
        .unwrap()
        .transform(tiny_skia::Transform::from_row(
            rx * cos,
            rx * sin,
            -ry * sin,
            ry * cos,
            center.x,
            center.y,
        ))
        .unwrap();
    let jaw_drop = rig.beak_open * 4.5;
    if jaw_drop > 0.05 {
        let mouth = d.oval(
            rig.snout_center + Vec2::new(0.0, jaw_drop * 0.6),
            4.5 + f.x.abs() * 4.0,
            1.4 + jaw_drop * 0.55,
        );
        d.fill(&mouth, (86, 46, 29));
        let lower = d.oval(
            rig.snout_center + Vec2::new(0.0, jaw_drop + 1.2),
            4.8 + f.x.abs() * 4.2,
            1.4,
        );
        d.fill(&lower, pal.goose_orange_dark);
    }
    d.line(&bill, pal.goose_orange_dark, 0.75);
    d.fill(&bill, pal.goose_orange);
    if f.y < -0.1 {
        d.fill(&head, pal.goose_white);
    }
    for sign in [-1.0, 1.0] {
        let visibility = (f.y * 0.8 + sign * f.x) * 2.0;
        if visibility <= 0.0 {
            continue;
        }
        let eye = rig.neck_head + local(4.0, sign * 8.2, 4.5);
        let c = d.pt(eye);
        let alpha = (visibility.min(1.0) * 255.0) as u8;
        if rig.pleased > 0.25 {
            let mut p = PathBuilder::new();
            p.move_to(c.x - 1.8 * scale, c.y + 0.5 * scale);
            p.quad_to(c.x, c.y - 1.8 * scale, c.x + 1.8 * scale, c.y + 0.5 * scale);
            stroke(
                d.pixels,
                &p.finish().expect("happy eye"),
                &paint((38, 42, 39), alpha),
                1.25 * scale,
            );
        } else {
            ellipse(
                d.pixels,
                c,
                1.6 * scale,
                (1.9 * (1.0 - rig.blink)).max(0.3) * scale,
                &paint((31, 36, 34), alpha),
            );
            if rig.blink < 0.5 {
                disc(
                    d.pixels,
                    c + Vec2::new(-0.4, -0.5) * scale,
                    0.43 * scale,
                    &paint((255, 255, 255), alpha),
                );
            }
        }
        if f.y > -0.2 && visibility > 0.3 {
            let nostril = d.pt(beak(sign * 3.0, 15.0, -0.4));
            ellipse(
                d.pixels,
                nostril,
                0.65 * scale,
                0.48 * scale,
                &paint(pal.goose_orange_dark, 190),
            );
        }
    }
}
