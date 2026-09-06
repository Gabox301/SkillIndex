use crate::skills::types::{DetectConfig, Technology};

pub const NODE_TECH: Technology = Technology {
    id: "node",
    name: "Node.js",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            ".nvmrc",
            ".node-version",
        ],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "wshobson/agents/nodejs-backend-patterns",
        "sickn33/antigravity-awesome-skills/nodejs-best-practices",
    ],
};
