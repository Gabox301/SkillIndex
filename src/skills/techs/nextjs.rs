use crate::skills::types::{DetectConfig, Technology};

pub const NEXTJS_TECH: Technology = Technology {
    id: "nextjs",
    name: "Next.js",
    detect: DetectConfig {
        packages: &["next"],
        package_patterns: &[],
        config_files: &["next.config.js", "next.config.mjs", "next.config.ts"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "vercel-labs/next-skills/next-best-practices",
        "vercel-labs/next-skills/next-cache-components",
        "vercel-labs/next-skills/next-upgrade",
    ],
};
