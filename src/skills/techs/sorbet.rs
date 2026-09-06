use crate::skills::types::{DetectConfig, Technology};

pub const SORBET_TECH: Technology = Technology {
    id: "sorbet",
    name: "Sorbet",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["sorbet/config"],
        file_extensions: &[],
        gems: &["sorbet", "sorbet-runtime"],
        config_file_content: &[],
    },
    skills: &[
        "DmitryPogrebnoy/ruby-agent-skills/generating-sorbet",
        "DmitryPogrebnoy/ruby-agent-skills/generating-sorbet-inline",
    ],
};
