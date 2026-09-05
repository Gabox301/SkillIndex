export const celeryTech = {
  id: 'celery',
  name: 'Celery',
  detect: {
    configFileContent: {
      files: ['pyproject.toml', 'requirements.txt', 'setup.py', 'Pipfile'],
      patterns: ['celery', 'Celery'],
    },
  },
  skills: ['wshobson/agents/python-background-jobs'],
};
