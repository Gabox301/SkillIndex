use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const ACTIVE_DIRECTORY_TECH: Technology = Technology {
    id: "active-directory",
    name: "Active Directory",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["ad.conf", "ldap.conf", "krb5.conf"],
            patterns: &["active-directory", "ldap", "kerberos", "domain controller"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "Gabox301/SkillIndex/performing-active-directory-bloodhound-analysis",
        "Gabox301/SkillIndex/performing-active-directory-compromise-investigation",
        "Gabox301/SkillIndex/performing-active-directory-forest-trust-attack",
        "Gabox301/SkillIndex/performing-active-directory-penetration-test",
        "Gabox301/SkillIndex/performing-active-directory-vulnerability-assessment",
    ],
};
