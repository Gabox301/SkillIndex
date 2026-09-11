use crate::skills::types::{DetectConfig, Technology};

pub const DENO_TECH: Technology = Technology {
    id: "deno",
    name: "Deno",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["deno.json", "deno.jsonc", "deno.lock"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "Gabox301/SkillIndex/deno-expert",
        "Gabox301/SkillIndex/deno-guidance",
        "denoland/skills/deno-frontend",
        "denoland/skills/deno-deploy",
        "denoland/skills/deno-sandbox",
        "mindrally/skills/deno-typescript",
    ],
};
