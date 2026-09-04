export const activeDirectoryTech = {
  id: 'active-directory',
  name: 'Active Directory',
  detect: {
    configFileContent: {
      files: ['ad.conf', 'ldap.conf', 'krb5.conf'],
      patterns: ['active-directory', 'ldap', 'kerberos', 'domain controller'],
    },
  },
  skills: [
    'Gabox301/SkillIndex/performing-active-directory-bloodhound-analysis',
    'Gabox301/SkillIndex/performing-active-directory-compromise-investigation',
    'Gabox301/SkillIndex/performing-active-directory-forest-trust-attack',
    'Gabox301/SkillIndex/performing-active-directory-penetration-test',
    'Gabox301/SkillIndex/performing-active-directory-vulnerability-assessment',
  ],
};
