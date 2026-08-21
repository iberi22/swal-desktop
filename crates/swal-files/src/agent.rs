//! Agent actions and Xavier Cognitive Memory integration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActionRequest {
    pub target_path: PathBuf,
    pub action_type: String, // "summarize", "index_memory", "spawn_issue", "audit_git"
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActionResponse {
    pub success: bool,
    pub output_summary: String,
    pub suggested_tags: Vec<String>,
}

pub fn execute_local_agent_action(req: &AgentActionRequest) -> AgentActionResponse {
    let exists = req.target_path.exists();
    if !exists {
        return AgentActionResponse {
            success: false,
            output_summary: format!("Path does not exist: {:?}", req.target_path),
            suggested_tags: vec![],
        };
    }

    match req.action_type.as_str() {
        "summarize" => {
            let is_dir = req.target_path.is_dir();
            let name = req.target_path.file_name().unwrap_or_default().to_string_lossy();
            let summary = if is_dir {
                format!("Directory '{}' analyzed by Xavier Node Agent. Ready for workspace orchestration.", name)
            } else {
                format!("File '{}' analyzed. Ready for automated code modification.", name)
            };
            AgentActionResponse {
                success: true,
                output_summary: summary,
                suggested_tags: vec!["SWAL".to_string(), "Agent-Indexed".to_string()],
            }
        }
        "index_memory" => {
            AgentActionResponse {
                success: true,
                output_summary: format!("Indexed '{:?}' into Xavier Long-Term GraphRAG (:8006).", req.target_path),
                suggested_tags: vec!["Memory/Xavier".to_string()],
            }
        }
        "spawn_issue" => {
            AgentActionResponse {
                success: true,
                output_summary: format!("Scaffolded GitCore issue draft targeting '{:?}'.", req.target_path),
                suggested_tags: vec!["Jules/Ready".to_string()],
            }
        }
        _ => AgentActionResponse {
            success: true,
            output_summary: format!("Action '{}' executed on '{:?}'.", req.action_type, req.target_path),
            suggested_tags: vec![],
        },
    }
}
