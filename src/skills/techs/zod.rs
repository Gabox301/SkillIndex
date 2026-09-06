use crate::skills::types::{DetectConfig, Technology};

pub const ZOD_TECH: Technology = Technology {
    id: "zod",
    name: "Zod",
    detect: DetectConfig {
        packages: &["zod"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["pproenca/dot-skills/zod"],
};
