export const wordpressTech = {
  id: "wordpress",
  name: "WordPress",
  detect: { "configFiles": ["wp-config.php", "wp-login.php"], "packagePatterns": [/^@wordpress\//], "configFileContent": { "files": ["composer.json", "style.css"], "patterns": ["johnpbloch/wordpress", "wpackagist", "Theme Name:"] } },
  skills: [
    "wordpress/agent-skills/wp-plugin-development",
    "wordpress/agent-skills/wp-rest-api",
    "wordpress/agent-skills/wp-block-themes",
    "wordpress/agent-skills/wp-block-development",
    "wordpress/agent-skills/wp-performance",
    "wordpress/agent-skills/wordpress-router",
    "wordpress/agent-skills/wp-project-triage",
    "wordpress/agent-skills/wp-wpcli-and-ops",
  ],
};
