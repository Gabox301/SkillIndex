use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const SECURITY_DEPLOYMENT_TECH: Technology = Technology {
    id: "security-deployment",
    name: "Security Deployment",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["deployment-plan-security.md", "zero-trust-deployment.md"],
            patterns: &["zero trust", "honeytoken", "canary"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "Gabox301/SkillIndex/deploying-active-directory-honeytokens",
        "Gabox301/SkillIndex/deploying-cloud-deception-with-decoy-resources",
        "Gabox301/SkillIndex/deploying-cloudflare-access-for-zero-trust",
        "Gabox301/SkillIndex/deploying-decoy-files-for-ransomware-detection",
        "Gabox301/SkillIndex/deploying-edr-agent-with-crowdstrike",
        "Gabox301/SkillIndex/deploying-honeytokens-and-canarytokens",
        "Gabox301/SkillIndex/deploying-osquery-for-endpoint-monitoring",
        "Gabox301/SkillIndex/deploying-palo-alto-prisma-access-zero-trust",
        "Gabox301/SkillIndex/deploying-ransomware-canary-files",
        "Gabox301/SkillIndex/deploying-software-defined-perimeter",
        "Gabox301/SkillIndex/deploying-tailscale-for-zero-trust-vpn",
    ],
};
