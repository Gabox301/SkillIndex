# SkillIndex

Detecta e instala automáticamente las mejores skills de agentes IA para tu proyecto. Un comando, cero configuración.

```bash
npx skillindex
```

`skillindex` escanea tu proyecto, detecta las tecnologías que usas e instala skills curadas de [agentes IA](https://skills.sh) que hacen que Cursor, Claude Code y otros asistentes realmente entiendan tu stack.

## Inicio rápido

Ejecútalo en la raíz de tu proyecto:

```bash
npx skillindex
```

Listo. Hará:

1. **Escanear** tu `package.json`, archivos de configuración y estructura del proyecto
2. **Detectar** cada tecnología de tu stack
3. **Mostrar** un selector interactivo con las mejores skills para tu proyecto
4. **Instalar** en paralelo con progreso en vivo
5. **Generar `CLAUDE.md` automáticamente** cuando Claude Code es uno de los agentes destino

### Omitir la confirmación

```bash
npx skillindex -y
```

### Vista previa sin instalar

```bash
npx skillindex --dry-run
```

### Resumen para Claude Code

Si `claude-code` es autodetectado o se pasa con `-a`, `skillindex` escribe un archivo `CLAUDE.md` en la raíz de tu proyecto resumiendo los archivos markdown instalados bajo `.claude/skills`.

## Opciones

| Flag              | Descripción                                                |
| ----------------- | ---------------------------------------------------------- |
| `-y`, `--yes`     | Omite la confirmación, instala todas las skills detectadas |
| `--dry-run`       | Muestra las skills detectadas sin instalar nada            |
| `-v`, `--verbose` | Muestra traza de instalación y detalles de errores         |
| `-h`, `--help`    | Muestra la ayuda                                           |

## Tecnologías soportadas

`skillindex` detecta **más de 50 tecnologías** desde tu `package.json`, lockfiles, archivos Gradle y archivos de configuración:

### Frameworks y librerías

| Tecnología           | Detectado desde                                                                                                                                 |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| React                | paquetes `react`, `react-dom`                                                                                                                   |
| Next.js              | paquete `next` o `next.config.*`                                                                                                                |
| Vue                  | paquete `vue`                                                                                                                                   |
| Nuxt                 | paquete `nuxt` o `nuxt.config.*`                                                                                                                |
| Svelte               | `svelte`, `@sveltejs/kit` o `svelte.config.js`                                                                                                  |
| Angular              | `@angular/core` o `angular.json`                                                                                                                |
| Astro                | paquete `astro` o `astro.config.*`                                                                                                              |
| Expo                 | paquete `expo`                                                                                                                                  |
| React Native         | paquete `react-native`                                                                                                                          |
| Flutter              | archivo `pubspec.yaml` con clave `flutter:`                                                                                                     |
| Kotlin Multiplatform | Gradle con plugin KMP: `kotlin("multiplatform")`, `org.jetbrains.kotlin.multiplatform`, o `kotlin-multiplatform` en `gradle/libs.versions.toml` |
| Android              | Gradle con `com.android.application`, `com.android.library`, o `com.android.kotlin.multiplatform.library`                                       |
| Remotion             | `remotion`, `@remotion/cli`                                                                                                                     |
| GSAP                 | paquete `gsap`                                                                                                                                  |
| Three.js             | `three`, `@react-three/fiber`, `@react-three/drei`                                                                                              |
| Express              | paquete `express`                                                                                                                               |
| Hono                 | paquete `hono`                                                                                                                                  |
| NestJS               | paquete `@nestjs/core`                                                                                                                          |
| Spring Boot          | Gradle con `spring-boot-starter` o `org.springframework.boot`                                                                                   |
| ASP.NET Core         | archivo `.csproj` con `Microsoft.NET.Sdk.Web`                                                                                                   |
| Blazor               | `.csproj` con `Microsoft.NET.Sdk.BlazorWebAssembly` o `Microsoft.AspNetCore.Components`                                                         |
| ASP.NET Minimal API  | `.csproj` con `Microsoft.AspNetCore.OpenApi` o `Swashbuckle.AspNetCore`                                                                         |

### Estilos y UI

| Tecnología   | Detectado desde                                          |
| ------------ | -------------------------------------------------------- |
| Tailwind CSS | `tailwindcss`, `@tailwindcss/vite` o `tailwind.config.*` |
| shadcn/ui    | `components.json`                                        |

### Runtimes y herramientas

| Tecnología | Detectado desde                                              |
| ---------- | ------------------------------------------------------------ |
| TypeScript | paquete `typescript` o `tsconfig.json`                       |
| Node.js    | `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `.nvmrc` |
| Bun        | `bun.lockb`, `bun.lock`, `bunfig.toml`                       |
| Deno       | `deno.json`, `deno.jsonc`, `deno.lock`                       |
| Dart       | `pubspec.yaml`                                               |
| Go         | `go.mod`, `go.work`                                          |
| Vite       | paquete `vite` o `vite.config.*`                             |
| Turborepo  | paquete `turbo` o `turbo.json`                               |
| Vitest     | paquete `vitest` o `vitest.config.*`                         |
| oxlint     | paquete `oxlint` o `.oxlintrc.json`                          |
| .NET       | `global.json`, `NuGet.Config`, `*.csproj`, `*.sln`           |
| C#         | `*.csproj`, `*.sln`                                          |

### Backend y datos

| Tecnología      | Detectado desde                                          |
| --------------- | -------------------------------------------------------- |
| Supabase        | `@supabase/supabase-js`, `@supabase/ssr`                 |
| Zod             | paquete `zod`                                            |
| React Hook Form | paquete `react-hook-form`                                |
| Neon Postgres   | `@neondatabase/serverless`                               |
| Prisma          | `prisma`, `@prisma/client`                               |
| Drizzle ORM     | `drizzle-orm`, `drizzle-kit`                             |
| Stripe          | `stripe`, `@stripe/stripe-js`, `@stripe/react-stripe-js` |
| Better Auth     | paquete `better-auth`                                    |

### Autenticación

| Tecnología | Detectado desde                                                                                                            |
| ---------- | -------------------------------------------------------------------------------------------------------------------------- |
| Clerk      | `@clerk/nextjs`, `@clerk/react`, `@clerk/expo`, `@clerk/astro`, `@clerk/remix`, `@clerk/vue`, o cualquier scope `@clerk/*` |

### Cloud y despliegue

| Tecnología        | Detectado desde                                      |
| ----------------- | ---------------------------------------------------- |
| Vercel            | `vercel.json`, `.vercel/`, `@astrojs/vercel`         |
| Cloudflare        | `wrangler`, `wrangler.toml`, `@astrojs/cloudflare`   |
| Cloudflare Agents | paquete `agents`                                     |
| Cloudflare AI     | `@cloudflare/ai` o binding AI en `wrangler.json`     |
| Durable Objects   | `durable_objects` en `wrangler.json`/`wrangler.toml` |
| Azure             | paquetes `@azure/*`                                  |
| AWS               | paquetes `@aws-sdk/*`, `aws-cdk*`                    |

### IA

| Tecnología    | Detectado desde                                               |
| ------------- | ------------------------------------------------------------- |
| Vercel AI SDK | `ai`, `@ai-sdk/openai`, `@ai-sdk/anthropic`, `@ai-sdk/google` |
| ElevenLabs    | paquete `elevenlabs`                                          |

### Otros

| Tecnología | Detectado desde                                                                            |
| ---------- | ------------------------------------------------------------------------------------------ |
| Playwright | `@playwright/test`, `playwright` o `playwright.config.*`                                   |
| SwiftUI    | `Package.swift`                                                                            |
| WordPress  | `wp-config.php`, `@wordpress/*`, `composer.json` con wpackagist, theme `style.css`         |
| Tauri      | `@tauri-apps/api`, `@tauri-apps/cli` o `src-tauri/tauri.conf.json`                         |
| Electron   | paquete `electron`, `electron-builder.yml`, `forge.config.js`, o `electron-vite.config.ts` |

### Detección de frontend web

Incluso sin framework, `skillindex` escanea tu árbol de archivos en busca de señales de frontend web (`.html`, `.css`, `.scss`, `.vue`, `.svelte`, `.jsx`, `.tsx`, `.twig`, `.blade.php`, etc.) e instala skills de diseño frontend, accesibilidad y SEO.

## Detección de combinaciones

Cuando se usan múltiples tecnologías juntas, `skillindex` detecta **combinaciones de tecnologías** y añade skills especializadas para la combinación:

- **Next.js + Supabase** — Mejores prácticas de Supabase Postgres para Next.js
- **Next.js + Vercel AI SDK** — Patrones del AI SDK con Next.js
- **Next.js + Playwright** — Mejores prácticas de testing E2E para Next.js
- **React + shadcn/ui** — Patrones de componentes shadcn con React
- **Tailwind CSS + shadcn/ui** — Integración Tailwind v4 + shadcn
- **Expo + Tailwind CSS** — Configuración de Tailwind para Expo
- **React Native + Expo** — Patrones de UI nativa
- **React Hook Form + Zod** — Patrones de validación de formularios con esquemas Zod
- **GSAP + React** — Patrones de animación GSAP en React
- **Cloudflare + Vite** — Guía de migración Vinext
- **Node.js + Express** — Patrones de servidor Express

## Cómo funciona

`skillindex` usa [skills.sh](https://skills.sh) por debajo — el registro abierto de skills para agentes IA. Las skills son archivos markdown que enseñan a los asistentes IA cómo trabajar con tecnologías específicas, siguiendo mejores prácticas y patrones de los mantenedores oficiales.

La detección se ejecuta completamente en local con cero peticiones de red hasta que comienza la instalación.

## Requisitos

- Node.js >= 22.0.0

## Licencia

MIT — Creado por [Gabriel Ortega](https://github.com/Gabox301)
