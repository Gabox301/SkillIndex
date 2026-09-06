use crate::skills::types::{DetectConfig, Technology};

pub const BETTER_AUTH_TECH: Technology = Technology {
    id: "better-auth",
    name: "Better Auth",
    detect: DetectConfig {
        packages: &["better-auth"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "better-auth/skills/best-practices",
        "better-auth/skills/emailAndPassword",
        "better-auth/skills/organization",
        "better-auth/skills/twoFactor",
    ],
};
