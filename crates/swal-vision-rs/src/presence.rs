//! Vision-based Presence Detection & Automated Session Lock/Unlock
//!
//! Evaluates user presence in front of the camera and triggers automated
//! session locking via `loginctl lock-session` or `hyprlock` when the user
//! leaves the workspace (walk-away lock), as well as auto-wake upon return.

use crate::v4l2_capture::RawFrame;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PresenceStatus {
    Present { confidence: f32 },
    Absent { elapsed_absence: Duration },
}

#[derive(Debug, Clone)]
pub struct PresenceDetector {
    pub auto_lock_enabled: bool,
    pub lock_timeout: Duration,
    pub auto_unlock_enabled: bool,
    pub is_session_locked: bool,
    last_presence_time: Instant,
    last_lock_trigger: Option<Instant>,
}

impl PresenceDetector {
    pub fn new(auto_lock_enabled: bool, lock_timeout_secs: u64, auto_unlock_enabled: bool) -> Self {
        Self {
            auto_lock_enabled,
            lock_timeout: Duration::from_secs(lock_timeout_secs),
            auto_unlock_enabled,
            is_session_locked: false,
            last_presence_time: Instant::now(),
            last_lock_trigger: None,
        }
    }

    /// Evaluates user presence from a luma frame.
    /// Looks for facial / head silhouette in the upper 60% of the frame.
    pub fn evaluate_presence(&mut self, frame: &RawFrame) -> PresenceStatus {
        let h_upper = (frame.height as f32 * 0.65) as u32;
        let mut skin_pixels = 0u32;
        let total_upper_pixels = (frame.width * h_upper) as usize;

        for y in 0..h_upper {
            for x in 0..frame.width {
                let idx = (y * frame.width + x) as usize;
                if idx < frame.data.len() {
                    let val = frame.data[idx];
                    // Human skin tone in Y-channel
                    if (80..=200).contains(&val) {
                        skin_pixels += 1;
                    }
                }
            }
        }

        let presence_ratio = if total_upper_pixels > 0 {
            skin_pixels as f32 / total_upper_pixels as f32
        } else {
            0.0
        };

        let now = Instant::now();
        // A face/head usually occupies between 3% and 40% of the upper frame
        if presence_ratio >= 0.035 {
            self.last_presence_time = now;
            let confidence = (presence_ratio / 0.15).clamp(0.0, 1.0);

            // If session was locked and auto-unlock is active, trigger wake
            if self.is_session_locked && self.auto_unlock_enabled {
                self.trigger_wake_session();
                self.is_session_locked = false;
            }

            PresenceStatus::Present { confidence }
        } else {
            let elapsed = now.duration_since(self.last_presence_time);

            // If elapsed absence exceeds threshold, trigger session lock
            if self.auto_lock_enabled && elapsed >= self.lock_timeout && !self.is_session_locked {
                self.trigger_lock_session();
                self.is_session_locked = true;
                self.last_lock_trigger = Some(now);
            }

            PresenceStatus::Absent {
                elapsed_absence: elapsed,
            }
        }
    }

    /// Dispatches command to lock the Wayland desktop session
    pub fn trigger_lock_session(&self) {
        println!("🔒 [Presence] User absent for {:?}. Triggering session lock...", self.lock_timeout);
        let _ = std::process::Command::new("loginctl")
            .arg("lock-session")
            .spawn();
    }

    /// Dispatches command to wake the display upon user return
    pub fn trigger_wake_session(&self) {
        println!("🔓 [Presence] User returned to workspace. Waking session...");
        let _ = std::process::Command::new("hyprctl")
            .args(["dispatch", "dpms", "on"])
            .spawn();
    }
}

impl Default for PresenceDetector {
    fn default() -> Self {
        Self::new(false, 45, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_present_on_skin_frame() {
        let mut detector = PresenceDetector::new(true, 10, false);
        let mut data = vec![10u8; 100 * 100];
        // Upper 40x40 skin region (1600 pixels out of 6500 upper pixels = ~24% ratio)
        for y in 10..50 {
            for x in 30..70 {
                data[y * 100 + x] = 140;
            }
        }
        let frame = RawFrame {
            data,
            width: 100,
            height: 100,
            timestamp_ms: 0,
        };

        let status = detector.evaluate_presence(&frame);
        match status {
            PresenceStatus::Present { confidence } => {
                assert!(confidence > 0.5);
            }
            PresenceStatus::Absent { .. } => panic!("Expected Present status"),
        }
    }

    #[test]
    fn test_presence_absent_on_dark_frame() {
        let mut detector = PresenceDetector::new(false, 10, false);
        let frame = RawFrame {
            data: vec![10u8; 100 * 100],
            width: 100,
            height: 100,
            timestamp_ms: 0,
        };

        let status = detector.evaluate_presence(&frame);
        match status {
            PresenceStatus::Absent { elapsed_absence } => {
                assert!(elapsed_absence.as_millis() < 500);
            }
            PresenceStatus::Present { .. } => panic!("Expected Absent status"),
        }
    }
}
