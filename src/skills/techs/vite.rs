use crate::skills::types::{DetectConfig, Technology};

pub const VITE_TECH: Technology = Technology {
    id: "vite",
    name: "Vite",
    detect: DetectConfig {
        packages: &["vite"],
        package_patterns: &[],
        config_files: &["vite.config.js", "vite.config.ts", "vite.config.mjs"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["antfu/skills/vite"],
};
