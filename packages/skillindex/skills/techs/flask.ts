export const flaskTech = {
  id: "flask",
  name: "Flask",
  detect: { "configFileContent": { "files": ["pyproject.toml", "requirements.txt", "setup.py", "setup.cfg", "Pipfile"], "patterns": ["flask", "Flask"] } },
  skills: [
    "aj-geddes/useful-ai-prompts/flask-api-development",
  ],
};
