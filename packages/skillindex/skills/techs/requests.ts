export const requestsTech = {
  id: "requests",
  name: "Requests",
  detect: { "configFileContent": { "files": ["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"], "patterns": ["requests", "Requests"] } },
  skills: [
    "affaan-m/everything-claude-code/python-patterns",
  ],
};
