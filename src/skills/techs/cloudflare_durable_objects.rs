use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const CLOUDFLARE_DURABLE_OBJECTS_TECH: Technology = Technology {
    id: "cloudflare-durable-objects",
    name: "Durable Objects",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["wrangler.json", "wrangler.jsonc", "wrangler.toml"],
            patterns: &["durable_objects"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["cloudflare/skills/durable-objects"],
};
