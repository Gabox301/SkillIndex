use crate::skills::types::{DetectConfig, Technology};

pub const CLOUDFLARE_AGENTS_TECH: Technology = Technology {
    id: "cloudflare-agents",
    name: "Cloudflare Agents",
    detect: DetectConfig {
        packages: &["agents"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "cloudflare/skills/agents-sdk",
        "Gabox301/SkillIndex/sandbox-sdk",
    ],
};
