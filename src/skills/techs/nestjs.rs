use crate::skills::types::{DetectConfig, Technology};

pub const NESTJS_TECH: Technology = Technology {
    id: "nestjs",
    name: "NestJS",
    detect: DetectConfig {
        packages: &["@nestjs/core"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["kadajett/agent-nestjs-skills/nestjs-best-practices"],
};
