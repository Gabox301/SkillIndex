use crate::skills::types::{DetectConfig, Technology};

pub const BOOK_TO_SKILL: Technology = Technology {
    id: "book-to-skill",
    name: "Book to Skill",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["virgiliojr94/book-to-skill/book-to-skill"],
};
