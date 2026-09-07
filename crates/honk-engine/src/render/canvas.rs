use tiny_skia::{Color, Pixmap};

/// Reusable transparent damage surface. Small size jitter reuses an allocation; a large
/// temporary trail cannot leave an oversized surface being cleared forever afterward.
#[derive(Default)]
pub struct DamageCanvas {
    pixmap: Option<Pixmap>,
    allocations: usize,
}

impl DamageCanvas {
    pub fn prepare(&mut self, width: u32, height: u32) -> Result<&mut Pixmap, &'static str> {
        let rounded = |extent: u32| extent.max(1).checked_add(31).map(|value| value / 32 * 32);
        let requested_width = rounded(width).ok_or("damage canvas width overflow")?;
        let requested_height = rounded(height).ok_or("damage canvas height overflow")?;
        let resize = |current: u32, requested: u32| {
            if current < requested || current > requested.saturating_mul(2) {
                requested
            } else {
                current
            }
        };
        let (next_width, next_height) =
            self.pixmap
                .as_ref()
                .map_or((requested_width, requested_height), |canvas| {
                    (
                        resize(canvas.width(), requested_width),
                        resize(canvas.height(), requested_height),
                    )
                });
        if self
            .pixmap
            .as_ref()
            .is_none_or(|canvas| canvas.width() != next_width || canvas.height() != next_height)
        {
            let next = Pixmap::new(next_width, next_height)
                .ok_or("could not allocate dirty overlay canvas")?;
            self.pixmap = Some(next);
            self.allocations += 1;
        }
        let canvas = self.pixmap.as_mut().ok_or("missing damage canvas")?;
        canvas.fill(Color::TRANSPARENT);
        Ok(canvas)
    }

    pub fn allocations(&self) -> usize {
        self.allocations
    }

    pub fn retained_bytes(&self) -> usize {
        self.pixmap.as_ref().map_or(0, |canvas| canvas.data().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_jittering_bounds_and_clears_old_pixels() {
        let mut owner = DamageCanvas::default();
        let canvas = owner.prepare(257, 129).unwrap();
        let address = canvas.data().as_ptr();
        canvas.fill(Color::WHITE);
        for _ in 0..300 {
            let canvas = owner.prepare(250, 120).unwrap();
            assert_eq!(canvas.data().as_ptr(), address);
            assert!(canvas.data().iter().all(|&byte| byte == 0));
        }
        assert_eq!(owner.allocations(), 1);
    }

    #[test]
    fn shrinks_each_extent_after_transient_damage_and_grows_again() {
        let mut owner = DamageCanvas::default();
        owner.prepare(3840, 2160).unwrap();
        let large = owner.retained_bytes();
        let small = owner.prepare(320, 300).unwrap();
        assert_eq!((small.width(), small.height()), (320, 320));
        assert!(owner.retained_bytes() < large / 10);
        let grown = owner.prepare(640, 300).unwrap();
        assert_eq!((grown.width(), grown.height()), (640, 320));
        assert_eq!(owner.allocations(), 3);
    }

    #[test]
    fn rejects_overflow_without_discarding_the_current_buffer() {
        let mut owner = DamageCanvas::default();
        owner.prepare(20, 20).unwrap();
        let retained = owner.retained_bytes();
        assert!(owner.prepare(u32::MAX, 20).is_err());
        assert_eq!(owner.retained_bytes(), retained);
    }
}
