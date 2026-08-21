//! Unix Domain Socket IPC for SWAL Telemetry streaming

use crate::{read_system_metrics, CpuTicks, SystemMetrics};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tokio::sync::broadcast;
use tokio::time::sleep;

pub const DEFAULT_SOCKET_PATH: &str = "/run/user/1000/swal/telemetry.sock";

pub struct TelemetryServer {
    socket_path: PathBuf,
    tx: broadcast::Sender<SystemMetrics>,
}

impl TelemetryServer {
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            tx,
        }
    }

    pub fn default_server() -> Self {
        Self::new(DEFAULT_SOCKET_PATH)
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemMetrics> {
        self.tx.subscribe()
    }

    /// Runs the non-blocking Unix Socket listener and broadcast loop.
    pub async fn run(&self, sampling_interval: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(parent) = self.socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        let tx = self.tx.clone();

        // Spawn background sampling task
        tokio::spawn(async move {
            let mut prev_cpu: Option<CpuTicks> = None;
            loop {
                let (metrics, curr_cpu) = read_system_metrics(prev_cpu);
                prev_cpu = Some(curr_cpu);
                let _ = tx.send(metrics);
                sleep(sampling_interval).await;
            }
        });

        // Accept loop for incoming client connections
        loop {
            match listener.accept().await {
                Ok((mut stream, _addr)) => {
                    let mut rx = self.tx.subscribe();
                    tokio::spawn(async move {
                        while let Ok(metrics) = rx.recv().await {
                            if let Ok(json) = serde_json::to_string(&metrics) {
                                let payload = format!("{}\n", json);
                                if stream.write_all(payload.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(_e) => {
                    sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncBufReadExt;
    use tokio::io::BufReader as AsyncBufReader;
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn test_telemetry_ipc_server() {
        let sock_path = format!("/tmp/test_swal_telemetry_{}.sock", std::process::id());
        let server = TelemetryServer::new(&sock_path);

        let server_path = server.socket_path().to_path_buf();

        // Run server in background task
        let server_handle = tokio::spawn(async move {
            let _ = server.run(Duration::from_millis(50)).await;
        });

        // Give server time to bind
        sleep(Duration::from_millis(100)).await;

        assert!(server_path.exists());

        // Connect client
        let mut client_stream = UnixStream::connect(&server_path)
            .await
            .expect("Failed to connect to IPC socket");

        let mut reader = AsyncBufReader::new(&mut client_stream);
        let mut line = String::new();

        let read_res = tokio::time::timeout(Duration::from_secs(2), reader.read_line(&mut line)).await;
        assert!(read_res.is_ok(), "Timed out reading from Unix socket");

        let metrics: SystemMetrics = serde_json::from_str(&line).expect("Failed to deserialize SystemMetrics");
        assert!(metrics.ram_total_mb > 0);

        // Cleanup
        server_handle.abort();
        let _ = std::fs::remove_file(&server_path);
    }
}
