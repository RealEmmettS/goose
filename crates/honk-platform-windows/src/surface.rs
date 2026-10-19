//! Checked source regions and bounded capacity for the native layered presenter.
//! Logical dimensions stay separate from DIB capacity and its padded row stride.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn whole(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    pub fn validate(self, width: u32, height: u32) -> Result<(), &'static str> {
        if self.width == 0
            || self.height == 0
            || self
                .x
                .checked_add(self.width)
                .is_none_or(|right| right > width)
            || self
                .y
                .checked_add(self.height)
                .is_none_or(|bottom| bottom > height)
        {
            return Err("invalid layered source region");
        }
        Ok(())
    }
}

pub(crate) fn byte_len(width: u32, height: u32) -> Result<usize, &'static str> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|&n| n <= isize::MAX as usize)
        .ok_or("layered bitmap byte size overflow")
}

pub(crate) fn capacity(
    current: Option<(u32, u32)>,
    width: u32,
    height: u32,
) -> Result<(u32, u32), &'static str> {
    let round = |extent: u32| {
        extent
            .max(1)
            .checked_add(31)
            .map(|n| n / 32 * 32)
            .filter(|&n| n <= i32::MAX as u32)
            .ok_or("layered bitmap extent overflow")
    };
    let required = (round(width)?, round(height)?);
    let resize = |old: u32, required: u32| {
        if old < required || old > required.saturating_mul(2) {
            required
        } else {
            old
        }
    };
    let next = current.map_or(required, |old| {
        (resize(old.0, required.0), resize(old.1, required.1))
    });
    byte_len(next.0, next.1)?;
    Ok(next)
}

/// Copy the complete logical viewport, including its transparent pixels. No temporary image
/// is allocated. Padding and rows outside the viewport are not presented; when it grows,
/// every newly exposed pixel is overwritten here before UpdateLayeredWindow can expose it.
pub(crate) fn copy_region(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    region: Region,
    destination: &mut [u8],
    destination_width: u32,
    destination_height: u32,
) -> Result<(), &'static str> {
    region.validate(source_width, source_height)?;
    if region.width > destination_width
        || region.height > destination_height
        || source.len() != byte_len(source_width, source_height)?
        || destination.len() != byte_len(destination_width, destination_height)?
    {
        return Err("layered bitmap source or destination size mismatch");
    }
    let source_stride = source_width as usize * 4;
    let destination_stride = destination_width as usize * 4;
    let row_bytes = region.width as usize * 4;
    for row in 0..region.height as usize {
        let start = (region.y as usize + row) * source_stride + region.x as usize * 4;
        let output = row * destination_stride;
        for (rgba, bgra) in source[start..start + row_bytes]
            .chunks_exact(4)
            .zip(destination[output..output + row_bytes].chunks_exact_mut(4))
        {
            bgra.copy_from_slice(&[rgba[2], rgba[1], rgba[0], rgba[3]]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_reuses_jitter_and_shrinks_each_extent_independently() {
        assert_eq!(capacity(None, 257, 129), Ok((288, 160)));
        assert_eq!(capacity(Some((288, 160)), 250, 120), Ok((288, 160)));
        assert_eq!(capacity(Some((3840, 2160)), 320, 300), Ok((320, 320)));
        assert_eq!(capacity(Some((320, 320)), 640, 300), Ok((640, 320)));
        assert_eq!(capacity(Some((640, 320)), 320, 300), Ok((640, 320)));
        assert!(capacity(None, u32::MAX, 20).is_err());
        assert!(capacity(None, i32::MAX as u32, 20).is_err());
    }

    #[test]
    fn crop_offsets_and_padded_stride_preserve_premultiplied_channels() {
        let mut source = vec![0; 4 * 3 * 4];
        source[20..24].copy_from_slice(&[80, 40, 10, 100]);
        source[24..28].copy_from_slice(&[0, 0, 0, 0]);
        source[36..40].copy_from_slice(&[10, 20, 30, 40]);
        source[40..44].copy_from_slice(&[255, 100, 50, 255]);
        let mut destination = vec![7; 8 * 4 * 4];
        copy_region(
            &source,
            4,
            3,
            Region {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            },
            &mut destination,
            8,
            4,
        )
        .unwrap();
        assert_eq!(&destination[..8], &[10, 40, 80, 100, 0, 0, 0, 0]);
        assert_eq!(&destination[32..40], &[30, 20, 10, 40, 50, 100, 255, 255]);
        assert!(destination[8..32].iter().all(|&n| n == 7));
        assert!(destination[40..].iter().all(|&n| n == 7));
    }

    #[test]
    fn expanding_viewport_overwrites_stale_pixels_and_final_clear_is_transparent() {
        let mut destination = vec![255; 32 * 32 * 4];
        let small = [0, 0, 0, 0];
        copy_region(&small, 1, 1, Region::whole(1, 1), &mut destination, 32, 32).unwrap();
        assert_eq!(&destination[..4], &[0; 4]);
        let cleared = vec![0; 31 * 31 * 4];
        copy_region(
            &cleared,
            31,
            31,
            Region::whole(31, 31),
            &mut destination,
            32,
            32,
        )
        .unwrap();
        for row in 0..31 {
            assert!(destination[row * 128..row * 128 + 124]
                .iter()
                .all(|&n| n == 0));
        }
        copy_region(
            &vec![0; 32 * 32 * 4],
            32,
            32,
            Region::whole(32, 32),
            &mut destination,
            32,
            32,
        )
        .unwrap();
        assert!(destination.iter().all(|&n| n == 0));
    }

    #[test]
    fn rejects_invalid_ranges_before_touching_the_destination() {
        for region in [
            Region {
                x: u32::MAX,
                y: 0,
                width: 2,
                height: 1,
            },
            Region {
                x: 1,
                y: 0,
                width: 2,
                height: 1,
            },
            Region::whole(0, 1),
        ] {
            let mut output = [99; 16];
            assert!(copy_region(&[0; 16], 2, 2, region, &mut output, 2, 2).is_err());
            assert_eq!(output, [99; 16]);
        }
        assert!(copy_region(&[0; 15], 2, 2, Region::whole(2, 2), &mut [0; 16], 2, 2).is_err());
        assert!(copy_region(&[0; 16], 2, 2, Region::whole(2, 2), &mut [0; 4], 1, 1).is_err());
    }
}
