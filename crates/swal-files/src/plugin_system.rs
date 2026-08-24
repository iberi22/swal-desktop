//! Agentic File Tool Plugin System for SWAL Files.
//! Provides dynamic plugin discovery, manifest parsing, trigger matching, and execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Triggers that activate a plugin action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginTrigger {
    ContextMenu,
    OmnibarCommand(String),
    FileOpenHook(String),
    AutoTransform,
}

/// Target selection requirements for a plugin
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginTarget {
    SingleFile,
    MultiSelection,
    Directory,
    Any,
}

/// Declarative manifest describing a file tool plugin
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilePluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub triggers: Vec<PluginTrigger>,
    pub target_types: Vec<String>,
    pub executable_command: String,
}

/// Execution context passed to plugins when invoked
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PluginExecutionContext {
    pub selected_paths: Vec<String>,
    pub current_directory: String,
    pub environment_vars: HashMap<String, String>,
}

/// Result returned after executing a plugin command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PluginExecutionResult {
    pub success: bool,
    pub output_message: String,
    pub modified_files: Vec<String>,
    pub created_files: Vec<String>,
}

/// Central registry managing registered file tool plugins
#[derive(Debug, Clone, Default)]
pub struct FilePluginRegistry {
    plugins: HashMap<String, FilePluginManifest>,
}

impl FilePluginRegistry {
    /// Create a new empty `FilePluginRegistry`
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Load plugin manifests from a directory containing `.json` manifest files
    /// or subdirectories with `plugin.json` / `manifest.json`.
    pub fn load_from_directory(&mut self, dir: &Path) -> Result<usize, String> {
        if !dir.exists() {
            return Err(format!("Plugin directory does not exist: {}", dir.display()));
        }

        let read_dir = fs::read_dir(dir).map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
        let mut loaded_count = 0;

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(manifest) = Self::load_manifest_file(&path) {
                    self.register_plugin(manifest);
                    loaded_count += 1;
                }
            } else if path.is_dir() {
                let plugin_json = path.join("plugin.json");
                let manifest_json = path.join("manifest.json");

                let manifest_path = if plugin_json.exists() {
                    Some(plugin_json)
                } else if manifest_json.exists() {
                    Some(manifest_json)
                } else {
                    None
                };

                if let Some(mp) = manifest_path {
                    if let Ok(manifest) = Self::load_manifest_file(&mp) {
                        self.register_plugin(manifest);
                        loaded_count += 1;
                    }
                }
            }
        }

        Ok(loaded_count)
    }

    /// Helper to parse a manifest JSON file
    fn load_manifest_file(path: &Path) -> Result<FilePluginManifest, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read manifest {}: {}", path.display(), e))?;
        let manifest: FilePluginManifest = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON manifest {}: {}", path.display(), e))?;
        Ok(manifest)
    }

    /// Register a single plugin manifest
    pub fn register_plugin(&mut self, manifest: FilePluginManifest) {
        self.plugins.insert(manifest.id.clone(), manifest);
    }

    /// Get a plugin manifest by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&FilePluginManifest> {
        self.plugins.get(plugin_id)
    }

    /// List all registered plugin manifests
    pub fn list_plugins(&self) -> Vec<&FilePluginManifest> {
        self.plugins.values().collect()
    }

    /// Find plugins that match a given target file path and trigger condition
    pub fn find_matching_plugins(&self, file_path: &Path, trigger: &PluginTrigger) -> Vec<&FilePluginManifest> {
        let mut matches = Vec::new();

        let is_dir = file_path.is_dir();
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        for manifest in self.plugins.values() {
            // 1. Check trigger match
            let trigger_matched = manifest.triggers.iter().any(|t| match (t, trigger) {
                (PluginTrigger::ContextMenu, PluginTrigger::ContextMenu) => true,
                (PluginTrigger::AutoTransform, PluginTrigger::AutoTransform) => true,
                (PluginTrigger::OmnibarCommand(c1), PluginTrigger::OmnibarCommand(c2)) => {
                    c1 == "*" || c1.eq_ignore_ascii_case(c2)
                }
                (PluginTrigger::FileOpenHook(h1), PluginTrigger::FileOpenHook(h2)) => {
                    h1 == "*" || h1.eq_ignore_ascii_case(h2)
                }
                _ => false,
            });

            if !trigger_matched {
                continue;
            }

            // 2. Check target_types match
            let target_matched = manifest.target_types.iter().any(|tt| {
                let clean_tt = tt.trim().trim_start_matches('.').to_lowercase();
                if clean_tt == "*" || clean_tt == "any" || clean_tt == "all" {
                    true
                } else if is_dir && (clean_tt == "dir" || clean_tt == "directory") {
                    true
                } else if !is_dir && !extension.is_empty() && clean_tt == extension {
                    true
                } else {
                    false
                }
            });

            if target_matched {
                matches.push(manifest);
            }
        }

        matches.sort_by(|a, b| a.id.cmp(&b.id));
        matches
    }

    /// Execute a plugin by ID given an execution context
    pub fn execute_plugin(&self, plugin_id: &str, ctx: &PluginExecutionContext) -> Result<PluginExecutionResult, String> {
        let manifest = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin with ID '{}' not found in registry", plugin_id))?;

        // Handle mock executions (for testing or lightweight custom hooks)
        if manifest.executable_command.starts_with("mock:") {
            let msg = manifest.executable_command.trim_start_matches("mock:").trim();
            return Ok(PluginExecutionResult {
                success: true,
                output_message: if msg.is_empty() {
                    format!("Executed mock plugin: {}", manifest.name)
                } else {
                    msg.to_string()
                },
                modified_files: ctx.selected_paths.clone(),
                created_files: vec![],
            });
        }

        // Prepare process command
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(&manifest.executable_command);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(&manifest.executable_command);
            c
        };

        if !ctx.current_directory.is_empty() {
            cmd.current_dir(&ctx.current_directory);
        }

        for (k, v) in &ctx.environment_vars {
            cmd.env(k, v);
        }

        cmd.env("SWAL_SELECTED_PATHS", ctx.selected_paths.join(":"));
        cmd.env("SWAL_CURRENT_DIR", &ctx.current_directory);
        cmd.env("SWAL_PLUGIN_ID", &manifest.id);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to launch plugin executable command '{}': {}", manifest.executable_command, e))?;

        let stdout_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).trim().to_string();

        let output_msg = if !stdout_str.is_empty() && !stderr_str.is_empty() {
            format!("{}\n{}", stdout_str, stderr_str)
        } else if !stdout_str.is_empty() {
            stdout_str
        } else {
            stderr_str
        };

        if output.status.success() {
            Ok(PluginExecutionResult {
                success: true,
                output_message: output_msg,
                modified_files: vec![],
                created_files: vec![],
            })
        } else {
            Err(format!(
                "Plugin '{}' process exited with status {}: {}",
                manifest.id,
                output.status,
                output_msg
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manifest_serialization_and_deserialization() {
        let manifest = FilePluginManifest {
            id: "image-optimizer".to_string(),
            name: "Image Optimizer".to_string(),
            version: "1.0.0".to_string(),
            author: "SWAL Team".to_string(),
            description: "Losslessly optimizes PNG and JPEG files".to_string(),
            triggers: vec![
                PluginTrigger::ContextMenu,
                PluginTrigger::OmnibarCommand("optimize".to_string()),
            ],
            target_types: vec!["png".to_string(), "jpg".to_string(), "jpeg".to_string()],
            executable_command: "optipng $SWAL_SELECTED_PATHS".to_string(),
        };

        let json = serde_json::to_string_pretty(&manifest).expect("Serialization failed");
        assert!(json.contains("image-optimizer"));
        assert!(json.contains("ContextMenu"));

        let deserialized: FilePluginManifest = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(manifest, deserialized);
    }

    #[test]
    fn test_plugin_registry_registration_and_lookup() {
        let mut registry = FilePluginRegistry::new();
        assert_eq!(registry.list_plugins().len(), 0);

        let manifest = FilePluginManifest {
            id: "code-formatter".to_string(),
            name: "Code Formatter".to_string(),
            version: "0.1.0".to_string(),
            author: "Dev".to_string(),
            description: "Format source code".to_string(),
            triggers: vec![PluginTrigger::ContextMenu],
            target_types: vec!["rs".to_string(), "json".to_string()],
            executable_command: "rustfmt $SWAL_SELECTED_PATHS".to_string(),
        };

        registry.register_plugin(manifest.clone());
        assert_eq!(registry.list_plugins().len(), 1);
        assert_eq!(registry.get_plugin("code-formatter"), Some(&manifest));
    }

    #[test]
    fn test_load_from_directory() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path();

        // 1. Root level JSON manifest
        let manifest1 = FilePluginManifest {
            id: "p1".to_string(),
            name: "Plugin 1".to_string(),
            version: "1.0".to_string(),
            author: "A".to_string(),
            description: "D1".to_string(),
            triggers: vec![PluginTrigger::ContextMenu],
            target_types: vec!["txt".to_string()],
            executable_command: "echo p1".to_string(),
        };
        let file1 = plugin_dir.join("p1.json");
        fs::write(&file1, serde_json::to_string(&manifest1).unwrap()).unwrap();

        // 2. Subdirectory plugin with plugin.json
        let sub_dir = plugin_dir.join("p2_folder");
        fs::create_dir(&sub_dir).unwrap();
        let manifest2 = FilePluginManifest {
            id: "p2".to_string(),
            name: "Plugin 2".to_string(),
            version: "2.0".to_string(),
            author: "B".to_string(),
            description: "D2".to_string(),
            triggers: vec![PluginTrigger::OmnibarCommand("p2".to_string())],
            target_types: vec!["*".to_string()],
            executable_command: "echo p2".to_string(),
        };
        fs::write(sub_dir.join("plugin.json"), serde_json::to_string(&manifest2).unwrap()).unwrap();

        let mut registry = FilePluginRegistry::new();
        let count = registry.load_from_directory(plugin_dir).unwrap();
        assert_eq!(count, 2);
        assert!(registry.get_plugin("p1").is_some());
        assert!(registry.get_plugin("p2").is_some());
    }

    #[test]
    fn test_trigger_and_target_matching() {
        let mut registry = FilePluginRegistry::new();

        let rust_fmt = FilePluginManifest {
            id: "rust-fmt".to_string(),
            name: "Rust Fmt".to_string(),
            version: "1.0".to_string(),
            author: "Core".to_string(),
            description: "Format Rust files".to_string(),
            triggers: vec![
                PluginTrigger::ContextMenu,
                PluginTrigger::FileOpenHook("rs".to_string()),
            ],
            target_types: vec!["rs".to_string()],
            executable_command: "echo format".to_string(),
        };

        let universal_cleaner = FilePluginManifest {
            id: "cleaner".to_string(),
            name: "Cleaner".to_string(),
            version: "1.0".to_string(),
            author: "Core".to_string(),
            description: "Clean files".to_string(),
            triggers: vec![PluginTrigger::ContextMenu],
            target_types: vec!["*".to_string()],
            executable_command: "echo clean".to_string(),
        };

        registry.register_plugin(rust_fmt);
        registry.register_plugin(universal_cleaner);

        // Matching context menu on main.rs
        let matches = registry.find_matching_plugins(Path::new("src/main.rs"), &PluginTrigger::ContextMenu);
        assert_eq!(matches.len(), 2);

        // Matching context menu on document.txt
        let txt_matches = registry.find_matching_plugins(Path::new("doc.txt"), &PluginTrigger::ContextMenu);
        assert_eq!(txt_matches.len(), 1);
        assert_eq!(txt_matches[0].id, "cleaner");

        // Matching file open hook on main.rs
        let hook_matches = registry.find_matching_plugins(Path::new("src/main.rs"), &PluginTrigger::FileOpenHook("rs".to_string()));
        assert_eq!(hook_matches.len(), 1);
        assert_eq!(hook_matches[0].id, "rust-fmt");
    }

    #[test]
    fn test_mock_plugin_execution() {
        let mut registry = FilePluginRegistry::new();
        let mock_manifest = FilePluginManifest {
            id: "summarizer".to_string(),
            name: "AI Markdown Summarizer".to_string(),
            version: "1.0.0".to_string(),
            author: "SWAL Agent".to_string(),
            description: "Generates summary of markdown file".to_string(),
            triggers: vec![PluginTrigger::ContextMenu],
            target_types: vec!["md".to_string()],
            executable_command: "mock: Summary generated successfully for selected files".to_string(),
        };

        registry.register_plugin(mock_manifest);

        let ctx = PluginExecutionContext {
            selected_paths: vec!["README.md".to_string()],
            current_directory: "/tmp".to_string(),
            environment_vars: HashMap::new(),
        };

        let result = registry.execute_plugin("summarizer", &ctx).expect("Execution failed");
        assert!(result.success);
        assert_eq!(result.output_message, "Summary generated successfully for selected files");
        assert_eq!(result.modified_files, vec!["README.md"]);
    }

    #[test]
    fn test_shell_plugin_execution() {
        let mut registry = FilePluginRegistry::new();
        let echo_manifest = FilePluginManifest {
            id: "echo-tool".to_string(),
            name: "Echo Tool".to_string(),
            version: "1.0".to_string(),
            author: "Tester".to_string(),
            description: "Echos input".to_string(),
            triggers: vec![PluginTrigger::ContextMenu],
            target_types: vec!["*".to_string()],
            executable_command: "echo 'Hello SWAL Plugin'".to_string(),
        };

        registry.register_plugin(echo_manifest);

        let ctx = PluginExecutionContext {
            selected_paths: vec!["file.txt".to_string()],
            current_directory: "/tmp".to_string(),
            environment_vars: HashMap::new(),
        };

        let result = registry.execute_plugin("echo-tool", &ctx).expect("Execution failed");
        assert!(result.success);
        assert!(result.output_message.contains("Hello SWAL Plugin"));
    }
}
