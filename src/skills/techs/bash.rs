use crate::skills::types::{DetectConfig, Technology};

pub const BASH_TECH: Technology = Technology {
    id: "bash",
    name: "Bash",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[".sh", ".bash"],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["wshobson/agents/bash-defensive-patterns"],
};
