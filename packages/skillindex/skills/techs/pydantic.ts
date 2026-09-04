export const pydanticTech = {
  id: "pydantic",
  name: "Pydantic",
  detect: { "configFileContent": { "files": ["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"], "patterns": ["pydantic", "Pydantic"] } },
  skills: [
    "bobmatnyc/claude-mpm-skills/pydantic",
  ],
};
