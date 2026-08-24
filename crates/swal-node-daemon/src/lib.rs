pub mod agent_config_ipc;
pub mod desktop_bridge;
pub mod doctor_engine;
pub mod hermes_stream;
pub mod native_shell;
pub mod settings_cli;
pub mod settings_store;
pub mod settings_window;
pub mod xavier;

pub use hermes_stream::{HermesEvent, HermesEventListener, HermesStreamEvent};

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::interval;
use xavier::{XavierClient, XavierHealthStatus};

/// Edge-Mesh node discovery status packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EdgeMeshNodeStatus {
    pub node_id: String,
    pub xavier_status: XavierHealthStatus,
    pub timestamp_secs: u64,
}

/// Configuration options for SWAL Node Daemon Tokio Coordinator.
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub xavier_api_url: String,
    pub xavier_mcp_port: u16,
    pub poll_interval_secs: u64,
    pub mesh_control_port: u16,
    pub mesh_broadcast_port: u16,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            xavier_api_url: "http://127.0.0.1:8006".to_string(),
            xavier_mcp_port: 8100,
            poll_interval_secs: 5,
            mesh_control_port: 8900,
            mesh_broadcast_port: 8901,
        }
    }
}

/// Daemon coordinator managing async loops and Edge-Mesh discovery socket.
pub struct SwalNodeDaemon {
    pub config: DaemonConfig,
    pub xavier_client: XavierClient,
    is_running: Arc<AtomicBool>,
}

impl SwalNodeDaemon {
    pub fn new(config: DaemonConfig) -> Self {
        let xavier_client = XavierClient::new(&config.xavier_api_url, config.xavier_mcp_port);
        Self {
            config,
            xavier_client,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Broadcasts node health status over local Edge-Mesh control UDP socket.
    pub async fn broadcast_node_status(
        &self,
        status: &XavierHealthStatus,
        socket: &Option<UdpSocket>,
    ) -> Result<usize, String> {
        let node_status = EdgeMeshNodeStatus {
            node_id: "swal-node-local".to_string(),
            xavier_status: status.clone(),
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let json_bytes = serde_json::to_vec(&node_status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?;

        if let Some(sock) = socket {
            let target = format!("255.255.255.255:{}", self.config.mesh_broadcast_port);
            match sock.send_to(&json_bytes, &target).await {
                Ok(bytes_sent) => Ok(bytes_sent),
                Err(err) => Err(format!("UDP socket send_to error: {}", err)),
            }
        } else {
            Ok(json_bytes.len())
        }
    }

    /// Primary async Tokio coordinator loop running every `poll_interval_secs` (5s default).
    pub async fn run_supervisor_loop(&self, max_ticks: Option<u64>) {
        self.is_running.store(true, Ordering::SeqCst);

        let mesh_socket = match UdpSocket::bind(format!("0.0.0.0:{}", self.config.mesh_control_port)).await {
            Ok(sock) => {
                let _ = sock.set_broadcast(true);
                Some(sock)
            }
            Err(_) => None,
        };

        let mut timer = interval(Duration::from_secs(self.config.poll_interval_secs));
        let mut tick_count = 0u64;

        while self.is_running.load(Ordering::SeqCst) {
            timer.tick().await;

            // Poll Xavier Health (HTTP :8006 and MCP :8100)
            let health = self.xavier_client.check_health_with_retry(3, 100).await;

            // Edge-Mesh P2P Discovery broadcast
            let _ = self.broadcast_node_status(&health, &mesh_socket).await;

            tick_count += 1;
            if let Some(max) = max_ticks {
                if tick_count >= max {
                    break;
                }
            }
        }

        self.is_running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_default_config() {
        let config = DaemonConfig::default();
        assert_eq!(config.xavier_api_url, "http://127.0.0.1:8006");
        assert_eq!(config.xavier_mcp_port, 8100);
        assert_eq!(config.poll_interval_secs, 5);
        assert_eq!(config.mesh_control_port, 8900);
        assert_eq!(config.mesh_broadcast_port, 8901);
    }

    #[test]
    fn test_daemon_initial_state() {
        let daemon = SwalNodeDaemon::new(DaemonConfig::default());
        assert!(!daemon.is_running());
        daemon.stop();
        assert!(!daemon.is_running());
    }

    #[tokio::test]
    async fn test_broadcast_node_status_formatting() {
        let daemon = SwalNodeDaemon::new(DaemonConfig::default());
        let health = XavierHealthStatus {
            http_ok: true,
            mcp_ok: true,
            http_status_code: Some(200),
            details: "Healthy".to_string(),
        };

        let res = daemon.broadcast_node_status(&health, &None).await;
        assert!(res.is_ok());
        assert!(res.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_supervisor_loop_max_ticks() {
        let mut config = DaemonConfig::default();
        config.poll_interval_secs = 1;
        let daemon = SwalNodeDaemon::new(config);

        // Run for 1 tick max
        daemon.run_supervisor_loop(Some(1)).await;
        assert!(!daemon.is_running());
    }
}
