export const securityHardeningTech = {
  id: "security-hardening",
  name: "Security Hardening",
  detect: { "configFileContent": { "files": ["hardening-baseline.md", "cis-benchmark.md"], "patterns": ["hardening", "cis benchmark"] } },
  skills: [
    "Gabox301/SkillIndex/securing-agentic-ai-tool-invocation",
    "Gabox301/SkillIndex/securing-azure-with-microsoft-defender",
    "Gabox301/SkillIndex/securing-container-registry-images",
    "Gabox301/SkillIndex/securing-container-registry-with-harbor",
    "Gabox301/SkillIndex/securing-github-actions-workflows",
    "Gabox301/SkillIndex/securing-helm-chart-deployments",
    "Gabox301/SkillIndex/securing-historian-server-in-ot-environment",
    "Gabox301/SkillIndex/securing-kubernetes-on-cloud",
    "Gabox301/SkillIndex/securing-remote-access-to-ot-environment",
    "Gabox301/SkillIndex/securing-serverless-functions",
  ],
};
