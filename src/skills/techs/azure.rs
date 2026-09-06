use crate::skills::types::{DetectConfig, Technology};

pub const AZURE_TECH: Technology = Technology {
    id: "azure",
    name: "Azure",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &["^@azure\\/"],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "microsoft/github-copilot-for-azure/azure-deploy",
        "microsoft/github-copilot-for-azure/azure-ai",
        "microsoft/github-copilot-for-azure/azure-cost",
        "microsoft/github-copilot-for-azure/azure-diagnostics",
    ],
};
