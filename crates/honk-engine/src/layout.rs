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

/// Outward-facing side of a real monitor region.
///
/// Screen coordinates grow rightward and downward, so `Top` points toward negative Y.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    Left,
    Right,
    Top,
    Bottom,
}

impl EdgeDirection {
    /// Unit vector pointing away from the visible desktop at this edge.
    pub const fn outward(self) -> Vec2 {
        match self {
            Self::Left => Vec2::new(-1.0, 0.0),
            Self::Right => Vec2::new(1.0, 0.0),
            Self::Top => Vec2::new(0.0, -1.0),
            Self::Bottom => Vec2::new(0.0, 1.0),
        }
    }

    /// Unit vector pointing into the visible desktop at this edge.
    pub const fn inward(self) -> Vec2 {
        let outward = self.outward();
        Vec2::new(-outward.x, -outward.y)
    }

    /// The matching entry side for a Pac-Man-style wrap.
    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }

    const fn horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// A contiguous part of a monitor edge with no real monitor immediately beyond it.
///
/// Shared monitor seams are removed from these spans. That makes an exposed edge a safe
/// place to leave the desktop, while a touching monitor remains a natural walkable crossing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExposedEdge {
    region: usize,
    direction: EdgeDirection,
    coordinate: f32,
    span_min: f32,
    span_max: f32,
}

impl ExposedEdge {
    pub const fn region(self) -> usize {
        self.region
    }

    pub const fn direction(self) -> EdgeDirection {
        self.direction
    }

    pub fn length(self) -> f32 {
        self.span_max - self.span_min
    }

    /// Point on the edge nearest `reference`, kept away from a corner when the span permits.
    pub fn point_near(self, reference: Vec2, corner_inset: f32) -> Vec2 {
        let inset = corner_inset.max(0.0).min(self.length() * 0.5);
        let span_min = self.span_min + inset;
        let span_max = self.span_max - inset;
        let along = if self.direction.horizontal() {
            reference.y
        } else {
            reference.x
        }
        .clamp(span_min, span_max);
        if self.direction.horizontal() {
            Vec2::new(self.coordinate, along)
        } else {
            Vec2::new(along, self.coordinate)
        }
    }

    /// Midpoint of this exposed span.
    pub fn midpoint(self) -> Vec2 {
        let along = (self.span_min + self.span_max) * 0.5;
        if self.direction.horizontal() {
            Vec2::new(self.coordinate, along)
        } else {
            Vec2::new(along, self.coordinate)
        }
    }

    fn orthogonal_distance(self, point: Vec2) -> f32 {
        let along = if self.direction.horizontal() {
            point.y
        } else {
            point.x
        };
        if along < self.span_min {
            self.span_min - along
        } else if along > self.span_max {
            along - self.span_max
        } else {
            0.0
        }
    }
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
    exposed_edges: Vec<ExposedEdge>,
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
        let exposed_edges = build_exposed_edges(&regions);
        Ok(Self {
            regions,
            adjacency,
            exposed_edges,
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

    /// All outer and gap-facing edge spans. Touching-monitor seams are intentionally absent.
    pub fn exposed_edges(&self) -> &[ExposedEdge] {
        &self.exposed_edges
    }

    /// Exposed edge nearest a world-space point.
    pub fn nearest_exposed_edge(&self, point: Vec2) -> ExposedEdge {
        self.exposed_edges
            .iter()
            .copied()
            .min_by(|a, b| {
                let pa = a.point_near(point, 0.0);
                let pb = b.point_near(point, 0.0);
                let da = Vec2::dot(point - pa, point - pa);
                let db = Vec2::dot(point - pb, point - pb);
                da.total_cmp(&db)
            })
            .expect("a finite non-empty desktop always has an exposed boundary")
    }

    /// Edge-length-weighted random exposed edge, optionally restricted to one monitor region.
    pub fn sample_exposed_edge(
        &self,
        rng: &mut impl RandomSource,
        region: Option<usize>,
    ) -> ExposedEdge {
        let candidates: Vec<_> = self
            .exposed_edges
            .iter()
            .copied()
            .filter(|edge| region.is_none_or(|region| edge.region == region))
            .collect();
        let candidates = if candidates.is_empty() {
            &self.exposed_edges[..]
        } else {
            &candidates[..]
        };
        let total: f64 = candidates.iter().map(|edge| edge.length() as f64).sum();
        let mut draw = rng.next_f64() * total;
        for edge in candidates {
            if draw < edge.length() as f64 {
                return *edge;
            }
            draw -= edge.length() as f64;
        }
        *candidates.last().expect("exposed edge candidates")
    }

    /// Pick the far-side exposed entry corresponding to an exit edge.
    ///
    /// The farthest boundary in the wrap direction wins; ties preserve the closest corridor.
    pub fn opposite_exposed_edge(&self, exit: ExposedEdge, reference: Vec2) -> ExposedEdge {
        let direction = exit.direction.opposite();
        self.exposed_edges
            .iter()
            .copied()
            .filter(|edge| edge.direction == direction)
            .min_by(|a, b| {
                let extreme_order = match direction {
                    EdgeDirection::Left | EdgeDirection::Top => {
                        a.coordinate.total_cmp(&b.coordinate)
                    }
                    EdgeDirection::Right | EdgeDirection::Bottom => {
                        b.coordinate.total_cmp(&a.coordinate)
                    }
                };
                if extreme_order == std::cmp::Ordering::Equal {
                    a.orthogonal_distance(reference)
                        .total_cmp(&b.orthogonal_distance(reference))
                } else {
                    extreme_order
                }
            })
            .expect("every exposed direction has an opposite desktop boundary")
    }

    /// Region containing `point`, if any.
    pub fn region_at(&self, point: Vec2) -> Option<usize> {
        self.regions
            .iter()
            .position(|region| region.contains(point))
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

fn build_exposed_edges(regions: &[Rect]) -> Vec<ExposedEdge> {
    let mut edges = Vec::new();
    for (region_index, region) in regions.iter().copied().enumerate() {
        for direction in [
            EdgeDirection::Left,
            EdgeDirection::Right,
            EdgeDirection::Top,
            EdgeDirection::Bottom,
        ] {
            let (coordinate, span_min, span_max) = match direction {
                EdgeDirection::Left => (region.min.x, region.min.y, region.max.y),
                EdgeDirection::Right => (region.max.x, region.min.y, region.max.y),
                EdgeDirection::Top => (region.min.y, region.min.x, region.max.x),
                EdgeDirection::Bottom => (region.max.y, region.min.x, region.max.x),
            };
            let mut spans = vec![(span_min, span_max)];
            for (other_index, other) in regions.iter().copied().enumerate() {
                if other_index == region_index || !blocks_edge(other, direction, coordinate) {
                    continue;
                }
                let blocker = if direction.horizontal() {
                    (other.min.y, other.max.y)
                } else {
                    (other.min.x, other.max.x)
                };
                spans = subtract_interval(spans, blocker);
                if spans.is_empty() {
                    break;
                }
            }
            edges.extend(spans.into_iter().filter_map(|(span_min, span_max)| {
                (span_max > span_min).then_some(ExposedEdge {
                    region: region_index,
                    direction,
                    coordinate,
                    span_min,
                    span_max,
                })
            }));
        }
    }
    edges
}

fn blocks_edge(other: Rect, direction: EdgeDirection, coordinate: f32) -> bool {
    // A display anywhere along the outward ray blocks the overlapping part of this face, even
    // when there is an OS-coordinate gap between them. Crossing that face should traverse onto
    // the farther display naturally; wrapping/staging is reserved for the outer skyline.
    match direction {
        EdgeDirection::Left => other.min.x < coordinate,
        EdgeDirection::Right => other.max.x > coordinate,
        EdgeDirection::Top => other.min.y < coordinate,
        EdgeDirection::Bottom => other.max.y > coordinate,
    }
}

fn subtract_interval(spans: Vec<(f32, f32)>, blocker: (f32, f32)) -> Vec<(f32, f32)> {
    spans
        .into_iter()
        .flat_map(|(start, end)| {
            let overlap_start = start.max(blocker.0);
            let overlap_end = end.min(blocker.1);
            if overlap_end <= overlap_start {
                vec![(start, end)]
            } else {
                let mut remaining = Vec::with_capacity(2);
                if overlap_start > start {
                    remaining.push((start, overlap_start));
                }
                if overlap_end < end {
                    remaining.push((overlap_end, end));
                }
                remaining
            }
        })
        .collect()
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

    #[test]
    fn single_monitor_exposes_all_four_sides() {
        let layout = DesktopLayout::single(Rect::new(Vec2::ZERO, Vec2::new(100.0, 80.0)));
        assert_eq!(layout.exposed_edges().len(), 4);
        for direction in [
            EdgeDirection::Left,
            EdgeDirection::Right,
            EdgeDirection::Top,
            EdgeDirection::Bottom,
        ] {
            assert!(layout
                .exposed_edges()
                .iter()
                .any(|edge| edge.direction() == direction));
        }
    }

    #[test]
    fn touching_monitor_corridor_is_not_exposed_but_uncovered_edge_is() {
        let layout = DesktopLayout::new(vec![
            Rect::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
            Rect::new(Vec2::new(100.0, 25.0), Vec2::new(200.0, 75.0)),
        ])
        .expect("layout");
        let right_spans: Vec<_> = layout
            .exposed_edges()
            .iter()
            .filter(|edge| edge.region() == 0 && edge.direction() == EdgeDirection::Right)
            .map(|edge| (edge.span_min, edge.span_max))
            .collect();
        assert_eq!(right_spans, vec![(0.0, 25.0), (75.0, 100.0)]);
        assert!(!right_spans
            .iter()
            .any(|(start, end)| *start <= 50.0 && *end >= 50.0));
    }

    #[test]
    fn gapped_monitor_corridor_is_not_an_exposed_wrap_edge() {
        let layout = DesktopLayout::new(vec![
            Rect::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
            Rect::new(Vec2::new(140.0, 20.0), Vec2::new(240.0, 120.0)),
        ])
        .expect("layout");
        let gap_facing_spans: Vec<_> = layout
            .exposed_edges()
            .iter()
            .filter(|edge| edge.region() == 0 && edge.direction() == EdgeDirection::Right)
            .map(|edge| (edge.span_min, edge.span_max))
            .collect();
        assert_eq!(gap_facing_spans, vec![(0.0, 20.0)]);
        assert!(gap_facing_spans
            .iter()
            .all(|(start, end)| !(start <= &50.0 && end >= &50.0)));
        assert!(layout.exposed_edges().iter().any(|edge| {
            edge.region() == 1
                && edge.direction() == EdgeDirection::Right
                && edge.span_min <= 50.0
                && edge.span_max >= 50.0
        }));
    }
}
