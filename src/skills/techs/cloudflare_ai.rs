use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const CLOUDFLARE_AI_TECH: Technology = Technology {
    id: "cloudflare-ai",
    name: "Cloudflare AI",
    detect: DetectConfig {
        packages: &["@cloudflare/ai"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["wrangler.json", "wrangler.jsonc"],
            patterns: &["\"ai\""],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["cloudflare/skills/agents-sdk"],
};
