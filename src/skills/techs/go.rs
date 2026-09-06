use crate::skills::types::{DetectConfig, Technology};

pub const GO_TECH: Technology = Technology {
    id: "go",
    name: "Go",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["go.mod", "go.work"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "affaan-m/everything-claude-code/golang-patterns",
        "affaan-m/everything-claude-code/golang-testing",
    ],
};
