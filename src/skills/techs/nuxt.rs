use crate::skills::types::{DetectConfig, Technology};

pub const NUXT_TECH: Technology = Technology {
    id: "nuxt",
    name: "Nuxt",
    detect: DetectConfig {
        packages: &["nuxt"],
        package_patterns: &[],
        config_files: &["nuxt.config.js", "nuxt.config.ts"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["antfu/skills/nuxt"],
};
