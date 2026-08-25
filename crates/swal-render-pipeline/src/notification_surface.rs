//! notification_surface.rs
//! System Tray & Toast Notification Surface in Pure Rust with Fluent Acrylic Effects

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

    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    pub fn get_active(&self) -> &VecDeque<NotificationToast> {
        &self.active_notifications
    }
}

pub struct NotificationSurface {
    pub queue: NotificationQueue,
    pub width: u32,
    pub height: u32,
}

impl NotificationSurface {
    pub fn new(max_visible: usize) -> Self {
        Self {
            queue: NotificationQueue::new(max_visible),
            width: 360,
            height: 600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_toast_creation_and_urgency() {
        let mut queue = NotificationQueue::new(3);
        let id = queue.post("Hermes", "Task Done", "Wave 7 deployed", NotificationUrgency::Normal);

        assert_eq!(id, 1);
        assert_eq!(queue.active_count(), 1);
        let toast = &queue.get_active()[0];
        assert_eq!(toast.app_name, "Hermes");
        assert_eq!(toast.timeout_ms, 5000);
    }

    #[test]
    fn test_notification_queue_post_and_dismiss() {
        let mut queue = NotificationQueue::new(5);
        let id1 = queue.post("Agent", "Notice 1", "Body 1", NotificationUrgency::Low);
        let id2 = queue.post("Agent", "Notice 2", "Body 2", NotificationUrgency::Critical);

        assert_eq!(queue.active_count(), 2);
        assert!(queue.dismiss(id1));
        assert_eq!(queue.active_count(), 1);
        assert_eq!(queue.history_count(), 1);
        assert!(!queue.dismiss(999));
        assert_eq!(queue.get_active()[0].id, id2);
    }

    #[test]
    fn test_notification_queue_max_visible_eviction() {
        let mut queue = NotificationQueue::new(2);
        queue.post("App", "1", "1", NotificationUrgency::Normal);
        queue.post("App", "2", "2", NotificationUrgency::Normal);
        queue.post("App", "3", "3", NotificationUrgency::Normal);

        assert_eq!(queue.active_count(), 2);
        assert_eq!(queue.history_count(), 1);
        assert_eq!(queue.get_active()[0].title, "2");
        assert_eq!(queue.get_active()[1].title, "3");
    }

    #[test]
    fn test_notification_toast_serde() {
        let toast = NotificationToast {
            id: 42,
            app_name: "Xavier".to_string(),
            title: "RAG Indexed".to_string(),
            body: "15 documents embedded".to_string(),
            icon: Some("book".to_string()),
            urgency: NotificationUrgency::Normal,
            timeout_ms: 5000,
            progress: Some(1.0),
            timestamp_ms: 1000,
        };

        let json = serde_json::to_string(&toast).expect("serialize");
        let deserialized: NotificationToast = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.app_name, "Xavier");
    }

    #[test]
    fn test_urgency_timeout_mapping() {
        let mut queue = NotificationQueue::new(5);
        let id_low = queue.post("A", "Low", "", NotificationUrgency::Low);
        let id_crit = queue.post("A", "Crit", "", NotificationUrgency::Critical);

        let active = queue.get_active();
        let low = active.iter().find(|t| t.id == id_low).unwrap();
        let crit = active.iter().find(|t| t.id == id_crit).unwrap();

        assert_eq!(low.timeout_ms, 3000);
        assert_eq!(crit.timeout_ms, 10000);
    }
}
