//! swal-render-pipeline
//! High-refresh (200Hz - 240Hz) lock-free render coordinator for SWAL Desktop

pub mod orb_surface;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct FrameScheduler {
    pub target_fps: u32,
    pub frame_budget: Duration,
    frame_counter: Arc<AtomicU64>,
}

impl FrameScheduler {
    pub fn new(target_fps: u32) -> Self {
        let nanos_per_frame = 1_000_000_000 / target_fps as u64;
        Self {
            target_fps,
            frame_budget: Duration::from_nanos(nanos_per_frame),
            frame_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Calculates if a frame render meets the 200Hz (5.0ms) / 240Hz (4.16ms) budget
    pub fn benchmark_render_tick<F>(&self, render_fn: F) -> (Duration, bool)
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        render_fn();
        let elapsed = start.elapsed();
        self.frame_counter.fetch_add(1, Ordering::Relaxed);
        let within_budget = elapsed <= self.frame_budget;
        (elapsed, within_budget)
    }

    pub fn total_frames(&self) -> u64 {
        self.frame_counter.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_200hz_frame_budget() {
        let scheduler = FrameScheduler::new(200); // 200Hz = 5.0ms per frame
        assert_eq!(scheduler.frame_budget, Duration::from_millis(5));

        let (elapsed, within_budget) = scheduler.benchmark_render_tick(|| {
            // Simulated microsecond UI calculation
            let _val: u64 = (0..1000).sum();
        });

        assert!(within_budget, "Render tick must complete well under 5.0ms");
        assert!(elapsed < Duration::from_millis(1));
        assert_eq!(scheduler.total_frames(), 1);
    }
}
