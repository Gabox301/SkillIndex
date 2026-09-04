export const laravelTech = {
  id: 'laravel',
  name: 'Laravel',
  detect: {
    configFiles: ['artisan', 'bootstrap/app.php'],
    configFileContent: { files: ['composer.json'], patterns: ['"laravel/framework"', '"illuminate/'] },
  },
  skills: ['jeffallan/claude-skills/laravel-specialist', 'affaan-m/everything-claude-code/laravel-patterns'],
};
