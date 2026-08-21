//! Omnibar & Command Palette parser (inspired by Files Omnibar)

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OmnibarIntent {
    Navigate(PathBuf),
    SearchQuery(String),
    AgentPrompt(String),
    Command(String),
}

pub fn parse_omnibar_input(input: &str, current_dir: &Path) -> OmnibarIntent {
    let trimmed = input.trim();
    if trimmed.starts_with('@') {
        let prompt = trimmed.trim_start_matches('@').trim().to_string();
        return OmnibarIntent::AgentPrompt(prompt);
    }

    if trimmed.starts_with('>') {
        let cmd = trimmed.trim_start_matches('>').trim().to_string();
        return OmnibarIntent::Command(cmd);
    }

    if trimmed.starts_with('?') {
        let query = trimmed.trim_start_matches('?').trim().to_string();
        return OmnibarIntent::SearchQuery(query);
    }

    // Direct path or relative path
    let candidate = if trimmed.starts_with('/') {
        PathBuf::from(trimmed)
    } else if trimmed.starts_with('~') {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/belal"));
        home.join(trimmed.trim_start_matches("~/").trim_start_matches('~'))
    } else {
        current_dir.join(trimmed)
    };

    if candidate.exists() {
        OmnibarIntent::Navigate(candidate)
    } else {
        OmnibarIntent::SearchQuery(trimmed.to_string())
    }
}
