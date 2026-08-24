use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Status of an individual system diagnostic probe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    Pass,
    Warning,
    Fail,
}

/// Detailed result of a diagnostic check probe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub category: String,
    pub status: CheckStatus,
    pub message: String,
    pub auto_fixable: bool,
}

/// Aggregated system diagnostic and self-healing status report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwalDoctorReport {
    pub checks: Vec<DiagnosticCheck>,
    pub all_passed: bool,
    pub error_count: usize,
    pub warning_count: usize,
}

impl SwalDoctorReport {
    pub fn new(checks: Vec<DiagnosticCheck>) -> Self {
        let mut error_count = 0;
        let mut warning_count = 0;

        for check in &checks {
            match check.status {
                CheckStatus::Fail => error_count += 1,
                CheckStatus::Warning => warning_count += 1,
                CheckStatus::Pass => {}
            }
        }

        let all_passed = error_count == 0 && warning_count == 0;

        Self {
            checks,
            all_passed,
            error_count,
            warning_count,
        }
    }
}

/// SWAL Doctor Embedded Self-Healing & Diagnostic Engine.
#[derive(Debug, Clone)]
pub struct SwalDoctorEngine {
    pub xavier_url: String,
    pub xavier_port: u16,
    pub config_dir: PathBuf,
}

impl Default for SwalDoctorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SwalDoctorEngine {
    /// Creates a new `SwalDoctorEngine` with default system target paths.
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let config_dir = PathBuf::from(home).join(".config").join("swal");
        Self {
            xavier_url: "http://127.0.0.1:8006".to_string(),
            xavier_port: 8006,
            config_dir,
        }
    }

    /// Creates a custom configured `SwalDoctorEngine`.
    pub fn with_config(config_dir: PathBuf, xavier_url: &str, xavier_port: u16) -> Self {
        Self {
            xavier_url: xavier_url.to_string(),
            xavier_port,
            config_dir,
        }
    }

    /// Runs all system diagnostic probes and aggregates the `SwalDoctorReport`.
    pub async fn run_full_diagnostics(&self) -> SwalDoctorReport {
        let mut checks = Vec::new();

        checks.push(self.probe_wayland_socket().await);
        checks.push(self.probe_gpu_acceleration().await);
        checks.push(self.probe_xavier_connectivity().await);
        checks.push(self.probe_settings_store().await);
        checks.push(self.probe_disk_space().await);

        SwalDoctorReport::new(checks)
    }

    /// Probes Wayland compositor socket availability.
    pub async fn probe_wayland_socket(&self) -> DiagnosticCheck {
        let wayland_display = std::env::var("WAYLAND_DISPLAY").ok();
        let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok();

        let mut found = false;
        let mut detail = String::new();

        if let (Some(runtime_dir), Some(display)) = (&xdg_runtime_dir, &wayland_display) {
            let socket_path = PathBuf::from(runtime_dir).join(display);
            if socket_path.exists() {
                found = true;
                detail = format!("Wayland socket available at {}", socket_path.display());
            }
        }

        if !found {
            let candidates = vec![
                PathBuf::from("/run/user/1000/wayland-0"),
                PathBuf::from("/tmp/wayland-0"),
            ];
            for path in candidates {
                if path.exists() {
                    found = true;
                    detail = format!("Wayland socket found at {}", path.display());
                    break;
                }
            }
        }

        if found {
            DiagnosticCheck {
                name: "wayland_socket".to_string(),
                category: "Display".to_string(),
                status: CheckStatus::Pass,
                message: detail,
                auto_fixable: false,
            }
        } else if let Some(display) = wayland_display {
            DiagnosticCheck {
                name: "wayland_socket".to_string(),
                category: "Display".to_string(),
                status: CheckStatus::Warning,
                message: format!(
                    "WAYLAND_DISPLAY set ({}) but socket file unreachable",
                    display
                ),
                auto_fixable: false,
            }
        } else {
            DiagnosticCheck {
                name: "wayland_socket".to_string(),
                category: "Display".to_string(),
                status: CheckStatus::Warning,
                message: "No Wayland compositor session active (WAYLAND_DISPLAY unset)".to_string(),
                auto_fixable: false,
            }
        }
    }

    /// Probes Direct Rendering Manager (DRI) GPU nodes for graphics hardware acceleration.
    pub async fn probe_gpu_acceleration(&self) -> DiagnosticCheck {
        let dri_render = PathBuf::from("/dev/dri/renderD128");
        let dri_card = PathBuf::from("/dev/dri/card0");
        let sys_drm = PathBuf::from("/sys/class/drm");

        if dri_render.exists() || dri_card.exists() || sys_drm.exists() {
            DiagnosticCheck {
                name: "gpu_acceleration".to_string(),
                category: "Graphics".to_string(),
                status: CheckStatus::Pass,
                message: "GPU Direct Rendering Manager (DRI) device nodes available".to_string(),
                auto_fixable: false,
            }
        } else {
            DiagnosticCheck {
                name: "gpu_acceleration".to_string(),
                category: "Graphics".to_string(),
                status: CheckStatus::Warning,
                message: "No DRI GPU nodes detected (software rasterizer fallback active)".to_string(),
                auto_fixable: false,
            }
        }
    }

    /// Probes Xavier GraphRAG Core TCP connectivity.
    pub async fn probe_xavier_connectivity(&self) -> DiagnosticCheck {
        let addr = format!("127.0.0.1:{}", self.xavier_port);
        match tokio::time::timeout(
            Duration::from_millis(300),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_stream)) => DiagnosticCheck {
                name: "xavier_connectivity".to_string(),
                category: "Xavier Core".to_string(),
                status: CheckStatus::Pass,
                message: format!("Xavier GraphRAG listening on port {}", self.xavier_port),
                auto_fixable: false,
            },
            _ => DiagnosticCheck {
                name: "xavier_connectivity".to_string(),
                category: "Xavier Core".to_string(),
                status: CheckStatus::Warning,
                message: format!(
                    "Xavier GraphRAG unreachable at {} (port {})",
                    self.xavier_url, self.xavier_port
                ),
                auto_fixable: false,
            },
        }
    }

    /// Probes SWAL settings store existence and JSON validity.
    pub async fn probe_settings_store(&self) -> DiagnosticCheck {
        let settings_file = self.config_dir.join("settings.json");

        if !self.config_dir.exists() {
            return DiagnosticCheck {
                name: "settings_store".to_string(),
                category: "Configuration".to_string(),
                status: CheckStatus::Fail,
                message: format!("Settings directory missing at {}", self.config_dir.display()),
                auto_fixable: true,
            };
        }

        if !settings_file.exists() {
            return DiagnosticCheck {
                name: "settings_store".to_string(),
                category: "Configuration".to_string(),
                status: CheckStatus::Fail,
                message: format!("Settings file missing at {}", settings_file.display()),
                auto_fixable: true,
            };
        }

        match std::fs::read_to_string(&settings_file) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => DiagnosticCheck {
                    name: "settings_store".to_string(),
                    category: "Configuration".to_string(),
                    status: CheckStatus::Pass,
                    message: format!("Settings store valid at {}", settings_file.display()),
                    auto_fixable: true,
                },
                Err(err) => DiagnosticCheck {
                    name: "settings_store".to_string(),
                    category: "Configuration".to_string(),
                    status: CheckStatus::Fail,
                    message: format!("Settings store JSON corrupted: {}", err),
                    auto_fixable: true,
                },
            },
            Err(err) => DiagnosticCheck {
                name: "settings_store".to_string(),
                category: "Configuration".to_string(),
                status: CheckStatus::Fail,
                message: format!("Failed to read settings file: {}", err),
                auto_fixable: true,
            },
        }
    }

    /// Probes storage disk space accessibility.
    pub async fn probe_disk_space(&self) -> DiagnosticCheck {
        let check_path = if self.config_dir.exists() {
            &self.config_dir
        } else {
            Path::new("/tmp")
        };

        match std::fs::metadata(check_path) {
            Ok(_) => DiagnosticCheck {
                name: "disk_space".to_string(),
                category: "Storage".to_string(),
                status: CheckStatus::Pass,
                message: format!("Storage filesystem accessible at {}", check_path.display()),
                auto_fixable: false,
            },
            Err(err) => DiagnosticCheck {
                name: "disk_space".to_string(),
                category: "Storage".to_string(),
                status: CheckStatus::Warning,
                message: format!("Storage disk space check warning: {}", err),
                auto_fixable: false,
            },
        }
    }

    /// Attempts auto-healing repair actions for specified diagnostic checks.
    pub fn attempt_auto_fix(&self, check_name: &str) -> bool {
        match check_name {
            "settings_store" | "settings" | "settings.json" => self.fix_settings_store(),
            "directory_structure" | "directories" | "config_directory" => self.fix_directory_structure(),
            _ => false,
        }
    }

    fn fix_settings_store(&self) -> bool {
        if std::fs::create_dir_all(&self.config_dir).is_err() {
            return false;
        }

        let settings_file = self.config_dir.join("settings.json");
        let default_settings = serde_json::json!({
            "version": 1,
            "theme": "fluent-mica",
            "telemetry_enabled": true,
            "auto_healing": true
        });

        match serde_json::to_string_pretty(&default_settings) {
            Ok(json) => std::fs::write(&settings_file, json).is_ok(),
            Err(_) => false,
        }
    }

    fn fix_directory_structure(&self) -> bool {
        let dir1 = std::fs::create_dir_all(&self.config_dir);
        let run_dir = PathBuf::from("/tmp/swal");
        let dir2 = std::fs::create_dir_all(&run_dir);
        dir1.is_ok() && dir2.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("swal_doc_test_{}_{}", prefix, nanos));
        let _ = std::fs::create_dir_all(&path);
        path
    }

    #[test]
    fn test_report_aggregation() {
        let checks = vec![
            DiagnosticCheck {
                name: "chk1".to_string(),
                category: "Cat".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                auto_fixable: false,
            },
            DiagnosticCheck {
                name: "chk2".to_string(),
                category: "Cat".to_string(),
                status: CheckStatus::Warning,
                message: "Warn".to_string(),
                auto_fixable: false,
            },
            DiagnosticCheck {
                name: "chk3".to_string(),
                category: "Cat".to_string(),
                status: CheckStatus::Fail,
                message: "Error".to_string(),
                auto_fixable: true,
            },
        ];

        let report = SwalDoctorReport::new(checks);
        assert!(!report.all_passed);
        assert_eq!(report.error_count, 1);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.checks.len(), 3);
    }

    #[tokio::test]
    async fn test_settings_store_probe_and_auto_fix() {
        let test_base = create_test_dir("settings");
        let config_dir = test_base.join(".config").join("swal");

        let engine = SwalDoctorEngine::with_config(config_dir.clone(), "http://127.0.0.1:8006", 8006);

        // Initially fails because directory & settings file do not exist
        let check = engine.probe_settings_store().await;
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.auto_fixable);

        // Execute auto fix
        let fixed = engine.attempt_auto_fix("settings_store");
        assert!(fixed);

        // Check again, should pass
        let check_after = engine.probe_settings_store().await;
        assert_eq!(check_after.status, CheckStatus::Pass);

        // Verify corrupted JSON auto-fix recovery
        std::fs::write(config_dir.join("settings.json"), "{ invalid json").expect("Failed to write corrupt json");
        let check_corrupt = engine.probe_settings_store().await;
        assert_eq!(check_corrupt.status, CheckStatus::Fail);

        let fixed_corrupt = engine.attempt_auto_fix("settings.json");
        assert!(fixed_corrupt);

        let check_corrupt_after = engine.probe_settings_store().await;
        assert_eq!(check_corrupt_after.status, CheckStatus::Pass);

        let _ = std::fs::remove_dir_all(&test_base);
    }

    #[tokio::test]
    async fn test_directory_structure_auto_fix() {
        let test_base = create_test_dir("dirs");
        let config_dir = test_base.join("custom_swal_dir");

        let engine = SwalDoctorEngine::with_config(config_dir.clone(), "http://127.0.0.1:8006", 8006);
        let fixed = engine.attempt_auto_fix("directory_structure");
        assert!(fixed);
        assert!(config_dir.exists());

        let _ = std::fs::remove_dir_all(&test_base);
    }

    #[tokio::test]
    async fn test_run_full_diagnostics() {
        let test_base = create_test_dir("full_diag");
        let config_dir = test_base.join(".config").join("swal");

        let engine = SwalDoctorEngine::with_config(config_dir, "http://127.0.0.1:8006", 59998);

        let report = engine.run_full_diagnostics().await;
        assert_eq!(report.checks.len(), 5);

        let check_names: Vec<String> = report.checks.into_iter().map(|c| c.name).collect();
        assert!(check_names.contains(&"wayland_socket".to_string()));
        assert!(check_names.contains(&"gpu_acceleration".to_string()));
        assert!(check_names.contains(&"xavier_connectivity".to_string()));
        assert!(check_names.contains(&"settings_store".to_string()));
        assert!(check_names.contains(&"disk_space".to_string()));

        let _ = std::fs::remove_dir_all(&test_base);
    }

    #[test]
    fn test_unknown_check_auto_fix() {
        let engine = SwalDoctorEngine::new();
        assert!(!engine.attempt_auto_fix("unknown_check"));
    }
}
