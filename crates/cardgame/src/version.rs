//! Version information for reproducibility tracking.
//!
//! This module provides version info that should be saved with experiment
//! results to ensure reproducibility. The game engine is deterministic,
//! so recording the version + seed guarantees identical game outcomes.

/// Engine version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Package name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Git commit hash (set at build time via build.rs, or "unknown" if not available)
pub const GIT_HASH: &str = match option_env!("GIT_HASH") {
    Some(hash) => hash,
    None => "unknown",
};

/// Combined version string for logging/saving
pub fn version_string() -> String {
    if GIT_HASH != "unknown" {
        format!("{} v{} ({})", NAME, VERSION, &GIT_HASH[..8.min(GIT_HASH.len())])
    } else {
        format!("{} v{}", NAME, VERSION)
    }
}

/// Version info struct for serialization in experiment configs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionInfo {
    pub name: String,
    pub version: String,
    pub git_hash: Option<String>,
}

impl VersionInfo {
    /// Get current version info
    pub fn current() -> Self {
        Self {
            name: NAME.to_string(),
            version: VERSION.to_string(),
            git_hash: if GIT_HASH != "unknown" {
                Some(GIT_HASH.to_string())
            } else {
                None
            },
        }
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_available() {
        assert!(!VERSION.is_empty());
        assert_eq!(VERSION, "0.7.0");
    }

    #[test]
    fn test_version_string() {
        let vs = version_string();
        assert!(vs.contains("cardgame"));
        assert!(vs.contains("0.7.0"));
    }

    #[test]
    fn test_version_info_serializable() {
        let info = VersionInfo::current();
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("0.7.0"));
    }
}
