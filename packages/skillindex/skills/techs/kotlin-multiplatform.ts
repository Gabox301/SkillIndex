export const kotlinMultiplatformTech = {
  id: 'kotlin-multiplatform',
  name: 'Kotlin Multiplatform',
  detect: {
    configFileContent: {
      scanGradleLayout: true,
      patterns: [
        'kotlin("multiplatform")',
        'org.jetbrains.kotlin.multiplatform',
        'id("org.jetbrains.kotlin.multiplatform")',
        'kotlin-multiplatform',
      ],
    },
  },
  skills: [
    'Kotlin/kotlin-agent-skills/kotlin-tooling-cocoapods-spm-migration',
    'Kotlin/kotlin-agent-skills/kotlin-tooling-agp9-migration',
  ],
};
