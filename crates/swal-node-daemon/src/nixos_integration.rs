//! nixos_integration.rs
//! Declarative NixOS Flake & Systemd User Units Generator in Pure Rust

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    OnAbnormal,
    No,
}

impl RestartPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            RestartPolicy::Always => "always",
            RestartPolicy::OnFailure => "on-failure",
            RestartPolicy::OnAbnormal => "on-abnormal",
            RestartPolicy::No => "no",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdUnit {
    pub name: String,
    pub description: String,
    pub exec_start: String,
    pub exec_stop: Option<String>,
    pub restart: RestartPolicy,
    pub restart_sec: u32,
    pub environment: HashMap<String, String>,
    pub wanted_by: Vec<String>,
    pub after: Vec<String>,
}

impl SystemdUnit {
    pub fn new(name: &str, description: &str, exec_start: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            exec_start: exec_start.to_string(),
            exec_stop: None,
            restart: RestartPolicy::OnFailure,
            restart_sec: 2,
            environment: HashMap::new(),
            wanted_by: vec!["graphical-session.target".to_string()],
            after: vec!["graphical-session.target".to_string()],
        }
    }

    pub fn to_unit_file_content(&self) -> String {
        let mut out = String::new();
        out.push_str("[Unit]\n");
        out.push_str(&format!("Description={}\n", self.description));
        if !self.after.is_empty() {
            out.push_str(&format!("After={}\n", self.after.join(" ")));
        }
        out.push_str("\n[Service]\nType=simple\n");
        out.push_str(&format!("ExecStart={}\n", self.exec_start));
        if let Some(stop) = &self.exec_stop {
            out.push_str(&format!("ExecStop={}\n", stop));
        }
        out.push_str(&format!("Restart={}\n", self.restart.as_str()));
        out.push_str(&format!("RestartSec={}s\n", self.restart_sec));

        for (k, v) in &self.environment {
            out.push_str(&format!("Environment=\"{}={}\"\n", k, v));
        }

        out.push_str("\n[Install]\n");
        if !self.wanted_by.is_empty() {
            out.push_str(&format!("WantedBy={}\n", self.wanted_by.join(" ")));
        }
        out
    }
}

pub struct NixOsServiceGenerator;

impl NixOsServiceGenerator {
    pub fn default_desktop_units() -> Vec<SystemdUnit> {
        vec![
            SystemdUnit::new(
                "swal-node-daemon.service",
                "SWAL Desktop Supervisor Node Daemon",
                "/run/current-system/sw/bin/swal-node-daemon",
            ),
            SystemdUnit::new(
                "swal-orb.service",
                "Hermes Ambient Voice & Cognition Orb Surface",
                "/run/current-system/sw/bin/swal-orb",
            ),
            SystemdUnit::new(
                "swal-files.service",
                "SWAL Agentic Native File Manager",
                "/run/current-system/sw/bin/swal-files --daemon",
            ),
        ]
    }

    pub fn generate_flake_nix_module_snippet() -> &'static str {
        r#"{ config, lib, pkgs, ... }:
{
  options.programs.swal-desktop = {
    enable = lib.mkEnableOption "SWAL Agentic Desktop Environment";
  };
}"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemd_unit_creation_and_defaults() {
        let unit = SystemdUnit::new("swal.service", "SWAL Desktop", "/bin/swal");
        assert_eq!(unit.name, "swal.service");
        assert_eq!(unit.restart, RestartPolicy::OnFailure);
        assert_eq!(unit.restart_sec, 2);
        assert_eq!(unit.wanted_by, vec!["graphical-session.target"]);
    }

    #[test]
    fn test_unit_file_content_generation() {
        let mut unit = SystemdUnit::new("swal-daemon.service", "Daemon", "/usr/bin/daemon");
        unit.environment.insert("RUST_LOG".to_string(), "info".to_string());

        let content = unit.to_unit_file_content();
        assert!(content.contains("[Unit]"));
        assert!(content.contains("Description=Daemon"));
        assert!(content.contains("[Service]"));
        assert!(content.contains("ExecStart=/usr/bin/daemon"));
        assert!(content.contains("Restart=on-failure"));
        assert!(content.contains("Environment=\"RUST_LOG=info\""));
        assert!(content.contains("[Install]"));
        assert!(content.contains("WantedBy=graphical-session.target"));
    }

    #[test]
    fn test_restart_policy_serialization() {
        assert_eq!(RestartPolicy::Always.as_str(), "always");
        assert_eq!(RestartPolicy::OnFailure.as_str(), "on-failure");
        assert_eq!(RestartPolicy::OnAbnormal.as_str(), "on-abnormal");
        assert_eq!(RestartPolicy::No.as_str(), "no");
    }

    #[test]
    fn test_default_desktop_units_generation() {
        let units = NixOsServiceGenerator::default_desktop_units();
        assert_eq!(units.len(), 3);
        assert_eq!(units[0].name, "swal-node-daemon.service");
        assert_eq!(units[1].name, "swal-orb.service");
        assert_eq!(units[2].name, "swal-files.service");
    }

    #[test]
    fn test_flake_nix_module_snippet_validity() {
        let snippet = NixOsServiceGenerator::generate_flake_nix_module_snippet();
        assert!(snippet.contains("programs.swal-desktop"));
        assert!(snippet.contains("lib.mkEnableOption"));
    }
}
