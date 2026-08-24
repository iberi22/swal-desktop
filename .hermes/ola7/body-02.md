# [Ola 7.02] feat-swal-62 — Declarative NixOS Flake & Systemd User Units Generator in Pure Rust

> Ola 7 — [Infra/NixOS/Systemd].
> Labels: `ola7`, `wave-7`

---

## Current State (MEDIBLE)
- Feature: `feat-swal-62` at 0% in `.gitcore/features.json`
- Module `crates/swal-node-daemon/src/nixos_integration.rs` will be created.
- Existing tests in `crates/swal-node-daemon`: 77 passing tests.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-node-daemon/src/nixos_integration.rs` providing automated generation, verification, and inspection of declarative systemd user unit definitions (`swal-desktop.service`, `swal-files.service`, `swal-orb.service`), socket activations, and Nix flake configuration validation.
- **File Target**: `crates/swal-node-daemon/src/nixos_integration.rs`

## Web Research Required
1. search: "systemd user units generation format Rust"
2. search: "NixOS declarative module options and flake schema validation"
3. search: "Rust parse generate systemd unit files Type=simple Restart=on-failure"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all tests pass
- [ ] `grep -rn "NixOsServiceGenerator" crates/swal-node-daemon/src/nixos_integration.rs` >= 1 match
- [ ] `grep -rn "SystemdUnit" crates/swal-node-daemon/src/nixos_integration.rs` >= 1 match
- [ ] `grep -rn "generate_unit_file" crates/swal-node-daemon/src/nixos_integration.rs` >= 1 match

## Exact Code Blueprint & Signatures

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
```

## Unit Tests Requirements
1. `test_systemd_unit_creation_and_defaults`
2. `test_unit_file_content_generation`
3. `test_restart_policy_serialization`
4. `test_default_desktop_units_generation`
5. `test_flake_nix_module_snippet_validity`

## Anti-Hallucination Guard
- Do NOT edit other crates or shared files.
- Place all implementation strictly inside `crates/swal-node-daemon/src/nixos_integration.rs`.
