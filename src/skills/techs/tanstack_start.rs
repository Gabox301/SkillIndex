use crate::skills::types::{DetectConfig, Technology};

pub const TANSTACK_START_TECH: Technology = Technology {
    id: "tanstack-start",
    name: "TanStack Start",
    detect: DetectConfig {
        packages: &["@tanstack/react-start", "@tanstack/start"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["tanstack-skills/tanstack-skills/tanstack-start"],
};
