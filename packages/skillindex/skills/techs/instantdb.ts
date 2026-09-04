export const instantdbTech = {
  id: 'instantdb',
  name: 'InstantDB',
  detect: {
    packages: [
      '@instantdb/core',
      '@instantdb/react',
      '@instantdb/react-native',
      '@instantdb/react-native-mmkv',
      '@instantdb/admin',
    ],
    configFiles: ['instant.schema.ts', 'instant.perms.ts'],
  },
  skills: ['instantdb/skills/instantdb'],
};
