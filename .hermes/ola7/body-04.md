# [Ola 7.04] feat-swal-64 — System Tray & Notification Surface in Pure Rust

> Ola 7 — [Render/Notifications/FluentAcrylic].
> Labels: `ola7`, `wave-7`

---

## Current State (MEDIBLE)
- Feature: `feat-swal-64` at 0% in `.gitcore/features.json`
- Render pipeline modules in `crates/swal-render-pipeline/src/` (fluent_acrylic, mica_shader, orb_surface, controls_render).
- Existing tests in `crates/swal-render-pipeline`: 77 passing tests.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-render-pipeline/src/notification_surface.rs` providing GPU-accelerated toast notifications, system tray icon render bounds, and interactive agent alerts with acrylic backdrop effects.
- **File Target**: `crates/swal-render-pipeline/src/notification_surface.rs`

## Web Research Required
1. search: "Wayland notification layer surface acrylic blur WGPU Rust"
2. search: "desktop toast notification queue layout animation timer Rust"
3. search: "system tray status notifier item rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all tests pass
- [ ] `grep -rn "NotificationSurface" crates/swal-render-pipeline/src/notification_surface.rs` >= 1 match
- [ ] `grep -rn "NotificationToast" crates/swal-render-pipeline/src/notification_surface.rs` >= 1 match
- [ ] `grep -rn "NotificationUrgency" crates/swal-render-pipeline/src/notification_surface.rs` >= 1 match

## Exact Code Blueprint & Signatures

```rust
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationToast {
    pub id: u64,
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub urgency: NotificationUrgency,
    pub timeout_ms: u32,
    pub progress: Option<f32>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct NotificationQueue {
    active_notifications: VecDeque<NotificationToast>,
    history: Vec<NotificationToast>,
    next_id: u64,
    max_visible: usize,
}

impl NotificationQueue {
    pub fn new(max_visible: usize) -> Self {
        Self {
            active_notifications: VecDeque::new(),
            history: Vec::new(),
            next_id: 1,
            max_visible,
        }
    }

    pub fn post(&mut self, app_name: &str, title: &str, body: &str, urgency: NotificationUrgency) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let toast = NotificationToast {
            id,
            app_name: app_name.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            icon: None,
            urgency,
            timeout_ms: match urgency {
                NotificationUrgency::Low => 3000,
                NotificationUrgency::Normal => 5000,
                NotificationUrgency::Critical => 10000,
            },
            progress: None,
            timestamp_ms: 0,
        };

        if self.active_notifications.len() >= self.max_visible {
            if let Some(old) = self.active_notifications.pop_front() {
                self.history.push(old);
            }
        }

        self.active_notifications.push_back(toast);
        id
    }

    pub fn dismiss(&mut self, id: u64) -> bool {
        if let Some(pos) = self.active_notifications.iter().position(|n| n.id == id) {
            if let Some(removed) = self.active_notifications.remove(pos) {
                self.history.push(removed);
                return true;
            }
        }
        false
    }

    pub fn active_count(&self) -> usize {
        self.active_notifications.len()
    }
}
```

## Unit Tests Requirements
1. `test_notification_toast_creation_and_urgency`
2. `test_notification_queue_post_and_dismiss`
3. `test_notification_queue_max_visible_eviction`
4. `test_notification_toast_serde`
5. `test_urgency_timeout_mapping`

## Anti-Hallucination Guard
- Do NOT edit other crates or shared files.
- Place all implementation strictly inside `crates/swal-render-pipeline/src/notification_surface.rs`.
