use crate::skills::types::{DetectConfig, Technology};

pub const DESIGN_DNA: Technology = Technology {
    id: "design-dna",
    name: "Design DNA",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["zanwei/design-dna/design-dna"],
};
