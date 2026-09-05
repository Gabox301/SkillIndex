export const numpyTech = {
  id: 'numpy',
  name: 'NumPy',
  detect: {
    configFileContent: {
      files: ['pyproject.toml', 'requirements.txt', 'setup.py', 'Pipfile'],
      patterns: ['numpy', 'NumPy', 'numpy'],
    },
  },
  skills: [
    'pluginagentmarketplace/custom-plugin-python/machine-learning',
    'pluginagentmarketplace/custom-plugin-python/pandas-data-analysis',
  ],
};
