use crate::skills::types::{DetectConfig, Technology};

pub const PINIA_TECH: Technology = Technology {
    id: "pinia",
    name: "Pinia",
    detect: DetectConfig {
        packages: &["pinia"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["vuejs-ai/skills/vue-pinia-best-practices"],
};
