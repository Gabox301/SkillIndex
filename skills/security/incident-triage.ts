export const incidentTriageTech = {
  id: 'incident-triage',
  name: 'Incident Triage',
  detect: {
    configFileContent: {
      files: ['incident-triage.md', 'ir-playbook.md'],
      patterns: ['incident', 'triage', 'playbook'],
    },
  },
  skills: [
    'Gabox301/SkillIndex/triaging-security-alerts-in-splunk',
    'Gabox301/SkillIndex/triaging-security-incident',
    'Gabox301/SkillIndex/triaging-security-incident-with-ir-playbook',
    'Gabox301/SkillIndex/triaging-vulnerabilities-with-ssvc-framework',
    'Gabox301/SkillIndex/triaging-windows-with-kape',
  ],
};
