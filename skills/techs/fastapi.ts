export const fastapiTech = {
  id: 'fastapi',
  name: 'FastAPI',
  detect: {
    configFileContent: {
      files: ['pyproject.toml', 'requirements.txt', 'setup.py', 'Pipfile'],
      patterns: ['fastapi', 'FastAPI'],
    },
  },
  skills: ['wshobson/agents/fastapi-templates', 'mindrally/skills/fastapi-python'],
};
