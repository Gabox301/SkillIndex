use crate::skills::types::{DetectConfig, Technology};

pub const PLAYWRIGHT_TECH: Technology = Technology {
    id: "playwright",
    name: "Playwright",
    detect: DetectConfig {
        packages: &["@playwright/test", "playwright"],
        package_patterns: &[],
        config_files: &["playwright.config.ts", "playwright.config.js"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["currents-dev/playwright-best-practices-skill/playwright-best-practices"],
};
