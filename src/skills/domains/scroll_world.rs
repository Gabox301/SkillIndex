use crate::skills::types::{DetectConfig, Technology};

/// Design domain skill: scroll-scrubbed landing pages. It ships a vanilla JS
/// scrub engine, so it has no framework or package signal to detect on and is
/// only offered through an explicit domain selection (`--domain scroll-world`).
pub const SCROLL_WORLD: Technology = Technology {
    id: "scroll-world",
    name: "Scroll World",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/scroll-world"],
};
