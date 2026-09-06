use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const RED_TEAM_EXECUTION_TECH: Technology = Technology {
    id: "red-team-execution",
    name: "Red Team Execution",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["red-team-plan.md", "engagement-plan.json"],
            patterns: &["red team", "adversary simulation", "engagement"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "Gabox301/SkillIndex/executing-active-directory-attack-simulation",
        "Gabox301/SkillIndex/executing-nist-rmf-authorization-to-operate",
        "Gabox301/SkillIndex/executing-phishing-simulation-campaign",
        "Gabox301/SkillIndex/executing-red-team-engagement-planning",
        "Gabox301/SkillIndex/executing-red-team-exercise",
    ],
};
