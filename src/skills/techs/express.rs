use crate::skills::types::{DetectConfig, Technology};

pub const EXPRESS_TECH: Technology = Technology {
    id: "express",
    name: "Express",
    detect: DetectConfig {
        packages: &["express"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["aj-geddes/useful-ai-prompts/nodejs-express-server"],
};
