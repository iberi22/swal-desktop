//! swal-vision-rs
//! Zero-ML Computer Vision, Hand Gesture Mouse Pointer & Presence Detection Engine

pub mod gesture_ipc;
pub mod hand_tracker;
pub mod presence;
pub mod v4l2_capture;

pub use gesture_ipc::{GestureConfig, GestureEvent, GESTURE_SOCKET};
pub use hand_tracker::{GestureKind, HandState, HandTracker};
pub use presence::{PresenceDetector, PresenceStatus};
pub use v4l2_capture::{FrameCapture, RawFrame};
