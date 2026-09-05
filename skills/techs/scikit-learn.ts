export const scikitLearnTech = {
  id: 'scikit-learn',
  name: 'Scikit-Learn',
  detect: {
    configFileContent: {
      files: ['pyproject.toml', 'requirements.txt', 'setup.py', 'Pipfile'],
      patterns: ['scikit-learn', 'scikit_learn', 'sklearn'],
    },
  },
  skills: ['davila7/claude-code-templates/scikit-learn', 'davila7/claude-code-templates/senior-data-scientist'],
};
