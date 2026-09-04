import type { ComboSkill, Technology } from './types.ts';

import { cloudSecurityCombo } from './combos/cloud-security.ts';
import { forensicsIrCombo } from './combos/forensics-ir.ts';
import { FRAMEWORK_COMBOS } from './combos/framework-combos.ts';
import { redTeamCombo } from './combos/red-team.ts';
import { securityOperationsCombo } from './combos/security-operations.ts';
import { agentWorkflows } from './domains/agent-workflows.ts';
import { archify } from './domains/archify.ts';
import { mattPocockSkills } from './domains/matt-pocock.ts';
import { sddWorkflow } from './domains/sdd-workflow.ts';
import { tasteSkill } from './domains/taste-skill.ts';
import { activeDirectoryTech } from './security/active-directory.ts';
import { evidenceExtractionTech } from './security/evidence-extraction.ts';
import { exploitationTech } from './security/exploitation.ts';
import { incidentTriageTech } from './security/incident-triage.ts';
import { penetrationTestingTech } from './security/penetration-testing.ts';
import { redTeamExecutionTech } from './security/red-team-execution.ts';
import { reverseEngineeringTech } from './security/reverse-engineering.ts';
import { securityAnalysisTech } from './security/security-analysis.ts';
import { securityAssessmentsTech } from './security/security-assessments.ts';
import { securityAuditingTech } from './security/security-auditing.ts';
import { securityConfigurationTech } from './security/security-configuration.ts';
import { securityDeploymentTech } from './security/security-deployment.ts';
import { securityEngineeringTech } from './security/security-engineering.ts';
import { securityHardeningTech } from './security/security-hardening.ts';
import { securityImplementationTech } from './security/security-implementation.ts';
import { securityTestingTech } from './security/security-testing.ts';
import { securityToolingTech } from './security/security-tooling.ts';
import { threatDetectionTech } from './security/threat-detection.ts';
import { threatHuntingTech } from './security/threat-hunting.ts';
import { vulnerabilityScanningTech } from './security/vulnerability-scanning.ts';
import { activeadminTech } from './techs/activeadmin.ts';
import { androidTech } from './techs/android.ts';
import { angularTech } from './techs/angular.ts';
import { appwriteTech } from './techs/appwrite.ts';
import { aspnetBlazorTech } from './techs/aspnet-blazor.ts';
import { aspnetMinimalApiTech } from './techs/aspnet-minimal-api.ts';
import { aspnetcoreTech } from './techs/aspnetcore.ts';
import { astroTech } from './techs/astro.ts';
import { awsTech } from './techs/aws.ts';
import { azureTech } from './techs/azure.ts';
import { bashTech } from './techs/bash.ts';
import { betterAuthTech } from './techs/better-auth.ts';
import { bunTech } from './techs/bun.ts';
import { celeryTech } from './techs/celery.ts';
import { chromeExtensionTech } from './techs/chrome-extension.ts';
import { clerkTech } from './techs/clerk.ts';
import { cloudflareAgentsTech } from './techs/cloudflare-agents.ts';
import { cloudflareAiTech } from './techs/cloudflare-ai.ts';
import { cloudflareDurableObjectsTech } from './techs/cloudflare-durable-objects.ts';
import { cloudflareTech } from './techs/cloudflare.ts';
import { csharpTech } from './techs/csharp.ts';
import { dartTech } from './techs/dart.ts';
import { denoTech } from './techs/deno.ts';
import { deviseTech } from './techs/devise.ts';
import { djangoTech } from './techs/django.ts';
import { dotnetTech } from './techs/dotnet.ts';
import { drizzleTech } from './techs/drizzle.ts';
import { electronTech } from './techs/electron.ts';
import { elevenlabsTech } from './techs/elevenlabs.ts';
import { elysiaTech } from './techs/elysia.ts';
import { expoTech } from './techs/expo.ts';
import { expressTech } from './techs/express.ts';
import { fastapiTech } from './techs/fastapi.ts';
import { fastmcpTech } from './techs/fastmcp.ts';
import { flaskTech } from './techs/flask.ts';
import { flutterTech } from './techs/flutter.ts';
import { goTech } from './techs/go.ts';
import { gsapTech } from './techs/gsap.ts';
import { honoTech } from './techs/hono.ts';
import { instantdbTech } from './techs/instantdb.ts';
import { javaTech } from './techs/java.ts';
import { kotlinMultiplatformTech } from './techs/kotlin-multiplatform.ts';
import { laravelTech } from './techs/laravel.ts';
import { neonTech } from './techs/neon.ts';
import { nestjsTech } from './techs/nestjs.ts';
import { nextjsTech } from './techs/nextjs.ts';
import { nodeTech } from './techs/node.ts';
import { numpyTech } from './techs/numpy.ts';
import { nuxtTech } from './techs/nuxt.ts';
import { oxlintTech } from './techs/oxlint.ts';
import { pandasTech } from './techs/pandas.ts';
import { phpTech } from './techs/php.ts';
import { piniaTech } from './techs/pinia.ts';
import { playwrightTech } from './techs/playwright.ts';
import { postgresRubyTech } from './techs/postgres-ruby.ts';
import { prismaTech } from './techs/prisma.ts';
import { pydanticTech } from './techs/pydantic.ts';
import { pytestTech } from './techs/pytest.ts';
import { pythonTech } from './techs/python.ts';
import { railsTech } from './techs/rails.ts';
import { reactHookFormTech } from './techs/react-hook-form.ts';
import { reactNativeTech } from './techs/react-native.ts';
import { reactRouterTech } from './techs/react-router.ts';
import { reactThreeFiberTech } from './techs/react-three-fiber.ts';
import { reactTech } from './techs/react.ts';
import { redisRubyTech } from './techs/redis-ruby.ts';
import { remotionTech } from './techs/remotion.ts';
import { requestsTech } from './techs/requests.ts';
import { rspecTech } from './techs/rspec.ts';
import { rubocopTech } from './techs/rubocop.ts';
import { rubyTech } from './techs/ruby.ts';
import { rustTech } from './techs/rust.ts';
import { scikitLearnTech } from './techs/scikit-learn.ts';
import { shadcnTech } from './techs/shadcn.ts';
import { sidekiqTech } from './techs/sidekiq.ts';
import { sorbetTech } from './techs/sorbet.ts';
import { springbootTech } from './techs/springboot.ts';
import { sqlalchemyTech } from './techs/sqlalchemy.ts';
import { stripeTech } from './techs/stripe.ts';
import { supabaseTech } from './techs/supabase.ts';
import { svelteTech } from './techs/svelte.ts';
import { swiftuiTech } from './techs/swiftui.ts';
import { tailwindTech } from './techs/tailwind.ts';
import { tanstackStartTech } from './techs/tanstack-start.ts';
import { tauriTech } from './techs/tauri.ts';
import { terraformTech } from './techs/terraform.ts';
import { threejsTech } from './techs/threejs.ts';
import { turborepoTech } from './techs/turborepo.ts';
import { typescriptTech } from './techs/typescript.ts';
import { vercelAiTech } from './techs/vercel-ai.ts';
import { vercelDeployTech } from './techs/vercel-deploy.ts';
import { viteTech } from './techs/vite.ts';
import { vitestTech } from './techs/vitest.ts';
import { vueTech } from './techs/vue.ts';
import { wordpressTech } from './techs/wordpress.ts';
import { zodTech } from './techs/zod.ts';

export const SKILLS_MAP: Technology[] = [
  reactTech,
  nextjsTech,
  vueTech,
  nuxtTech,
  piniaTech,
  svelteTech,
  angularTech,
  astroTech,
  tailwindTech,
  shadcnTech,
  typescriptTech,
  reactHookFormTech,
  zodTech,
  supabaseTech,
  neonTech,
  instantdbTech,
  playwrightTech,
  expoTech,
  reactNativeTech,
  dartTech,
  flutterTech,
  kotlinMultiplatformTech,
  androidTech,
  remotionTech,
  reactRouterTech,
  tanstackStartTech,
  chromeExtensionTech,
  clerkTech,
  betterAuthTech,
  turborepoTech,
  viteTech,
  azureTech,
  vercelAiTech,
  elevenlabsTech,
  vercelDeployTech,
  cloudflareTech,
  cloudflareDurableObjectsTech,
  cloudflareAgentsTech,
  cloudflareAiTech,
  terraformTech,
  awsTech,
  activeDirectoryTech,
  threatDetectionTech,
  securityImplementationTech,
  securityAnalysisTech,
  threatHuntingTech,
  swiftuiTech,
  oxlintTech,
  gsapTech,
  threejsTech,
  reactThreeFiberTech,
  bunTech,
  elysiaTech,
  nodeTech,
  bashTech,
  expressTech,
  goTech,
  denoTech,
  wordpressTech,
  javaTech,
  springbootTech,
  prismaTech,
  stripeTech,
  honoTech,
  vitestTech,
  drizzleTech,
  nestjsTech,
  tauriTech,
  electronTech,
  dotnetTech,
  csharpTech,
  aspnetcoreTech,
  aspnetBlazorTech,
  aspnetMinimalApiTech,
  rustTech,
  rubyTech,
  railsTech,
  redisRubyTech,
  postgresRubyTech,
  sorbetTech,
  activeadminTech,
  deviseTech,
  sidekiqTech,
  rspecTech,
  rubocopTech,
  phpTech,
  laravelTech,
  pythonTech,
  fastapiTech,
  fastmcpTech,
  djangoTech,
  flaskTech,
  pydanticTech,
  sqlalchemyTech,
  pytestTech,
  pandasTech,
  numpyTech,
  scikitLearnTech,
  celeryTech,
  requestsTech,
  securityAssessmentsTech,
  securityEngineeringTech,
  exploitationTech,
  securityTestingTech,
  penetrationTestingTech,
  securityConfigurationTech,
  securityHardeningTech,
  securityAuditingTech,
  securityDeploymentTech,
  sddWorkflow,
  vulnerabilityScanningTech,
  appwriteTech,
  evidenceExtractionTech,
  reverseEngineeringTech,
  redTeamExecutionTech,
  incidentTriageTech,
  securityToolingTech,
  agentWorkflows,
  mattPocockSkills,
  tasteSkill,
  archify,
];

export const COMBO_SKILLS_MAP: ComboSkill[] = [
  ...FRAMEWORK_COMBOS,
  securityOperationsCombo,
  redTeamCombo,
  cloudSecurityCombo,
  forensicsIrCombo,
];
