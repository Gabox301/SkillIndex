use crate::skills::types::{DetectConfig, Technology};

/// Design domain skill: no package or file signal identifies it, so it is only
/// offered through an explicit domain selection (`--domain landing-page-design`).
pub const LANDING_PAGE_DESIGN: Technology = Technology {
    id: "landing-page-design",
    name: "Landing Page Design",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/landing-page-design"],
};
