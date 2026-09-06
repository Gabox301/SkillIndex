use std::path::PathBuf;

pub use crate::registry::InstallSecurityCheck;

/// Options that control where and how a skill is installed.
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub project_dir: Option<PathBuf>,
    pub registry_dir: Option<PathBuf>,
    pub registry_base_url: Option<String>,
    /// Override for the list of registry base URLs, bypassing the version/main
    /// resolution. Defaults to `None` (production path); tests set it to exercise
    /// the multi-base fallback (e.g. v{ver} → main). Kept as a always-present
    /// field so the struct has one consistent shape across unit tests, integration
    /// tests, and production, instead of a `#[cfg(test)]` field that only exists in
    /// some compilation contexts.
    pub registry_base_urls_override: Option<Vec<String>>,
}

/// Result of a single skill installation.
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub success: bool,
    pub output: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub command: String,
    pub security_check: Option<InstallSecurityCheck>,
}

/// Aggregated result of installing multiple skills.
#[derive(Debug, Clone)]
pub struct InstallAllResult {
    pub installed: usize,
    pub failed: usize,
    pub security_checks: Vec<InstallSecurityCheck>,
    pub errors: Vec<InstallError>,
}

/// Error for a single skill within an `install_all` run.
#[derive(Debug, Clone)]
pub struct InstallError {
    pub name: String,
    pub output: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub command: String,
}

/// Entry describing a skill to be installed in batch.
#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub skill: String,
    pub sources: Vec<String>,
    pub installed: bool,
}
