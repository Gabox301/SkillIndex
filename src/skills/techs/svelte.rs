use crate::skills::types::{DetectConfig, Technology};

pub const SVELTE_TECH: Technology = Technology {
    id: "svelte",
    name: "Svelte",
    detect: DetectConfig {
        packages: &["svelte", "@sveltejs/kit"],
        package_patterns: &[],
        config_files: &["svelte.config.js"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "ejirocodes/agent-skills/svelte5-best-practices",
        "sveltejs/ai-tools/svelte-code-writer",
    ],
};
