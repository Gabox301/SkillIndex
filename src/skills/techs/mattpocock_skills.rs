use crate::skills::types::{DetectConfig, Technology};

pub const MATTPOCOCK_SKILLS_TECH: Technology = Technology {
    id: "mattpocock-skills",
    name: "Matt Pocock Skills",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "mattpocock/skills/setup-matt-pocock-skills",
        "mattpocock/skills/grilling",
        "mattpocock/skills/teach",
        "mattpocock/skills/grill-me",
        "mattpocock/skills/grill-with-docs",
        "mattpocock/skills/improve-codebase-architecture",
        "mattpocock/skills/tdd",
    ],
};
