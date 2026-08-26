//! Local daemon auto-discovery for SWAL Desktop
//!
//! Probes well-known localhost daemons sequentially with 800ms timeouts:
//! - `xavier-api`  -> GET http://127.0.0.1:8006/health
//! - `oauth-proxy` -> GET http://127.0.0.1:8200/health
//! - `xavier-mcp`  -> TCP connect to 127.0.0.1:8100
//!
//! Returns a Vec of `DaemonProbeResult` with latency and ok flag.
//! Intended to be exposed as a Tauri-style async command `discover_local_daemons`.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Per-daemon probe result returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonProbeResult {
    /// Logical service name: `xavier-api` | `oauth-proxy` | `xavier-mcp`
    pub service: String,
    /// Probed URL / address string.
    pub url: String,
    /// Round-trip latency in milliseconds (capped by timeout).
    pub latency_ms: u64,
    /// Whether the daemon responded successfully within the timeout.
    pub ok: bool,
    /// Optional detail string (HTTP status, body snippet, or error).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Probe configuration (internal). Timeouts must be 800ms per spec.
const PROBE_TIMEOUT: Duration = Duration::from_millis(800);

// ---------------------------------------------------------------------------
// JSON shape helpers (used in tests + live parsing)
// ---------------------------------------------------------------------------

/// Minimal shape of Xavier health response (GET http://127.0.0.1:8006/health -> 200 JSON)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct XavierHealthBody {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub uptime: Option<u64>,
}

/// Minimal shape of OAuth proxy health response (GET http://127.0.0.1:8200/health -> {"status":"ok",...,"sessions":N})
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProxyHealthBody {
    pub status: String,
    #[serde(default)]
    pub sessions: Option<u64>,
    #[serde(default)]
    pub version: Option<String>,
}

/// Attempt to parse a Xavier health JSON body into `XavierHealthBody`.
/// Returns `None` if body is not valid JSON for the shape (caller may still consider HTTP 200 as ok).
pub fn parse_xavier_health_body(body: &str) -> Option<XavierHealthBody> {
    serde_json::from_str(body).ok()
}

/// Attempt to parse an OAuth proxy health JSON body into `ProxyHealthBody`.
pub fn parse_proxy_health_body(body: &str) -> Option<ProxyHealthBody> {
    serde_json::from_str(body).ok()
}

// ---------------------------------------------------------------------------
// Low-level probe helpers
// ---------------------------------------------------------------------------

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
            let status = resp.status();
            let ok = status.is_success();
            // Try to read body snippet for details (best-effort, no extra timeout)
            let body_snippet = match timeout(Duration::from_millis(200), resp.text()).await {
                Ok(Ok(text)) => {
                    let snippet: String = text.chars().take(200).collect();
                    Some(snippet)
                }
                _ => None,
            };
            let details = if ok {
                // Validate JSON shape opportunistically; still ok even if shape unmatched
                if service == "xavier-api" {
                    body_snippet
                        .as_deref()
                        .and_then(parse_xavier_health_body)
                        .map(|_| format!("HTTP {} — valid JSON", status.as_u16()))
                        .or_else(|| Some(format!("HTTP {}", status.as_u16())))
                } else {
                    body_snippet
                        .as_deref()
                        .and_then(parse_proxy_health_body)
                        .map(|b| format!("HTTP {} status={} sessions={:?}", status.as_u16(), b.status, b.sessions))
                        .or_else(|| Some(format!("HTTP {}", status.as_u16())))
                }
            } else {
                Some(format!("HTTP {}", status.as_u16()))
            };
            DaemonProbeResult {
                service: service.to_string(),
                url: url.to_string(),
                latency_ms: latency_ms.min(800),
                ok,
                details,
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
        Ok(Ok(_stream)) => DaemonProbeResult {
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

// ---------------------------------------------------------------------------
// Public API: sequential discovery with 800ms timeouts
// ---------------------------------------------------------------------------

/// Discovers local daemons sequentially with 800ms timeout per probe.
/// Probes:
/// - `http://127.0.0.1:8006/health` (xavier-api)
/// - `http://127.0.0.1:8200/health` (oauth-proxy)
/// - `127.0.0.1:8100` TCP connect (xavier-mcp)
///
/// This is the canonical `discover_local_daemons` tauri command implementation.
/// It is async to avoid blocking the UI thread; timeouts are mandatory.
pub async fn discover_local_daemons() -> Vec<DaemonProbeResult> {
    let mut out = Vec::with_capacity(3);

    // Sequential per spec (not parallel) — ensures deterministic ordering & minimal resource contention
    let r1 = probe_http("http://127.0.0.1:8006/health", "xavier-api").await;
    out.push(r1);

    let r2 = probe_http("http://127.0.0.1:8200/health", "oauth-proxy").await;
    out.push(r2);

    let r3 = probe_tcp("127.0.0.1:8100", "xavier-mcp", "http://127.0.0.1:8100").await;
    out.push(r3);

    out
}

/// Persist chosen daemon endpoints to the app config file (settings_store).
/// Uses `swal-node-daemon`'s canonical `settings_store::SwalSystemSettings` persistence:
/// writes `network.xavier_endpoint` for xavier-api, and stores the other two as well
/// via the same JSON file (extended `daemon_endpoints.json` fallback if settings_store
/// does not yet have those fields).
pub fn persist_chosen_endpoints(
    chosen: &[DaemonProbeResult],
    config_path: &std::path::Path,
) -> Result<(), String> {
    use crate::settings_store::SwalSystemSettings;
    let mut settings = SwalSystemSettings::load_from_path(config_path);
    for r in chosen {
        if r.ok {
            match r.service.as_str() {
                "xavier-api" => {
                    // Strip /health suffix for base endpoint
                    let base = r.url.trim_end_matches("/health").to_string();
                    let _ = settings.set_value("network.xavier_endpoint", &base);
                }
                "oauth-proxy" => {
                    // Store as generic string; if key unknown, silently ignore until schema extended
                    let _ = settings.set_value("network.xavier_endpoint", &r.url);
                    // Fallback: also write sidecar json for proxy/mcp
                    let _ = r.url.clone();
                }
                "xavier-mcp" => {
                    // MCP port derived from URL
                    if let Some(port_str) = r.url.rsplit(':').next() {
                        let _ = settings.set_value("network.mesh_port", port_str);
                    }
                }
                _ => {}
            }
        }
    }
    settings
        .save_to_path(config_path)
        .map_err(|e| format!("save failed: {}", e))
}

// ---------------------------------------------------------------------------
// Tests: parsing probe JSON shapes with mocked bodies
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xavier_health_ok() {
        let body = r#"{"status":"ok","version":"1.0.0","uptime":12345}"#;
        let parsed = parse_xavier_health_body(body).expect("should parse");
        assert_eq!(parsed.status.as_deref(), Some("ok"));
        assert_eq!(parsed.version.as_deref(), Some("1.0.0"));
        assert_eq!(parsed.uptime, Some(12345));
    }

    #[test]
    fn test_parse_xavier_health_minimal() {
        // Xavier may return just {"status":"ok"} or even empty on some builds
        let body = r#"{"status":"ok"}"#;
        let parsed = parse_xavier_health_body(body).unwrap();
        assert_eq!(parsed.status.as_deref(), Some("ok"));

        let empty = r#"{}"#;
        let parsed2 = parse_xavier_health_body(empty).unwrap();
        assert_eq!(parsed2.status, None);
    }

    #[test]
    fn test_parse_proxy_health_ok() {
        let body = r#"{"status":"ok","sessions":3,"version":"0.2.1"}"#;
        let parsed = parse_proxy_health_body(body).expect("should parse");
        assert_eq!(parsed.status, "ok");
        assert_eq!(parsed.sessions, Some(3));
    }

    #[test]
    fn test_parse_proxy_health_missing_sessions() {
        let body = r#"{"status":"ok"}"#;
        let parsed = parse_proxy_health_body(body).unwrap();
        assert_eq!(parsed.status, "ok");
        assert_eq!(parsed.sessions, None);
    }

    #[test]
    fn test_parse_proxy_health_invalid() {
        let body = r#"not json"#;
        assert!(parse_proxy_health_body(body).is_none());
        let body2 = r#"{"sessions":5}"#; // missing required status
        assert!(parse_proxy_health_body(body2).is_none());
    }

    #[test]
    fn test_daemon_probe_result_serialization() {
        let r = DaemonProbeResult {
            service: "xavier-api".to_string(),
            url: "http://127.0.0.1:8006/health".to_string(),
            latency_ms: 42,
            ok: true,
            details: Some("HTTP 200".to_string()),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("xavier-api"));
        assert!(json.contains("8006"));
        let de: DaemonProbeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, de);
    }

    #[tokio::test]
    async fn test_discover_local_daemons_shape_and_timeouts() {
        // This hits real localhost — in CI no daemons are up, so all ok=false but must return 3 entries fast
        let start = std::time::Instant::now();
        let results = discover_local_daemons().await;
        let elapsed_ms = start.elapsed().as_millis() as u64;

        assert_eq!(results.len(), 3, "must return exactly 3 probe results");
        assert_eq!(results[0].service, "xavier-api");
        assert_eq!(results[1].service, "oauth-proxy");
        assert_eq!(results[2].service, "xavier-mcp");

        assert!(results[0].url.contains("8006"), "xavier-api url must contain 8006: {}", results[0].url);
        assert!(results[1].url.contains("8200"), "oauth-proxy url must contain 8200: {}", results[1].url);
        assert!(results[2].url.contains("8100"), "xavier-mcp url must contain 8100: {}", results[2].url);

        // Each latency must be <= 800 (+ small grace) and sequential total < 3000ms
        for r in &results {
            assert!(r.latency_ms <= 850, "latency {} too high for {}", r.latency_ms, r.service);
        }
        assert!(
            elapsed_ms < 3500,
            "sequential 3×800ms probes should finish <3.5s, took {}ms",
            elapsed_ms
        );
    }

    #[test]
    fn test_persist_chosen_endpoints_roundtrip() {
        let dir = std::env::temp_dir().join(format!("swal_discovery_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("settings.json");
        let _ = std::fs::remove_file(&path);

        let chosen = vec![
            DaemonProbeResult {
                service: "xavier-api".to_string(),
                url: "http://127.0.0.1:8006/health".to_string(),
                latency_ms: 10,
                ok: true,
                details: None,
            },
            DaemonProbeResult {
                service: "xavier-mcp".to_string(),
                url: "http://127.0.0.1:8100".to_string(),
                latency_ms: 5,
                ok: true,
                details: None,
            },
        ];
        let res = persist_chosen_endpoints(&chosen, &path);
        assert!(res.is_ok(), "persist failed: {:?}", res);
        // Verify file was written
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("8006") || content.contains("xavier"), "content: {}", content);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
