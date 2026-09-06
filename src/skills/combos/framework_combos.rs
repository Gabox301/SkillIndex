use crate::skills::types::ComboSkill;

pub const FRAMEWORK_COMBOS: &[ComboSkill] = &[
    ComboSkill {
        id: "expo-tailwind",
        name: "Expo + Tailwind CSS",
        requires: &["expo", "tailwind"],
        skills: &["expo/skills/expo-tailwind-setup"],
    },
    ComboSkill {
        id: "react-hook-form-zod",
        name: "React Hook Form + Zod",
        requires: &["react-hook-form", "zod"],
        skills: &["pproenca/dot-skills/zod"],
    },
    ComboSkill {
        id: "nextjs-supabase",
        name: "Next.js + Supabase",
        requires: &["nextjs", "supabase"],
        skills: &["supabase/agent-skills/supabase-postgres-best-practices"],
    },
    ComboSkill {
        id: "react-native-expo",
        name: "React Native + Expo",
        requires: &["react-native", "expo"],
        skills: &[
            "expo/skills/building-native-ui",
            "sleekdotdesign/agent-skills/design-mobile-apps",
        ],
    },
    ComboSkill {
        id: "nextjs-vercel-ai",
        name: "Next.js + Vercel AI SDK",
        requires: &["nextjs", "vercel-ai"],
        skills: &[
            "vercel/ai/use-ai-sdk",
            "vercel-labs/next-skills/next-best-practices",
        ],
    },
    ComboSkill {
        id: "nextjs-playwright",
        name: "Next.js + Playwright",
        requires: &["nextjs", "playwright"],
        skills: &["currents-dev/playwright-best-practices-skill/playwright-best-practices"],
    },
    ComboSkill {
        id: "react-shadcn",
        name: "React + shadcn/ui",
        requires: &["react", "shadcn"],
        skills: &[
            "shadcn/ui/shadcn",
            "vercel-labs/agent-skills/react-best-practices",
        ],
    },
    ComboSkill {
        id: "tailwind-shadcn",
        name: "Tailwind CSS + shadcn/ui",
        requires: &["tailwind", "shadcn"],
        skills: &["secondsky/claude-skills/tailwind-v4-shadcn"],
    },
    ComboSkill {
        id: "gsap-react",
        name: "GSAP + React",
        requires: &["gsap", "react"],
        skills: &["greensock/gsap-skills/gsap-react"],
    },
    ComboSkill {
        id: "cloudflare-vite",
        name: "Cloudflare + Vite",
        requires: &["cloudflare", "vite"],
        skills: &["cloudflare/vinext/migrate-to-vinext"],
    },
    ComboSkill {
        id: "node-express",
        name: "Node.js + Express",
        requires: &["node", "express"],
        skills: &["aj-geddes/useful-ai-prompts/nodejs-express-server"],
    },
    ComboSkill {
        id: "nextjs-clerk",
        name: "Next.js + Clerk",
        requires: &["nextjs", "clerk"],
        skills: &["clerk/skills/clerk-nextjs-patterns"],
    },
    ComboSkill {
        id: "nuxt-clerk",
        name: "Nuxt + Clerk",
        requires: &["nuxt", "clerk"],
        skills: &["clerk/skills/clerk-nuxt-patterns"],
    },
    ComboSkill {
        id: "vue-clerk",
        name: "Vue + Clerk",
        requires: &["vue", "clerk"],
        skills: &["clerk/skills/clerk-vue-patterns"],
    },
    ComboSkill {
        id: "react-clerk",
        name: "React + Clerk",
        requires: &["react", "clerk"],
        skills: &["clerk/skills/clerk-react-patterns"],
    },
    ComboSkill {
        id: "astro-clerk",
        name: "Astro + Clerk",
        requires: &["astro", "clerk"],
        skills: &["clerk/skills/clerk-astro-patterns"],
    },
    ComboSkill {
        id: "expo-clerk",
        name: "Expo + Clerk",
        requires: &["expo", "clerk"],
        skills: &["clerk/skills/clerk-expo-patterns"],
    },
    ComboSkill {
        id: "react-react-three-fiber",
        name: "React + React Three Fiber",
        requires: &["threejs", "react", "@react-three/fiber"],
        skills: &["vercel-labs/json-render/react-three-fiber"],
    },
    ComboSkill {
        id: "react-router-clerk",
        name: "React Router + Clerk",
        requires: &["react-router", "clerk"],
        skills: &[
            "clerk/skills/clerk-react-router-patterns",
            "clerk/skills/clerk-setup",
            "clerk/skills/clerk",
        ],
    },
    ComboSkill {
        id: "tanstack-clerk",
        name: "TanStack Start + Clerk",
        requires: &["tanstack-start", "clerk"],
        skills: &[
            "clerk/skills/clerk-tanstack-patterns",
            "clerk/skills/clerk-setup",
            "clerk/skills/clerk",
        ],
    },
    ComboSkill {
        id: "chrome-extension-clerk",
        name: "Chrome Extension + Clerk",
        requires: &["chrome-extension", "clerk"],
        skills: &[
            "clerk/skills/clerk-chrome-extension-patterns",
            "clerk/skills/clerk-setup",
            "clerk/skills/clerk",
        ],
    },
    ComboSkill {
        id: "swiftui-clerk",
        name: "SwiftUI + Clerk",
        requires: &["swiftui", "clerk"],
        skills: &[
            "clerk/skills/clerk-swift",
            "clerk/skills/clerk-setup",
            "clerk/skills/clerk",
        ],
    },
    ComboSkill {
        id: "android-clerk",
        name: "Android + Clerk",
        requires: &["android", "clerk"],
        skills: &[
            "clerk/skills/clerk-android",
            "clerk/skills/clerk-setup",
            "clerk/skills/clerk",
        ],
    },
    ComboSkill {
        id: "rails-rspec",
        name: "Ruby on Rails + RSpec",
        requires: &["rails", "rspec"],
        skills: &[
            "igmarin/rails-agent-skills/rails-tdd-slices",
            "igmarin/rails-agent-skills/rails-bug-triage",
        ],
    },
    ComboSkill {
        id: "rails-sidekiq",
        name: "Ruby on Rails + Sidekiq",
        requires: &["rails", "sidekiq"],
        skills: &[],
    },
];
