use crate::v4l2_capture::RawFrame;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureKind {
    Open,
    Pinch,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandState {
    pub gesture: GestureKind,
    pub x_norm: f32,
    pub y_norm: f32,
    pub openness: f32,
    pub confidence: f32,
}

pub struct HandTracker {
    pub smoothing_factor: f32,
    pub deadband: f32,
    prev_x: f32,
    prev_y: f32,
    last_click_time: Option<Instant>,
    click_debounce: Duration,
}

impl HandTracker {
    pub fn new() -> Self {
        Self {
            smoothing_factor: 0.70, // 0.70 EMA smoothing
            deadband: 0.005,        // 0.5% screen deadband for jitter reduction
            prev_x: 0.5,
            prev_y: 0.5,
            last_click_time: None,
            click_debounce: Duration::from_millis(200),
        }
    }

    pub fn with_smoothing(mut self, factor: f32) -> Self {
        self.smoothing_factor = factor.clamp(0.05, 0.95);
        self
    }

    /// Process a raw luma frame, segment skin cluster, apply EMA smoothing and click debouncing.
    pub fn process_frame(&mut self, frame: &RawFrame) -> HandState {
        let bbox = self.find_skin_bounding_box(&frame.data, frame.width, frame.height);
        match bbox {
            None => HandState {
                gesture: GestureKind::None,
                x_norm: self.prev_x,
                y_norm: self.prev_y,
                openness: 0.0,
                confidence: 0.0,
            },
            Some((x0, y0, x1, y1)) => {
                let raw_cx = (x0 + x1) as f32 / 2.0;
                let raw_cy = (y0 + y1) as f32 / 2.0;
                let raw_x_norm = (raw_cx / frame.width as f32).clamp(0.0, 1.0);
                let raw_y_norm = (raw_cy / frame.height as f32).clamp(0.0, 1.0);

                let bbox_area = ((x1 - x0) * (y1 - y0)) as f32;
                let skin_count = self.count_skin_pixels(&frame.data, frame.width, x0, y0, x1, y1) as f32;

                let openness = if bbox_area > 0.0 {
                    (skin_count / bbox_area).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let confidence = (skin_count / 8000.0).clamp(0.0, 1.0);

                // Apply Deadband & EMA Smoothing Filter
                let dx = raw_x_norm - self.prev_x;
                let dy = raw_y_norm - self.prev_y;

                let x_norm = if dx.abs() > self.deadband {
                    self.smoothing_factor * raw_x_norm + (1.0 - self.smoothing_factor) * self.prev_x
                } else {
                    self.prev_x
                };

                let y_norm = if dy.abs() > self.deadband {
                    self.smoothing_factor * raw_y_norm + (1.0 - self.smoothing_factor) * self.prev_y
                } else {
                    self.prev_y
                };

                self.prev_x = x_norm;
                self.prev_y = y_norm;

                // Gesture Classification with Debounce
                let mut gesture = if openness > 0.65 {
                    GestureKind::Open
                } else if openness < 0.45 {
                    GestureKind::Pinch
                } else {
                    GestureKind::Open
                };

                if gesture == GestureKind::Pinch {
                    let now = Instant::now();
                    if let Some(last) = self.last_click_time {
                        if now.duration_since(last) < self.click_debounce {
                            // Suppress repeated pinch within debounce window
                            gesture = GestureKind::Open;
                        } else {
                            self.last_click_time = Some(now);
                        }
                    } else {
                        self.last_click_time = Some(now);
                    }
                }

                HandState {
                    gesture,
                    x_norm,
                    y_norm,
                    openness,
                    confidence,
                }
            }
        }
    }

    fn count_skin_pixels(
        &self,
        data: &[u8],
        width: u32,
        x0: u32,
        y0: u32,
        x1: u32,
        y1: u32,
    ) -> u32 {
        let mut count = 0u32;
        for y in y0..y1 {
            for x in x0..x1 {
                let idx = (y * width + x) as usize;
                if idx < data.len() {
                    let v = data[idx];
                    if (80..=200).contains(&v) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn find_skin_bounding_box(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Option<(u32, u32, u32, u32)> {
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0u32;
        let mut max_y = 0u32;
        let mut found = false;

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                if idx < data.len() {
                    let v = data[idx];
                    if (80..=200).contains(&v) {
                        found = true;
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
            }
        }

        if found && max_x > min_x && max_y > min_y {
            Some((min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }
}

impl Default for HandTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_hand_dark_frame() {
        let mut tracker = HandTracker::new();
        let frame = RawFrame {
            data: vec![10u8; 64 * 64],
            width: 64,
            height: 64,
            timestamp_ms: 0,
        };
        let state = tracker.process_frame(&frame);
        assert_eq!(state.gesture, GestureKind::None);
        assert_eq!(state.confidence, 0.0);
    }

    #[test]
    fn test_open_hand_center() {
        let mut tracker = HandTracker::new();
        let mut data = vec![10u8; 64 * 64];
        // 20x20 central skin cluster
        for y in 22..42 {
            for x in 22..42 {
                data[y * 64 + x] = 140;
            }
        }
        let frame = RawFrame {
            data,
            width: 64,
            height: 64,
            timestamp_ms: 0,
        };
        let state = tracker.process_frame(&frame);
        assert_eq!(state.gesture, GestureKind::Open);
        assert!((state.x_norm - 0.5).abs() < 0.1);
        assert!((state.y_norm - 0.5).abs() < 0.1);
        assert!(state.openness > 0.6);
    }

    #[test]
    fn test_smoothing_ema() {
        let mut tracker = HandTracker::new().with_smoothing(0.5);
        let mut data1 = vec![10u8; 64 * 64];
        for y in 10..30 {
            for x in 10..30 {
                data1[y * 64 + x] = 140;
            }
        }
        let frame1 = RawFrame {
            data: data1,
            width: 64,
            height: 64,
            timestamp_ms: 0,
        };
        let state1 = tracker.process_frame(&frame1);

        // Frame with hand shifted to the right
        let mut data2 = vec![10u8; 64 * 64];
        for y in 10..30 {
            for x in 30..50 {
                data2[y * 64 + x] = 140;
            }
        }
        let frame2 = RawFrame {
            data: data2,
            width: 64,
            height: 64,
            timestamp_ms: 33,
        };
        let state2 = tracker.process_frame(&frame2);

        // state2.x_norm should be between state1.x_norm and raw target due to EMA smoothing
        assert!(state2.x_norm > state1.x_norm);
    }
}
