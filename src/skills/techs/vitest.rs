use crate::skills::types::{DetectConfig, Technology};

pub const VITEST_TECH: Technology = Technology {
    id: "vitest",
    name: "Vitest",
    detect: DetectConfig {
        packages: &["vitest"],
        package_patterns: &[],
        config_files: &["vitest.config.ts", "vitest.config.js", "vitest.config.mts"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["antfu/skills/vitest"],
};
