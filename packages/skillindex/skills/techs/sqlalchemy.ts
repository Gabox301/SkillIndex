export const sqlalchemyTech = {
  id: "sqlalchemy",
  name: "SQLAlchemy",
  detect: { "configFileContent": { "files": ["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"], "patterns": ["sqlalchemy", "SQLAlchemy"] } },
  skills: [
    "bobmatnyc/claude-mpm-skills/sqlalchemy",
    "wispbit-ai/skills/sqlalchemy-alembic-expert-best-practices-code-review",
  ],
};
