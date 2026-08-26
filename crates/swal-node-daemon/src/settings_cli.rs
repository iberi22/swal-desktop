//! Settings CLI Companion Tool for SWAL Desktop
//!
//! Provides `SettingsCliRunner` for querying, updating, listing, resetting, and exporting
//! canonical JSON schema for system settings over dot-notation keys with IPC broadcast.

use serde_json::{json, Value};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/swal_hermes_orb.sock";

/// Canonical JSON schema definition for SWAL Desktop system settings.
pub const CANONICAL_SETTINGS_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SWAL System Settings Schema",
  "type": "object",
  "properties": {
    "system": {
      "type": "object",
      "properties": {
        "hostname": { "type": "string" },
        "volume": { "type": "integer", "minimum": 0, "maximum": 100 },
        "muted": { "type": "boolean" },
        "power_mode": { "type": "string" }
      },
      "required": ["hostname", "volume", "muted", "power_mode"]
    },
    "appearance": {
      "type": "object",
      "properties": {
        "theme": { "type": "string" },
        "accent_color": { "type": "string" },
        "mica_effect": { "type": "boolean" },
        "scaling": { "type": "number" }
      },
      "required": ["theme", "accent_color", "mica_effect", "scaling"]
    },
    "display": {
      "type": "object",
      "properties": {
        "resolution": { "type": "string" },
        "refresh_rate": { "type": "integer" },
        "brightness": { "type": "integer", "minimum": 0, "maximum": 100 }
      },
      "required": ["resolution", "refresh_rate", "brightness"]
    },
    "network": {
      "type": "object",
      "properties": {
        "wifi_enabled": { "type": "boolean" },
        "bluetooth_enabled": { "type": "boolean" }
      },
      "required": ["wifi_enabled", "bluetooth_enabled"]
    },
    "hermes": {
      "type": "object",
      "properties": {
        "orb_state": { "type": "string" },
        "cognitive_mode": { "type": "string" }
      },
      "required": ["orb_state", "cognitive_mode"]
    }
  },
  "required": ["system", "appearance", "display", "network", "hermes"]
}"#;

/// Returns the factory default system settings as JSON `Value`.
pub fn default_settings() -> Value {
    json!({
        "system": {
            "hostname": "swal-desktop",
            "volume": 80,
            "muted": false,
            "power_mode": "performance"
        },
        "appearance": {
            "theme": "fluent-dark",
            "accent_color": "#0078d4",
            "mica_effect": true,
            "scaling": 1.0
        },
        "display": {
            "resolution": "2560x1440",
            "refresh_rate": 240,
            "brightness": 100
        },
        "network": {
            "wifi_enabled": true,
            "bluetooth_enabled": true
        },
        "hermes": {
            "orb_state": "idle",
            "cognitive_mode": "active"
        }
    })
}

/// Settings CLI runner for managing system configuration and broadcasting IPC updates.
#[derive(Debug, Clone)]
pub struct SettingsCliRunner {
    settings: Value,
    socket_path: PathBuf,
}

impl Default for SettingsCliRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsCliRunner {
    /// Creates a new `SettingsCliRunner` with default settings and standard IPC socket path.
    pub fn new() -> Self {
        Self::with_socket_path(DEFAULT_SOCKET_PATH)
    }

    /// Creates a `SettingsCliRunner` with a custom IPC socket path.
    pub fn with_socket_path<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            settings: default_settings(),
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    /// Returns the active IPC socket path.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Returns the canonical JSON schema string for system settings.
    pub fn export_schema() -> String {
        CANONICAL_SETTINGS_SCHEMA.to_string()
    }

    /// Retrieves the setting value for a dot-notation key (e.g., "appearance.theme").
    pub fn get(&self, key: &str) -> String {
        if key.trim().is_empty() {
            return "Error: Key path cannot be empty".to_string();
        }
        match self.resolve_key_path(key) {
            Some(val) => {
                if let Some(s) = val.as_str() {
                    s.to_string()
                } else {
                    val.to_string()
                }
            }
            None => format!("Error: Key '{}' not found", key),
        }
    }

    /// Updates the value for a dot-notation key and triggers Unix socket IPC broadcast.
    pub fn set(&mut self, key: &str, value: &str) -> Result<String, String> {
        let trimmed_key = key.trim();
        if trimmed_key.is_empty() {
            return Err("Error: Key path cannot be empty".to_string());
        }

        if self.resolve_key_path(trimmed_key).is_none() {
            return Err(format!("Error: Key '{}' not found", trimmed_key));
        }

        let parsed_value = parse_setting_value(value);
        self.mutate_key_path(trimmed_key, parsed_value.clone())?;

        // Trigger IPC broadcast
        self.trigger_ipc_broadcast(trimmed_key, &parsed_value);

        Ok(format!("Setting '{}' updated to '{}'", trimmed_key, value))
    }

    /// Lists all settings formatted as pretty-printed JSON.
    pub fn list(&self) -> String {
        serde_json::to_string_pretty(&self.settings)
            .unwrap_or_else(|e| format!("Error formatting settings JSON: {}", e))
    }

    /// Resets all settings to factory default values and triggers IPC broadcast.
    pub fn reset(&mut self) -> Result<String, String> {
        self.settings = default_settings();
        self.trigger_ipc_broadcast("all", &json!({"action": "reset"}));
        Ok("Settings reset to factory defaults".to_string())
    }

    /// Navigates down a dot-separated path to find a value reference.
    fn resolve_key_path(&self, key: &str) -> Option<&Value> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut curr = &self.settings;
        for part in parts {
            match curr {
                Value::Object(map) => {
                    curr = map.get(part)?;
                }
                _ => return None,
            }
        }
        Some(curr)
    }

    /// Mutates the target value at a dot-separated path.
    fn mutate_key_path(&mut self, key: &str, new_value: Value) -> Result<(), String> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut curr = &mut self.settings;
        for (i, &part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                if let Value::Object(map) = curr {
                    map.insert(part.to_string(), new_value);
                    return Ok(());
                } else {
                    return Err(format!("Error: Parent path for '{}' is not an object", key));
                }
            } else {
                if let Value::Object(map) = curr {
                    if let Some(next) = map.get_mut(part) {
                        curr = next;
                    } else {
                        return Err(format!("Error: Key '{}' not found", key));
                    }
                } else {
                    return Err(format!("Error: Key '{}' not found", key));
                }
            }
        }
        Err(format!("Error: Key '{}' not found", key))
    }

    /// Sends a JSON payload over the Unix domain socket if listener is online.
    fn trigger_ipc_broadcast(&self, key: &str, value: &Value) {
        if !self.socket_path.exists() {
            return;
        }

        let payload = json!({
            "cmd": "settings_update",
            "key": key,
            "value": value
        });

        if let Ok(mut stream) = UnixStream::connect(&self.socket_path) {
            let _ = writeln!(stream, "{}", payload.to_string());
            let _ = stream.flush();
        }
    }

    /// Evaluates CLI command arguments and returns execution output string.
    pub fn run_cli_args(&mut self, args: &[String]) -> String {
        if args.is_empty() {
            return "Usage: swal-settings <get|set|list|reset|export-schema> [args...]".to_string();
        }

        match args[0].as_str() {
            "get" => {
                if args.len() < 2 {
                    "Error: Missing key argument for 'get' command".to_string()
                } else {
                    self.get(&args[1])
                }
            }
            "set" => {
                if args.len() < 3 {
                    "Error: Usage: set <key> <value>".to_string()
                } else {
                    match self.set(&args[1], &args[2]) {
                        Ok(msg) => msg,
                        Err(err) => err,
                    }
                }
            }
            "list" => self.list(),
            "reset" => match self.reset() {
                Ok(msg) => msg,
                Err(err) => err,
            },
            "export-schema" | "export_schema" | "--schema" => Self::export_schema(),
            "--help" | "-h" | "help" => {
                "SWAL Settings CLI Companion Tool\n\n\
                Commands:\n\
                get <key>           Get value for dot-notation key\n\
                set <key> <val>     Set value for dot-notation key and broadcast IPC\n\
                list                List all settings in pretty JSON\n\
                reset               Reset settings to factory defaults\n\
                export-schema       Export canonical JSON schema".to_string()
            }
            unknown => format!("Error: Unknown command '{}'. Use --help for usage.", unknown),
        }
    }
}

/// Helper function to parse user input string into JSON Value types (bool, number, json, or string).
fn parse_setting_value(val: &str) -> Value {
    if val == "true" {
        Value::Bool(true)
    } else if val == "false" {
        Value::Bool(false)
    } else if let Ok(i) = val.parse::<i64>() {
        Value::Number(i.into())
    } else if let Ok(f) = val.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            Value::Number(n)
        } else {
            Value::String(val.to_string())
        }
    } else if let Ok(v) = serde_json::from_str::<Value>(val) {
        v
    } else {
        Value::String(val.to_string())
    }
}

/// Binary entrypoint for the `swal-settings` CLI (replaces eww/scripts/swal_settings.py).
/// Exposed as a [[bin]] target so the settings window and scripts call Rust directly
/// instead of shelling out through Python. See Plan Task 3.2.
pub fn cli_main() {
    let args: Vec<String> = std::env::args().collect();
    let mut runner = SettingsCliRunner::new();
    let output = runner.run_cli_args(&args[1..]);
    println!("{}", output);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::os::unix::net::UnixListener;

    #[test]
    fn test_get_existing_keys() {
        let runner = SettingsCliRunner::new();

        assert_eq!(runner.get("appearance.theme"), "fluent-dark");
        assert_eq!(runner.get("system.volume"), "80");
        assert_eq!(runner.get("network.wifi_enabled"), "true");
        assert_eq!(runner.get("display.refresh_rate"), "240");
        assert_eq!(runner.get("hermes.orb_state"), "idle");
    }

    #[test]
    fn test_get_unknown_key_error_handling() {
        let runner = SettingsCliRunner::new();

        let err1 = runner.get("nonexistent.key");
        assert!(err1.contains("Error: Key 'nonexistent.key' not found"));

        let err2 = runner.get("system.invalid_field");
        assert!(err2.contains("Error: Key 'system.invalid_field' not found"));

        let err3 = runner.get("");
        assert!(err3.contains("Error: Key path cannot be empty"));
    }

    #[test]
    fn test_set_valid_keys_mutation() {
        let mut runner = SettingsCliRunner::new();

        let set_res = runner.set("appearance.theme", "cyber-neon");
        assert!(set_res.is_ok());
        assert_eq!(runner.get("appearance.theme"), "cyber-neon");

        let set_num = runner.set("system.volume", "95");
        assert!(set_num.is_ok());
        assert_eq!(runner.get("system.volume"), "95");

        let set_bool = runner.set("network.wifi_enabled", "false");
        assert!(set_bool.is_ok());
        assert_eq!(runner.get("network.wifi_enabled"), "false");
    }

    #[test]
    fn test_set_unknown_key_error_handling() {
        let mut runner = SettingsCliRunner::new();

        let err = runner.set("invalid.path", "value");
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Error: Key 'invalid.path' not found"));

        let empty_err = runner.set("", "val");
        assert!(empty_err.is_err());
        assert!(empty_err.unwrap_err().contains("Error: Key path cannot be empty"));
    }

    #[test]
    fn test_list_formatting() {
        let runner = SettingsCliRunner::new();
        let list_output = runner.list();

        assert!(list_output.contains("\"theme\": \"fluent-dark\""));
        assert!(list_output.contains("\"hostname\": \"swal-desktop\""));
        assert!(list_output.contains("\"refresh_rate\": 240"));
    }

    #[test]
    fn test_reset_factory_defaults() {
        let mut runner = SettingsCliRunner::new();

        runner.set("appearance.theme", "nord-swal").unwrap();
        assert_eq!(runner.get("appearance.theme"), "nord-swal");

        let reset_res = runner.reset();
        assert!(reset_res.is_ok());
        assert_eq!(runner.get("appearance.theme"), "fluent-dark");
    }

    #[test]
    fn test_export_schema() {
        let schema = SettingsCliRunner::export_schema();
        assert!(schema.contains("\"title\": \"SWAL System Settings Schema\""));
        assert!(schema.contains("\"required\": [\"hostname\", \"volume\", \"muted\", \"power_mode\"]"));
    }

    #[test]
    fn test_cli_args_parsing_branches() {
        let mut runner = SettingsCliRunner::new();

        let get_out = runner.run_cli_args(&["get".to_string(), "appearance.theme".to_string()]);
        assert_eq!(get_out, "fluent-dark");

        let set_out = runner.run_cli_args(&[
            "set".to_string(),
            "appearance.theme".to_string(),
            "hive-dark".to_string(),
        ]);
        assert!(set_out.contains("Setting 'appearance.theme' updated to 'hive-dark'"));
        assert_eq!(runner.get("appearance.theme"), "hive-dark");

        let list_out = runner.run_cli_args(&["list".to_string()]);
        assert!(list_out.contains("hive-dark"));

        let schema_out = runner.run_cli_args(&["export-schema".to_string()]);
        assert!(schema_out.contains("SWAL System Settings Schema"));

        let reset_out = runner.run_cli_args(&["reset".to_string()]);
        assert_eq!(reset_out, "Settings reset to factory defaults");
        assert_eq!(runner.get("appearance.theme"), "fluent-dark");

        let missing_get = runner.run_cli_args(&["get".to_string()]);
        assert!(missing_get.contains("Error: Missing key argument"));

        let missing_set = runner.run_cli_args(&["set".to_string(), "key".to_string()]);
        assert!(missing_set.contains("Error: Usage: set <key> <value>"));

        let help_out = runner.run_cli_args(&["--help".to_string()]);
        assert!(help_out.contains("SWAL Settings CLI Companion Tool"));

        let unknown_cmd = runner.run_cli_args(&["unknown".to_string()]);
        assert!(unknown_cmd.contains("Error: Unknown command 'unknown'"));
    }

    #[test]
    fn test_ipc_socket_broadcast() {
        let socket_path = format!("/tmp/test_settings_cli_{}.sock", std::process::id());
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).expect("Failed to bind test socket");

        let mut runner = SettingsCliRunner::with_socket_path(&socket_path);

        let set_res = runner.set("appearance.theme", "cyber-neon");
        assert!(set_res.is_ok());

        let (mut stream, _) = listener.accept().expect("Failed to accept IPC connection");
        let mut buf = String::new();
        stream.read_to_string(&mut buf).expect("Failed to read IPC packet");

        assert!(buf.contains("settings_update"));
        assert!(buf.contains("appearance.theme"));
        assert!(buf.contains("cyber-neon"));

        let _ = std::fs::remove_file(&socket_path);
    }
}
