//! Canonical per-user path resolution (single source of truth for `$HOME`).
//!
//! Phase 2 portability: no hardcoded personal paths anywhere in the workspace.
//! Every helper resolves against the real environment (`dirs`, XDG base dir
//! spec) and falls back to a portable default instead of a personal path.

use std::path::PathBuf;

/// Returns the canonical home directory, falling back to `/` when unresolvable.
pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

/// Returns the SWAL config directory: `$XDG_CONFIG_HOME/swal` if set,
/// otherwise `~/.config/swal`.
pub fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".config"))
        .join("swal")
}

/// Returns the legacy EWW scripts directory (`~/.config/eww/scripts`), used by
/// the hybrid fallback layer until the zero-EWW milestone (Phase 3) lands.
pub fn eww_scripts_dir() -> PathBuf {
    home_dir().join(".config").join("eww").join("scripts")
}

/// Returns `~/.local/bin`, the user-local executables directory.
pub fn local_bin_dir() -> PathBuf {
    home_dir().join(".local").join("bin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_dir_never_empty() {
        // home_dir() must always produce a usable absolute-ish fallback.
        let h = home_dir();
        assert!(!h.as_os_str().is_empty());
    }

    #[test]
    fn config_dir_ends_with_swal() {
        let c = config_dir();
        assert!(c.ends_with("swal"), "got: {:?}", c);
    }

    #[test]
    fn eww_scripts_dir_ends_with_scripts() {
        let s = eww_scripts_dir();
        assert!(s.ends_with("eww/scripts"), "got: {:?}", s);
    }
}