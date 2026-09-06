use crate::skills::types::{DetectConfig, Technology};

pub const ASTRO_TECH: Technology = Technology {
    id: "astro",
    name: "Astro",
    detect: DetectConfig {
        packages: &["astro"],
        package_patterns: &[],
        config_files: &["astro.config.mjs", "astro.config.js", "astro.config.ts"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["astrolicious/agent-skills/astro"],
};
