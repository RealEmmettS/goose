//! Bounded, opt-in developer frame measurements. Ordinary startup has no recording or clocks.
use honk_platform_windows::PresentationProfile;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const MAX_SAMPLES: usize = 65_536;

#[derive(Clone, Copy, Serialize)]
struct Sample {
    elapsed_ms: f64,
    render_us: f64,
    allocation_us: f64,
    copy_us: f64,
    update_us: f64,
    total_us: f64,
    bitmap_allocations: u64,
    presented_pixels: u64,
}

pub(super) struct FrameProfile {
    output: File,
    start: Instant,
    started_unix_ms: u128,
    samples: Vec<Sample>,
    frames: usize,
}

impl FrameProfile {
    pub fn from_env() -> Option<Self> {
        let path = std::env::var_os("HONK300_WINDOWS_PROFILE")?;
        if path.is_empty() {
            return None;
        }
        let output = match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("honk300: frame profile unavailable ({error})");
                return None;
            }
        };
        Some(Self {
            output,
            start: Instant::now(),
            started_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            samples: Vec::with_capacity(MAX_SAMPLES),
            frames: 0,
        })
    }

    pub fn record(&mut self, start: Instant, render_us: f64, native: PresentationProfile) {
        let sample = Sample {
            elapsed_ms: self.start.elapsed().as_secs_f64() * 1e3,
            render_us,
            allocation_us: native.allocation_us,
            copy_us: native.copy_us,
            update_us: native.update_us,
            total_us: start.elapsed().as_secs_f64() * 1e6,
            bitmap_allocations: native.bitmap_allocations,
            presented_pixels: native.pixels,
        };
        if self.samples.len() < MAX_SAMPLES {
            self.samples.push(sample);
        } else {
            self.samples[self.frames % MAX_SAMPLES] = sample;
        }
        self.frames += 1;
    }
}

impl Drop for FrameProfile {
    fn drop(&mut self) {
        self.samples
            .sort_by(|a, b| a.elapsed_ms.total_cmp(&b.elapsed_ms));
        let value = serde_json::json!({
            "schema": "honk300.windows-frame-profile.v1", "pid": std::process::id(),
            "started_unix_ms": self.started_unix_ms, "frames": self.frames,
            "retained_frames": self.samples.len(), "capacity": MAX_SAMPLES,
            "scope": "CPU render and real per-monitor presentation; clocks enabled only for this diagnostic run",
            "samples": self.samples,
        });
        if let Err(error) = serde_json::to_writer(&mut self.output, &value)
            .map_err(std::io::Error::other)
            .and_then(|()| self.output.sync_all())
        {
            eprintln!("honk300: frame profile could not be finalized ({error})");
        }
    }
}
