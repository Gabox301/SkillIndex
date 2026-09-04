export const androidTech = {
  id: "android",
  name: "Android",
  detect: { "configFileContent": { "scanGradleLayout": true, "patterns": ["com.android.application", "com.android.library", "id(\"com.android.application\")", "id(\"com.android.library\")", "com.android.kotlin.multiplatform.library"] } },
  skills: [
    "krutikJain/android-agent-skills/android-kotlin-core",
    "krutikJain/android-agent-skills/android-compose-foundations",
    "krutikJain/android-agent-skills/android-architecture-clean",
    "krutikJain/android-agent-skills/android-di-hilt",
    "krutikJain/android-agent-skills/android-gradle-build-logic",
    "krutikJain/android-agent-skills/android-coroutines-flow",
    "krutikJain/android-agent-skills/android-networking-retrofit-okhttp",
    "krutikJain/android-agent-skills/android-testing-unit",
  ],
};
