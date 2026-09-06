use crate::skills::types::{DetectConfig, Technology};

pub const DRIZZLE_TECH: Technology = Technology {
    id: "drizzle",
    name: "Drizzle ORM",
    detect: DetectConfig {
        packages: &["drizzle-orm", "drizzle-kit"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["bobmatnyc/claude-mpm-skills/drizzle"],
};
