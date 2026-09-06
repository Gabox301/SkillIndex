use crate::skills::types::{DetectConfig, Technology};

pub const VERCEL_DEPLOY_TECH: Technology = Technology {
    id: "vercel-deploy",
    name: "Vercel",
    detect: DetectConfig {
        packages: &["vercel", "@astrojs/vercel"],
        package_patterns: &[],
        config_files: &["vercel.json", ".vercel"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["vercel-labs/agent-skills/deploy-to-vercel"],
};
