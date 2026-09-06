use crate::skills::types::{DetectConfig, Technology};

pub const VERCEL_AI_TECH: Technology = Technology {
    id: "vercel-ai",
    name: "Vercel AI SDK",
    detect: DetectConfig {
        packages: &[
            "ai",
            "@ai-sdk/openai",
            "@ai-sdk/anthropic",
            "@ai-sdk/google",
        ],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["vercel/ai/use-ai-sdk"],
};
