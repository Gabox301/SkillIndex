use crate::skills::types::{DetectConfig, Technology};

pub const SHADCN_TECH: Technology = Technology {
    id: "shadcn",
    name: "shadcn/ui",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["components.json"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["shadcn/ui/shadcn"],
};
