export const evidenceExtractionTech = {
  id: "evidence-extraction",
  name: "Evidence Extraction",
  detect: { "configFileContent": { "files": ["dfir-evidence.md", "forensic-acquisition.md"], "patterns": ["forensic", "evidence", "acquisition"] } },
  skills: [
    "Gabox301/SkillIndex/extracting-browser-history-artifacts",
    "Gabox301/SkillIndex/extracting-config-from-agent-tesla-rat",
    "Gabox301/SkillIndex/extracting-credentials-from-memory-dump",
    "Gabox301/SkillIndex/extracting-iocs-from-malware-samples",
    "Gabox301/SkillIndex/extracting-memory-artifacts-with-rekall",
    "Gabox301/SkillIndex/extracting-windows-event-logs-artifacts",
  ],
};
