use crate::skills::types::{DetectConfig, Technology};

pub const ARCHIFY: Technology = Technology {
    id: "archify",
    name: "Archify",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["tt-a1i/archify/archify"],
};
