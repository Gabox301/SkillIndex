export const securityAuditingTech = {
  id: "security-auditing",
  name: "Security Auditing",
  detect: { "configFileContent": { "files": ["audit-report.md", "security-audit.md"], "patterns": ["audit", "compliance", "iam review"] } },
  skills: [
    "Gabox301/SkillIndex/auditing-azure-active-directory-configuration",
    "Gabox301/SkillIndex/auditing-cloud-with-cis-benchmarks",
    "Gabox301/SkillIndex/auditing-entra-id-with-aadinternals",
    "Gabox301/SkillIndex/auditing-foundry-smart-contract-security",
    "Gabox301/SkillIndex/auditing-gcp-iam-permissions",
    "Gabox301/SkillIndex/auditing-kubernetes-cluster-rbac",
    "Gabox301/SkillIndex/auditing-kubernetes-rbac-privilege-escalation",
    "Gabox301/SkillIndex/auditing-mcp-servers-for-tool-poisoning",
    "Gabox301/SkillIndex/auditing-terraform-infrastructure-for-security",
    "Gabox301/SkillIndex/auditing-tls-certificate-transparency-logs",
    "Gabox301/SkillIndex/auditing-uefi-firmware-with-chipsec",
  ],
};
