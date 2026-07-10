//! Platform-neutral desktop monitor topology.

use crate::math::{Rect, Vec2};
use crate::rng::RandomSource;

/// Why a monitor-region layout could not be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopLayoutError {
    /// A desktop needs at least one visible region.
    Empty,
    /// Every region must have finite coordinates and positive area.
    InvalidRegion,
}

impl std::fmt::Display for DesktopLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("desktop layout has no regions"),
            Self::InvalidRegion => f.write_str("desktop layout contains an invalid region"),
        }
    }
}

impl std::error::Error for DesktopLayoutError {}

/// The visible desktop as real monitor rectangles, rather than one filled outer box.
///
/// `bounds` remains useful for off-screen excursion edges; all ordinary target sampling
/// and clamping uses `regions`, so L-shaped and gapped monitor arrangements remain honest.
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopLayout {
    regions: Vec<Rect>,
    adjacency: Vec<Vec<usize>>,
    bounds: Rect,
}

impl DesktopLayout {
    /// Validate and build a layout from monitor rectangles.
    pub fn new(regions: Vec<Rect>) -> Result<Self, DesktopLayoutError> {
        if regions.is_empty() {
            return Err(DesktopLayoutError::Empty);
        }
        if regions.iter().any(|r| {
            !r.min.x.is_finite()
                || !r.min.y.is_finite()
                || !r.max.x.is_finite()
                || !r.max.y.is_finite()
                || r.width() <= 0.0
                || r.height() <= 0.0
        }) {
            return Err(DesktopLayoutError::InvalidRegion);
        }

        let bounds = regions[1..].iter().copied().fold(regions[0], Rect::union);
        let mut adjacency = vec![Vec::new(); regions.len()];
        for i in 0..regions.len() {
            for j in (i + 1)..regions.len() {
                if regions_touch(regions[i], regions[j]) {
                    adjacency[i].push(j);
                    adjacency[j].push(i);
                }
            }
        }
        Ok(Self {
            regions,
            adjacency,
            bounds,
        })
    }

    /// Compatibility constructor for the historical one-rectangle desktop model.
    pub fn single(bounds: Rect) -> Self {
        Self::new(vec![bounds]).expect("single desktop bounds must be finite and non-empty")
    }

    /// Actual visible monitor rectangles.
    pub fn regions(&self) -> &[Rect] {
        &self.regions
    }

    /// Region indices touching or overlapping `index`.
    pub fn adjacent(&self, index: usize) -> &[usize] {
        self.adjacency.get(index).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Outer union bounds, including any gaps between monitors.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Whether a point belongs to any visible monitor region.
    pub fn contains(&self, point: Vec2) -> bool {
        self.regions.iter().any(|region| region.contains(point))
    }

    /// Sample uniformly by visible area, never from an inter-monitor gap.
    pub fn sample_point(&self, rng: &mut impl RandomSource) -> Vec2 {
        self.sample_point_inset(rng, 0.0)
    }

    /// Sample from a monitor region with a proportional safe inset on every edge.
    pub fn sample_point_inset(&self, rng: &mut impl RandomSource, inset: f32) -> Vec2 {
        let total_area: f64 = self
            .regions
            .iter()
            .map(|r| r.width() as f64 * r.height() as f64)
            .sum();
        let mut draw = rng.next_f64() * total_area;
        let mut chosen = *self.regions.last().expect("validated non-empty layout");
        for region in &self.regions {
            let area = region.width() as f64 * region.height() as f64;
            if draw < area {
                chosen = *region;
                break;
            }
            draw -= area;
        }
        let inset = inset.clamp(0.0, 0.49);
        let dx = chosen.width() * inset;
        let dy = chosen.height() * inset;
        Vec2::new(
            rng.range(chosen.min.x + dx, chosen.max.x - dx),
            rng.range(chosen.min.y + dy, chosen.max.y - dy),
        )
    }

    /// Clamp to the nearest visible region (a point already visible is unchanged).
    pub fn clamp_point(&self, point: Vec2) -> Vec2 {
        self.regions
            .iter()
            .map(|region| {
                let clamped = Vec2::new(
                    point.x.clamp(region.min.x, region.max.x),
                    point.y.clamp(region.min.y, region.max.y),
                );
                let delta = point - clamped;
                (clamped, Vec2::dot(delta, delta))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(point, _)| point)
            .expect("validated non-empty layout")
    }

    /// Clip a rectangle to visible regions, returning the bounding union of visible pieces.
    pub fn clip_rect(&self, rect: Rect) -> Option<Rect> {
        self.regions
            .iter()
            .filter_map(|region| rect.intersection(*region))
            .reduce(Rect::union)
    }
}

fn regions_touch(a: Rect, b: Rect) -> bool {
    let separated_x = a.max.x < b.min.x || b.max.x < a.min.x;
    let separated_y = a.max.y < b.min.y || b.max.y < a.min.y;
    !separated_x && !separated_y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacency_tracks_touching_regions_not_gaps() {
        let layout = DesktopLayout::new(vec![
            Rect::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
            Rect::new(Vec2::new(100.0, 0.0), Vec2::new(200.0, 100.0)),
            Rect::new(Vec2::new(300.0, 0.0), Vec2::new(400.0, 100.0)),
        ])
        .expect("layout");
        assert_eq!(layout.adjacent(0), &[1]);
        assert_eq!(layout.adjacent(1), &[0]);
        assert!(layout.adjacent(2).is_empty());
    }
}
