use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const INCIDENT_TRIAGE_TECH: Technology = Technology {
    id: "incident-triage",
    name: "Incident Triage",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["incident-triage.md", "ir-playbook.md"],
            patterns: &["incident", "triage", "playbook"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "Gabox301/SkillIndex/triaging-security-alerts-in-splunk",
        "Gabox301/SkillIndex/triaging-security-incident",
        "Gabox301/SkillIndex/triaging-security-incident-with-ir-playbook",
        "Gabox301/SkillIndex/triaging-vulnerabilities-with-ssvc-framework",
        "Gabox301/SkillIndex/triaging-windows-with-kape",
    ],
};
