use crate::skills::types::{DetectConfig, Technology};

pub const ANYDOC: Technology = Technology {
    id: "anydoc",
    name: "AnyDoc Document Conversion",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["firecrawl/anydoc/convert-documents-to-markdown"],
};
