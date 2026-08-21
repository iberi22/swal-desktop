//! Git integration and repository status detection

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitRepoSummary {
    pub is_git_repo: bool,
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged_count: usize,
    pub modified_count: usize,
    pub untracked_count: usize,
    pub conflicted_count: usize,
    pub is_clean: bool,
    pub summary: String,
    pub badge: String,
}

pub fn detect_git_status_for_dir(dir_path: &Path) -> GitRepoSummary {
    let mut current = Some(dir_path);
    let mut repo_root: Option<PathBuf> = None;

    while let Some(d) = current {
        if d.join(".git").exists() {
            repo_root = Some(d.to_path_buf());
            break;
        }
        current = d.parent();
    }

    let root = match repo_root {
        Some(r) => r,
        None => {
            return GitRepoSummary {
                is_git_repo: false,
                branch: String::new(),
                ahead: 0,
                behind: 0,
                staged_count: 0,
                modified_count: 0,
                untracked_count: 0,
                conflicted_count: 0,
                is_clean: true,
                summary: "Sin Git".to_string(),
                badge: String::new(),
            };
        }
    };

    let output = Command::new("git")
        .args(["status", "--porcelain=v2", "--branch"])
        .current_dir(&root)
        .output();

    let mut branch_name = "main".to_string();
    let mut ahead = 0;
    let mut behind = 0;
    let mut staged = 0;
    let mut modified = 0;
    let mut untracked = 0;
    let mut conflicted = 0;

    if let Ok(out) = output {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("# branch.head ") {
                    branch_name = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    if parts.len() >= 2 {
                        ahead = parts[0].trim_start_matches('+').parse().unwrap_or(0);
                        behind = parts[1].trim_start_matches('-').parse().unwrap_or(0);
                    }
                } else if line.starts_with("1 ") || line.starts_with("2 ") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 2 {
                        let xy = fields[1];
                        let mut chars = xy.chars();
                        let staged_ch = chars.next().unwrap_or('.');
                        let unstaged_ch = chars.next().unwrap_or('.');

                        if staged_ch != '.' {
                            staged += 1;
                        }
                        if unstaged_ch != '.' {
                            modified += 1;
                        }
                    }
                } else if line.starts_with("? ") {
                    untracked += 1;
                } else if line.starts_with("u ") {
                    conflicted += 1;
                }
            }
        }
    }

    let is_clean = staged == 0 && modified == 0 && untracked == 0 && conflicted == 0;
    let mut details = Vec::new();
    if staged > 0 {
        details.push(format!("+{} staged", staged));
    }
    if modified > 0 {
        details.push(format!("●{} mod", modified));
    }
    if untracked > 0 {
        details.push(format!("…{} untracked", untracked));
    }
    if conflicted > 0 {
        details.push(format!("!{} conflict", conflicted));
    }

    let tracking_str = if ahead > 0 || behind > 0 {
        format!(" (+{} -{})", ahead, behind)
    } else {
        String::new()
    };

    let summary = if is_clean {
        format!("🌿 {}{} · Limpio", branch_name, tracking_str)
    } else {
        format!("🌿 {}{} · {}", branch_name, tracking_str, details.join(", "))
    };

    let badge = if is_clean {
        "✓ Clean".to_string()
    } else {
        format!("● {} cambios", modified + staged + untracked)
    };

    GitRepoSummary {
        is_git_repo: true,
        branch: branch_name,
        ahead,
        behind,
        staged_count: staged,
        modified_count: modified,
        untracked_count: untracked,
        conflicted_count: conflicted,
        is_clean,
        summary,
        badge,
    }
}
