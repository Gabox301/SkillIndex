export const springbootTech = {
  id: 'springboot',
  name: 'Spring Boot',
  detect: {
    configFiles: [
      'src/main/resources/application.properties',
      'src/main/resources/application.yml',
      'src/main/resources/application.yaml',
    ],
    configFileContent: { files: ['pom.xml'], patterns: ['spring-boot-starter', 'org.springframework.boot'] },
  },
  skills: ['github/awesome-copilot/java-springboot'],
};
