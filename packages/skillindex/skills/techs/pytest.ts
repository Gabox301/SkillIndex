export const pytestTech = {
  id: "pytest",
  name: "Pytest",
  detect: { "configFileContent": { "files": ["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"], "patterns": ["pytest", "Pytest"] } },
  skills: [
    "wshobson/agents/python-testing-patterns",
  ],
};
