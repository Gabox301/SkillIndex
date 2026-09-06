use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u64,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    pub reviewer: Reviewer,
    pub skills: HashMap<String, RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reviewer {
    pub model: String,
    #[serde(rename = "promptVersion")]
    pub prompt_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub source: String,
    #[serde(rename = "skillPath")]
    pub skill_path: String,
    #[serde(rename = "commitSha")]
    pub commit_sha: String,
    pub files: Vec<String>,
    pub sha256: HashMap<String, String>,
    #[serde(rename = "bundleHash")]
    pub bundle_hash: String,
    pub review: Review,
    #[serde(rename = "securityCheck")]
    pub security_check: Option<SecurityCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub status: String,
    pub flags: Vec<String>,
    pub summary: String,
    pub model: String,
    #[serde(rename = "promptVersion")]
    pub prompt_version: String,
    #[serde(rename = "reviewedAt")]
    pub reviewed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheck {
    pub status: String,
    pub findings: Vec<String>,
    pub summary: String,
    #[serde(rename = "checkedAt")]
    pub checked_at: String,
}

#[derive(Debug, Clone)]
pub struct InstallSecurityCheck {
    pub name: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub ok: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedSkillPath {
    pub repo: String,
    pub skill_name: String,
    pub full: String,
}
