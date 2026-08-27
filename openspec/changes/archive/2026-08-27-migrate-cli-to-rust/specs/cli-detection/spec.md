# cli-detection Specification

## Purpose
Detection `detectTechnologies`/`resolveWorkspaces`/scanners identical to `lib.ts`.

## Requirements

### Requirement: Tech and Workspace Detection
MUST evaluate `SKILLS_MAP` signals with caches and MUST resolve workspaces `pnpm-workspace.yaml > package.json > deno.json`, expanding `*` via `readdir`, skipping lacking `package.json|deno.json(c)`.

#### Scenario: Package+config hit
- GIVEN `package.json` has `react` and `next.config.mjs` exists
- WHEN `detectTechnologies` runs
- THEN returns `react`/`nextjs`

#### Scenario: pnpm wins
- GIVEN both `pnpm-workspace.yaml` and `package.json` workspaces
- WHEN `resolveWorkspaces` runs
- THEN parses pnpm line-wise, ignores npm

#### Scenario: Deno fallback
- GIVEN only `deno.json` `{workspace:["packages/*"]}`
- WHEN resolving
- THEN expands, filters `resolve(d)!==resolve(root)`

### Requirement: Boundaries and Frontend
MUST skip `SCAN_SKIP_DIRS` (`node_modules,.git,vendor,.next,dist,build,.output,.nuxt,.svelte-kit,__pycache__,.cache,coverage,.turbo,.terraform,var,bin,obj,.vs`) and MUST compute `isFrontend = FRONTEND_PACKAGES || hasWebFrontendFiles depth3`.

#### Scenario: Skip not descended
- GIVEN `node_modules/x/App.vue` exists
- WHEN scanning
- THEN no detection from that subtree

#### Scenario: File frontend
- GIVEN no frontend pkg but `src/App.vue` depth≤3
- WHEN detection runs
- THEN `isFrontend` true

### Requirement: Extended Parsers
MUST port verbatim `parseSettingsGradleModules` (`/include\s*\(?\s*([^)]+)/`), cached `gradleLayoutCandidatePaths`/`dotNetLayoutCandidatePaths` depth-2 `.sln/.csproj/.fsproj`, and `resolveConfigFileContentPaths` substring `patterns`.

#### Scenario: Gradle include
- GIVEN `include(":app",":lib:core")`
- WHEN parsing
- THEN `["app","lib/core"]`

#### Scenario: .NET depth
- GIVEN `src/A/B/C/App.csproj` depth3
- WHEN scanning
- THEN excluded
