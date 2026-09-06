use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const EVIDENCE_EXTRACTION_TECH: Technology = Technology {
    id: "evidence-extraction",
    name: "Evidence Extraction",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["dfir-evidence.md", "forensic-acquisition.md"],
            patterns: &["forensic", "evidence", "acquisition"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "Gabox301/SkillIndex/extracting-browser-history-artifacts",
        "Gabox301/SkillIndex/extracting-config-from-agent-tesla-rat",
        "Gabox301/SkillIndex/extracting-credentials-from-memory-dump",
        "Gabox301/SkillIndex/extracting-iocs-from-malware-samples",
        "Gabox301/SkillIndex/extracting-memory-artifacts-with-rekall",
        "Gabox301/SkillIndex/extracting-windows-event-logs-artifacts",
    ],
};
