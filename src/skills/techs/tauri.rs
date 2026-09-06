use crate::skills::types::{DetectConfig, Technology};

pub const TAURI_TECH: Technology = Technology {
    id: "tauri",
    name: "Tauri",
    detect: DetectConfig {
        packages: &["@tauri-apps/api", "@tauri-apps/cli"],
        package_patterns: &[],
        config_files: &["src-tauri/tauri.conf.json"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["nodnarbnitram/claude-code-extensions/tauri-v2"],
};
