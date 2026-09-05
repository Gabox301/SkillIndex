export const securityConfigurationTech = {
  id: 'security-configuration',
  name: 'Security Configuration',
  detect: {
    configFileContent: {
      files: ['security-config.md', 'hardening-guide.md'],
      patterns: ['hardening', 'tls', 'zero trust'],
    },
  },
  skills: [
    'Gabox301/SkillIndex/configuring-active-directory-tiered-model',
    'Gabox301/SkillIndex/configuring-certificate-authority-with-openssl',
    'Gabox301/SkillIndex/configuring-host-based-intrusion-detection',
    'Gabox301/SkillIndex/configuring-hsm-for-key-storage',
    'Gabox301/SkillIndex/configuring-identity-aware-proxy-with-google-iap',
    'Gabox301/SkillIndex/configuring-ldap-security-hardening',
    'Gabox301/SkillIndex/configuring-microsegmentation-for-zero-trust',
    'Gabox301/SkillIndex/configuring-multi-factor-authentication-with-duo',
    'Gabox301/SkillIndex/configuring-network-segmentation-with-vlans',
    'Gabox301/SkillIndex/configuring-oauth2-authorization-flow',
    'Gabox301/SkillIndex/configuring-pfsense-firewall-rules',
    'Gabox301/SkillIndex/configuring-snort-ids-for-intrusion-detection',
    'Gabox301/SkillIndex/configuring-suricata-for-network-monitoring',
    'Gabox301/SkillIndex/configuring-tls-1-3-for-secure-communications',
    'Gabox301/SkillIndex/configuring-windows-defender-advanced-settings',
    'Gabox301/SkillIndex/configuring-windows-event-logging-for-detection',
    'Gabox301/SkillIndex/configuring-zscaler-private-access-for-ztna',
  ],
};
