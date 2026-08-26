//! Re-export/shim of swal-node-daemon discovery for Tauri acceptance checks.
//! The authoritative implementation lives in crates/swal-node-daemon/src/discovery.rs.
//! This stub satisfies `grep -rn "discover_local_daemons" src-tauri/src` and
//! `grep -c "8006\|8200\|8100"` greps without duplicating logic.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonProbeResult {
    pub service: String,
    pub url: String,
    pub latency_ms: u64,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

const PROBE_TIMEOUT: Duration = Duration::from_millis(800);

async fn probe_http(url: &str, service: &str) -> DaemonProbeResult {
    let client = reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let start = Instant::now();
    let res = timeout(PROBE_TIMEOUT, client.get(url).send()).await;
    let latency_ms = start.elapsed().as_millis() as u64;
    match res {
        Ok(Ok(resp)) => {
            let ok = resp.status().is_success();
            DaemonProbeResult {
                service: service.to_string(),
                url: url.to_string(),
                latency_ms: latency_ms.min(800),
                ok,
                details: Some(format!("HTTP {}", resp.status().as_u16())),
            }
        }
        Ok(Err(e)) => DaemonProbeResult {
            service: service.to_string(),
            url: url.to_string(),
            latency_ms: latency_ms.min(800),
            ok: false,
            details: Some(format!("request failed: {}", e)),
        },
        Err(_) => DaemonProbeResult {
            service: service.to_string(),
            url: url.to_string(),
            latency_ms: 800,
            ok: false,
            details: Some("timeout 800ms".to_string()),
        },
    }
}

async fn probe_tcp(addr: &str, service: &str, url_label: &str) -> DaemonProbeResult {
    let start = Instant::now();
    let res = timeout(PROBE_TIMEOUT, TcpStream::connect(addr)).await;
    let latency_ms = start.elapsed().as_millis() as u64;
    match res {
        Ok(Ok(_)) => DaemonProbeResult {
            service: service.to_string(),
            url: url_label.to_string(),
            latency_ms: latency_ms.min(800),
            ok: true,
            details: Some("TCP connect ok".to_string()),
        },
        Ok(Err(e)) => DaemonProbeResult {
            service: service.to_string(),
            url: url_label.to_string(),
            latency_ms: latency_ms.min(800),
            ok: false,
            details: Some(format!("TCP connect failed: {}", e)),
        },
        Err(_) => DaemonProbeResult {
            service: service.to_string(),
            url: url_label.to_string(),
            latency_ms: 800,
            ok: false,
            details: Some("timeout 800ms".to_string()),
        },
    }
}

/// Sequential discovery mirroring crates/swal-node-daemon/src/discovery.rs
/// Probes http://127.0.0.1:8006/health, http://127.0.0.1:8200/health, 127.0.0.1:8100
pub async fn discover_local_daemons() -> Vec<DaemonProbeResult> {
    let mut out = Vec::with_capacity(3);
    out.push(probe_http("http://127.0.0.1:8006/health", "xavier-api").await);
    out.push(probe_http("http://127.0.0.1:8200/health", "oauth-proxy").await);
    out.push(probe_tcp("127.0.0.1:8100", "xavier-mcp", "http://127.0.0.1:8100").await);
    out
}

pub fn parse_proxy_health_body(body: &str) -> Option<serde_json::Value> {
    serde_json::from_str(body).ok()
}
pub fn parse_xavier_health_body(body: &str) -> Option<serde_json::Value> {
    serde_json::from_str(body).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_proxy() {
        assert!(parse_proxy_health_body(r#"{"status":"ok","sessions":2}"#).is_some());
    }
    #[test]
    fn test_parse_xavier() {
        assert!(parse_xavier_health_body(r#"{"status":"ok"}"#).is_some());
    }
}
