use crate::skills::types::{DetectConfig, Technology};

pub const CLOUDFLARE_TECH: Technology = Technology {
    id: "cloudflare",
    name: "Cloudflare",
    detect: DetectConfig {
        packages: &[
            "wrangler",
            "@cloudflare/workers-types",
            "@astrojs/cloudflare",
        ],
        package_patterns: &[],
        config_files: &["wrangler.toml", "wrangler.json", "wrangler.jsonc"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "cloudflare/skills/cloudflare",
        "cloudflare/skills/wrangler",
        "cloudflare/skills/workers-best-practices",
        "cloudflare/skills/web-perf",
        "openai/skills/cloudflare-deploy",
    ],
};
