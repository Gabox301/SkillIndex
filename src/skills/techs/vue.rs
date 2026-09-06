use crate::skills::types::{DetectConfig, Technology};

pub const VUE_TECH: Technology = Technology {
    id: "vue",
    name: "Vue",
    detect: DetectConfig {
        packages: &["vue"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "hyf0/vue-skills/vue-debug-guides",
        "antfu/skills/vue",
        "antfu/skills/vue-best-practices",
    ],
};
