use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Health status of Xavier Memory Core services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XavierHealthStatus {
    pub http_ok: bool,
    pub mcp_ok: bool,
    pub http_status_code: Option<u16>,
    pub details: String,
}

impl XavierHealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.http_ok && self.mcp_ok
    }
}

/// HTTP client & MCP socket monitor for Xavier Cognitive Memory Core.
#[derive(Clone)]
pub struct XavierClient {
    pub api_url: String,
    pub mcp_port: u16,
    client: reqwest::Client,
}

impl XavierClient {
    /// Creates a new `XavierClient` with custom API URL and MCP port.
    pub fn new(api_url: &str, mcp_port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_default();

        Self {
            api_url: api_url.trim_end_matches('/').to_string(),
            mcp_port,
            client,
        }
    }

    /// Creates a `XavierClient` with default configuration (`http://127.0.0.1:8006`, MCP port `8100`).
    pub fn default_config() -> Self {
        Self::new("http://127.0.0.1:8006", 8100)
    }

    /// Checks HTTP `/health` endpoint asynchronously.
    pub async fn check_http_health(&self) -> (bool, Option<u16>, String) {
        let health_url = format!("{}/health", self.api_url);
        match self.client.get(&health_url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    (true, Some(status.as_u16()), "HTTP health check passed".to_string())
                } else {
                    (
                        false,
                        Some(status.as_u16()),
                        format!("HTTP health returned status {}", status),
                    )
                }
            }
            Err(err) => (
                false,
                None,
                format!("HTTP health request failed: {}", err),
            ),
        }
    }

    /// Checks MCP TCP socket listener asynchronously on `mcp_port`.
    pub async fn check_mcp_health(&self) -> (bool, String) {
        let addr = format!("127.0.0.1:{}", self.mcp_port);
        match timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await {
            Ok(Ok(_stream)) => (true, "MCP socket connection succeeded".to_string()),
            Ok(Err(err)) => (false, format!("MCP socket connection failed: {}", err)),
            Err(_) => (false, "MCP socket connection timed out".to_string()),
        }
    }

    /// Performs full health inspection across HTTP endpoint and MCP TCP port.
    pub async fn check_health(&self) -> XavierHealthStatus {
        let (http_ok, http_status_code, http_msg) = self.check_http_health().await;
        let (mcp_ok, mcp_msg) = self.check_mcp_health().await;

        let details = format!("HTTP: {}; MCP: {}", http_msg, mcp_msg);

        XavierHealthStatus {
            http_ok,
            mcp_ok,
            http_status_code,
            details,
        }
    }

    /// Performs health check with exponential retry backoff on failure.
    pub async fn check_health_with_retry(
        &self,
        max_retries: u32,
        base_backoff_ms: u64,
    ) -> XavierHealthStatus {
        let mut attempt = 0;
        loop {
            let status = self.check_health().await;
            if status.is_healthy() || attempt >= max_retries {
                return status;
            }

            attempt += 1;
            let backoff = Duration::from_millis(base_backoff_ms * (1 << (attempt - 1)));
            tokio::time::sleep(backoff).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xavier_health_status_is_healthy() {
        let healthy = XavierHealthStatus {
            http_ok: true,
            mcp_ok: true,
            http_status_code: Some(200),
            details: "all good".to_string(),
        };
        assert!(healthy.is_healthy());

        let unhealthy_http = XavierHealthStatus {
            http_ok: false,
            mcp_ok: true,
            http_status_code: Some(500),
            details: "http failed".to_string(),
        };
        assert!(!unhealthy_http.is_healthy());

        let unhealthy_mcp = XavierHealthStatus {
            http_ok: true,
            mcp_ok: false,
            http_status_code: Some(200),
            details: "mcp failed".to_string(),
        };
        assert!(!unhealthy_mcp.is_healthy());
    }

    #[test]
    fn test_xavier_client_initialization() {
        let client = XavierClient::default_config();
        assert_eq!(client.api_url, "http://127.0.0.1:8006");
        assert_eq!(client.mcp_port, 8100);

        let custom = XavierClient::new("http://localhost:9000/", 9100);
        assert_eq!(custom.api_url, "http://localhost:9000");
        assert_eq!(custom.mcp_port, 9100);
    }

    #[tokio::test]
    async fn test_check_mcp_health_unbound_port() {
        let client = XavierClient::new("http://127.0.0.1:8006", 59999);
        let (ok, msg) = client.check_mcp_health().await;
        assert!(!ok);
        assert!(msg.contains("failed") || msg.contains("timed out"));
    }
}
