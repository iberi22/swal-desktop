use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use swal_vision_rs::{
    gesture_ipc::{GestureConfig, GestureEvent, GESTURE_SOCKET},
    hand_tracker::HandTracker,
    presence::PresenceDetector,
    v4l2_capture::FrameCapture,
};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;

#[tokio::main]
async fn main() {
    let config = GestureConfig::load_or_default();
    println!(
        "🎥 [swal-vision] Started on camera '{}' (Gestures: {}, Presence Lock: {}/{}s)",
        config.camera_device,
        if config.enabled { "ON" } else { "OFF" },
        if config.presence_auto_lock { "ON" } else { "OFF" },
        config.auto_lock_timeout_secs
    );

    // Clean up old socket if present
    if Path::new(GESTURE_SOCKET).exists() {
        let _ = std::fs::remove_file(GESTURE_SOCKET);
    }

    let capture = FrameCapture::new(&config.camera_device, 320, 240);
    let mut tracker = HandTracker::new().with_smoothing(config.cursor_smoothing);
    let mut presence = PresenceDetector::new(
        config.presence_auto_lock,
        config.auto_lock_timeout_secs,
        config.auto_unlock_on_presence,
    );

    // Watch channel for broadcasting gesture events to IPC clients
    let (tx, rx) = tokio::sync::watch::channel(GestureEvent::none_event());

    // Task 1: Video Capture, Gesture Tracking & Presence Detection Loop
    let capture_task = tokio::spawn(async move {
        let mut tick_count: u64 = 0;
        loop {
            // Read hardware V4L2 frame or fallback to mock
            let frame = capture.capture_frame();

            // 1. Process Hand Gesture Tracking (~30 FPS)
            let state = tracker.process_frame(&frame);
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let event = GestureEvent::from_hand_state(&state, ts);
            let _ = tx.send(event);

            // 2. Process Presence / Walk-Away Detection every ~1.5s (45 frames) for 0% CPU footprint
            tick_count = tick_count.wrapping_add(1);
            if tick_count % 45 == 0 {
                let _ = presence.evaluate_presence(&frame);
            }

            tokio::time::sleep(Duration::from_millis(33)).await; // ~30fps
        }
    });

    // Task 2: High-speed Unix Domain Socket Server for Desktop Clients
    let ipc_task = tokio::spawn(async move {
        let listener = match UnixListener::bind(GESTURE_SOCKET) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("❌ Failed to bind gesture socket {}: {}", GESTURE_SOCKET, e);
                return;
            }
        };
        println!("✓ Gesture IPC socket active at {}", GESTURE_SOCKET);

        loop {
            if let Ok((mut client, _)) = listener.accept().await {
                let mut rx2 = rx.clone();
                tokio::spawn(async move {
                    loop {
                        if rx2.changed().await.is_err() {
                            break;
                        }
                        let event = rx2.borrow().clone();
                        if let Ok(mut json) = serde_json::to_string(&event) {
                            json.push('\n');
                            if client.write_all(json.as_bytes()).await.is_err() {
                                break;
                            }
                        }
                    }
                });
            }
        }
    });

    // Graceful shutdown signal handling
    tokio::signal::ctrl_c().await.ok();
    println!("\n🛑 [swal-vision] Shutting down cleanly...");
    capture_task.abort();
    ipc_task.abort();
    let _ = std::fs::remove_file(GESTURE_SOCKET);
}
