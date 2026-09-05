export const redTeamExecutionTech = {
  id: 'red-team-execution',
  name: 'Red Team Execution',
  detect: {
    configFileContent: {
      files: ['red-team-plan.md', 'engagement-plan.json'],
      patterns: ['red team', 'adversary simulation', 'engagement'],
    },
  },
  skills: [
    'Gabox301/SkillIndex/executing-active-directory-attack-simulation',
    'Gabox301/SkillIndex/executing-nist-rmf-authorization-to-operate',
    'Gabox301/SkillIndex/executing-phishing-simulation-campaign',
    'Gabox301/SkillIndex/executing-red-team-engagement-planning',
    'Gabox301/SkillIndex/executing-red-team-exercise',
  ],
};
