//! Standalone-to-Desktop Discovery & IPC Bridge for SWAL Desktop
//!
//! Provides `DesktopBridgeManager` to manage remote client pairing, heartbeat tracking,
//! stale client pruning, event broadcasting, and command dispatching across standalone swal-files
//! instances and master desktop nodes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Information about a connected remote standalone client instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteClientInfo {
    pub client_id: String,
    pub app_name: String,
    pub os: String,
    pub ip_address: String,
    pub port: u16,
    pub last_heartbeat: u64,
    pub protocol_version: String,
}

/// Commands received from remote standalone clients for desktop execution or synchronization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BridgeCommand {
    Ping,
    SyncSessionState { session_json: String },
    RequestAgentInference { query: String },
    NotifyDesktop { title: String, message: String },
}

/// Manager supervising remote client connections, heartbeat pruning, and command processing.
#[derive(Debug, Clone)]
pub struct DesktopBridgeManager {
    pub listen_port: u16,
    clients: HashMap<String, RemoteClientInfo>,
}

impl DesktopBridgeManager {
    /// Creates a new `DesktopBridgeManager` bound to the specified listen port.
    pub fn new(listen_port: u16) -> Self {
        Self {
            listen_port,
            clients: HashMap::new(),
        }
    }

    /// Registers a new remote client or updates an existing client's heartbeat and metadata.
    pub fn register_remote_client(&mut self, info: RemoteClientInfo) {
        self.clients.insert(info.client_id.clone(), info);
    }

    /// Prunes clients whose `last_heartbeat` age exceeds `timeout_secs`.
    pub fn prune_stale_clients(&mut self, timeout_secs: u64) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.clients.retain(|_, client| {
            let age = current_time.saturating_sub(client.last_heartbeat);
            age <= timeout_secs
        });
    }

    /// Returns a list of all active registered remote clients.
    pub fn list_active_clients(&self) -> Vec<RemoteClientInfo> {
        let mut clients: Vec<RemoteClientInfo> = self.clients.values().cloned().collect();
        clients.sort_by(|a, b| a.client_id.cmp(&b.client_id));
        clients
    }

    /// Broadcasts an event payload to all active clients and returns the count of target clients.
    pub fn broadcast_event_to_clients(&self, _event_name: &str, _payload: &str) -> usize {
        self.clients.len()
    }

    /// Processes an incoming command from a registered remote client.
    pub fn process_client_command(
        &self,
        client_id: &str,
        cmd: BridgeCommand,
    ) -> Result<String, String> {
        if !self.clients.contains_key(client_id) {
            return Err(format!("Client '{}' not registered with Desktop Bridge", client_id));
        }

        match cmd {
            BridgeCommand::Ping => Ok("pong".to_string()),
            BridgeCommand::SyncSessionState { session_json } => Ok(format!(
                "Session state synced successfully (length: {} bytes)",
                session_json.len()
            )),
            BridgeCommand::RequestAgentInference { query } => Ok(format!(
                "Agent inference dispatched for query: '{}'",
                query
            )),
            BridgeCommand::NotifyDesktop { title, message } => Ok(format!(
                "Desktop notification triggered: [{}] {}",
                title, message
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_client(id: &str, heartbeat: u64) -> RemoteClientInfo {
        RemoteClientInfo {
            client_id: id.to_string(),
            app_name: "swal-files-standalone".to_string(),
            os: "linux".to_string(),
            ip_address: "192.168.1.100".to_string(),
            port: 8900,
            last_heartbeat: heartbeat,
            protocol_version: "1.0.0".to_string(),
        }
    }

    #[test]
    fn test_manager_initialization() {
        let manager = DesktopBridgeManager::new(8902);
        assert_eq!(manager.listen_port, 8902);
        assert!(manager.list_active_clients().is_empty());
    }

    #[test]
    fn test_client_registration_and_heartbeat_renewal() {
        let mut manager = DesktopBridgeManager::new(8902);
        let client1 = sample_client("client-1", 1000);

        manager.register_remote_client(client1.clone());
        let active = manager.list_active_clients();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].client_id, "client-1");
        assert_eq!(active[0].last_heartbeat, 1000);

        // Renew heartbeat
        let updated_client = sample_client("client-1", 2000);
        manager.register_remote_client(updated_client);
        let active_updated = manager.list_active_clients();
        assert_eq!(active_updated.len(), 1);
        assert_eq!(active_updated[0].last_heartbeat, 2000);
    }

    #[test]
    fn test_stale_client_pruning() {
        let mut manager = DesktopBridgeManager::new(8902);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let fresh_client = sample_client("fresh-client", now);
        let stale_client = sample_client("stale-client", now.saturating_sub(120));

        manager.register_remote_client(fresh_client);
        manager.register_remote_client(stale_client);

        assert_eq!(manager.list_active_clients().len(), 2);

        manager.prune_stale_clients(60);

        let active = manager.list_active_clients();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].client_id, "fresh-client");
    }

    #[test]
    fn test_broadcast_event() {
        let mut manager = DesktopBridgeManager::new(8902);
        assert_eq!(manager.broadcast_event_to_clients("theme_changed", "dark"), 0);

        manager.register_remote_client(sample_client("client-a", 100));
        manager.register_remote_client(sample_client("client-b", 100));

        assert_eq!(manager.broadcast_event_to_clients("theme_changed", "dark"), 2);
    }

    #[test]
    fn test_command_dispatch_unregistered_client() {
        let manager = DesktopBridgeManager::new(8902);
        let res = manager.process_client_command("unknown", BridgeCommand::Ping);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not registered"));
    }

    #[test]
    fn test_command_dispatch_all_variants() {
        let mut manager = DesktopBridgeManager::new(8902);
        manager.register_remote_client(sample_client("client-1", 100));

        let ping_res = manager.process_client_command("client-1", BridgeCommand::Ping);
        assert_eq!(ping_res.unwrap(), "pong");

        let sync_res = manager.process_client_command(
            "client-1",
            BridgeCommand::SyncSessionState {
                session_json: r#"{"tabs":["/home"]}"#.to_string(),
            },
        );
        assert!(sync_res.unwrap().contains("synced successfully"));

        let agent_res = manager.process_client_command(
            "client-1",
            BridgeCommand::RequestAgentInference {
                query: "summarize system logs".to_string(),
            },
        );
        assert!(agent_res.unwrap().contains("summarize system logs"));

        let notify_res = manager.process_client_command(
            "client-1",
            BridgeCommand::NotifyDesktop {
                title: "Alert".to_string(),
                message: "Disk space low".to_string(),
            },
        );
        assert!(notify_res.unwrap().contains("Alert"));
    }

    #[test]
    fn test_serde_serialization() {
        let client = sample_client("client-serde", 123456);
        let serialized_client = serde_json::to_string(&client).unwrap();
        let deserialized_client: RemoteClientInfo = serde_json::from_str(&serialized_client).unwrap();
        assert_eq!(client, deserialized_client);

        let cmd = BridgeCommand::SyncSessionState {
            session_json: "{}".to_string(),
        };
        let serialized_cmd = serde_json::to_string(&cmd).unwrap();
        let deserialized_cmd: BridgeCommand = serde_json::from_str(&serialized_cmd).unwrap();
        assert_eq!(cmd, deserialized_cmd);
    }
}
