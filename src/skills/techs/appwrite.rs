use crate::skills::types::{DetectConfig, Technology};

pub const APPWRITE_TECH: Technology = Technology {
    id: "appwrite",
    name: "Appwrite",
    detect: DetectConfig {
        packages: &["appwrite", "node-appwrite", "@appwrite.io/console"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "Gabox301/SkillIndex/appwrite-cli",
        "Gabox301/SkillIndex/appwrite-go",
        "Gabox301/SkillIndex/appwrite-python",
        "Gabox301/SkillIndex/appwrite-ruby",
        "Gabox301/SkillIndex/appwrite-rust",
        "Gabox301/SkillIndex/appwrite-typescript",
    ],
};
