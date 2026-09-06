use crate::skills::types::{DetectConfig, Technology};

pub const RUST_TECH: Technology = Technology {
    id: "rust",
    name: "Rust",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["Cargo.toml"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["apollographql/skills/rust-best-practices"],
};
