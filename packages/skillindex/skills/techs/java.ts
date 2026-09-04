export const javaTech = {
  id: 'java',
  name: 'Java',
  detect: {
    configFiles: ['pom.xml'],
    configFileContent: {
      scanGradleLayout: true,
      patterns: [
        'sourceCompatibility',
        'targetCompatibility',
        'JavaVersion',
        'id("java")',
        "id 'java'",
        'id("java-library")',
        "id 'java-library'",
      ],
    },
  },
  skills: ['github/awesome-copilot/java-docs', 'affaan-m/everything-claude-code/java-coding-standards'],
};
