pub mod activeadmin;
pub mod android;
pub mod angular;
pub mod appwrite;
pub mod aspnet_blazor;
pub mod aspnet_minimal_api;
pub mod aspnetcore;
pub mod astro;
pub mod aws;
pub mod azure;
pub mod bash;
pub mod better_auth;
pub mod bun;
pub mod celery;
pub mod chrome_extension;
pub mod clerk;
pub mod cloudflare;
pub mod cloudflare_agents;
pub mod cloudflare_ai;
pub mod cloudflare_durable_objects;
pub mod csharp;
pub mod dart;
pub mod deno;
pub mod devise;
pub mod django;
pub mod dotnet;
pub mod drizzle;
pub mod electron;
pub mod elevenlabs;
pub mod elysia;
pub mod expo;
pub mod express;
pub mod fastapi;
pub mod fastmcp;
pub mod flask;
pub mod flutter;
pub mod go;
pub mod gsap;
pub mod hono;
pub mod instantdb;
pub mod java;
pub mod kotlin_multiplatform;
pub mod laravel;
pub mod mattpocock_skills;
pub mod neon;
pub mod nestjs;
pub mod nextjs;
pub mod node;
pub mod numpy;
pub mod nuxt;
pub mod oxlint;
pub mod pandas;
pub mod php;
pub mod pinia;
pub mod playwright;
pub mod postgres_ruby;
pub mod prisma;
pub mod pydantic;
pub mod pytest;
pub mod python;
pub mod rails;
pub mod react;
pub mod react_hook_form;
pub mod react_native;
pub mod react_router;
pub mod react_three_fiber;
pub mod redis_ruby;
pub mod remotion;
pub mod requests;
pub mod rspec;
pub mod rubocop;
pub mod ruby;
pub mod rust;
pub mod scikit_learn;
pub mod shadcn;
pub mod sidekiq;
pub mod sorbet;
pub mod springboot;
pub mod sqlalchemy;
pub mod stripe;
pub mod supabase;
pub mod svelte;
pub mod swiftui;
pub mod tailwind;
pub mod tanstack_start;
pub mod tauri;
pub mod terraform;
pub mod threejs;
pub mod turborepo;
pub mod typescript;
pub mod vercel_ai;
pub mod vercel_deploy;
pub mod vite;
pub mod vitest;
pub mod vue;
pub mod wordpress;
pub mod zod;

pub use activeadmin::ACTIVEADMIN_TECH;
pub use android::ANDROID_TECH;
pub use angular::ANGULAR_TECH;
pub use appwrite::APPWRITE_TECH;
pub use aspnet_blazor::ASPNET_BLAZOR_TECH;
pub use aspnet_minimal_api::ASPNET_MINIMAL_API_TECH;
pub use aspnetcore::ASPNETCORE_TECH;
pub use astro::ASTRO_TECH;
pub use aws::AWS_TECH;
pub use azure::AZURE_TECH;
pub use bash::BASH_TECH;
pub use better_auth::BETTER_AUTH_TECH;
pub use bun::BUN_TECH;
pub use celery::CELERY_TECH;
pub use chrome_extension::CHROME_EXTENSION_TECH;
pub use clerk::CLERK_TECH;
pub use cloudflare::CLOUDFLARE_TECH;
pub use cloudflare_agents::CLOUDFLARE_AGENTS_TECH;
pub use cloudflare_ai::CLOUDFLARE_AI_TECH;
pub use cloudflare_durable_objects::CLOUDFLARE_DURABLE_OBJECTS_TECH;
pub use csharp::CSHARP_TECH;
pub use dart::DART_TECH;
pub use deno::DENO_TECH;
pub use devise::DEVISE_TECH;
pub use django::DJANGO_TECH;
pub use dotnet::DOTNET_TECH;
pub use drizzle::DRIZZLE_TECH;
pub use electron::ELECTRON_TECH;
pub use elevenlabs::ELEVENLABS_TECH;
pub use elysia::ELYSIA_TECH;
pub use expo::EXPO_TECH;
pub use express::EXPRESS_TECH;
pub use fastapi::FASTAPI_TECH;
pub use fastmcp::FASTMCP_TECH;
pub use flask::FLASK_TECH;
pub use flutter::FLUTTER_TECH;
pub use go::GO_TECH;
pub use gsap::GSAP_TECH;
pub use hono::HONO_TECH;
pub use instantdb::INSTANTDB_TECH;
pub use java::JAVA_TECH;
pub use kotlin_multiplatform::KOTLIN_MULTIPLATFORM_TECH;
pub use laravel::LARAVEL_TECH;
pub use mattpocock_skills::MATTPOCOCK_SKILLS_TECH;
pub use neon::NEON_TECH;
pub use nestjs::NESTJS_TECH;
pub use nextjs::NEXTJS_TECH;
pub use node::NODE_TECH;
pub use numpy::NUMPY_TECH;
pub use nuxt::NUXT_TECH;
pub use oxlint::OXLINT_TECH;
pub use pandas::PANDAS_TECH;
pub use php::PHP_TECH;
pub use pinia::PINIA_TECH;
pub use playwright::PLAYWRIGHT_TECH;
pub use postgres_ruby::POSTGRES_RUBY_TECH;
pub use prisma::PRISMA_TECH;
pub use pydantic::PYDANTIC_TECH;
pub use pytest::PYTEST_TECH;
pub use python::PYTHON_TECH;
pub use rails::RAILS_TECH;
pub use react::REACT_TECH;
pub use react_hook_form::REACT_HOOK_FORM_TECH;
pub use react_native::REACT_NATIVE_TECH;
pub use react_router::REACT_ROUTER_TECH;
pub use redis_ruby::REDIS_RUBY_TECH;
pub use remotion::REMOTION_TECH;
pub use requests::REQUESTS_TECH;
pub use rspec::RSPEC_TECH;
pub use rubocop::RUBOCOP_TECH;
pub use ruby::RUBY_TECH;
pub use rust::RUST_TECH;
pub use scikit_learn::SCIKIT_LEARN_TECH;
pub use shadcn::SHADCN_TECH;
pub use sidekiq::SIDEKIQ_TECH;
pub use sorbet::SORBET_TECH;
pub use springboot::SPRINGBOOT_TECH;
pub use sqlalchemy::SQLALCHEMY_TECH;
pub use stripe::STRIPE_TECH;
pub use supabase::SUPABASE_TECH;
pub use svelte::SVELTE_TECH;
pub use swiftui::SWIFTUI_TECH;
pub use tailwind::TAILWIND_TECH;
pub use tanstack_start::TANSTACK_START_TECH;
pub use tauri::TAURI_TECH;
pub use terraform::TERRAFORM_TECH;
pub use threejs::THREEJS_TECH;
pub use turborepo::TURBOREPO_TECH;
pub use typescript::TYPESCRIPT_TECH;
pub use vercel_ai::VERCEL_AI_TECH;
pub use vercel_deploy::VERCEL_DEPLOY_TECH;
pub use vite::VITE_TECH;
pub use vitest::VITEST_TECH;
pub use vue::VUE_TECH;
pub use wordpress::WORDPRESS_TECH;
pub use zod::ZOD_TECH;

pub const TECHS: &[crate::skills::types::Technology] = &[
    ACTIVEADMIN_TECH,
    ANDROID_TECH,
    ANGULAR_TECH,
    APPWRITE_TECH,
    ASPNET_BLAZOR_TECH,
    ASPNET_MINIMAL_API_TECH,
    ASPNETCORE_TECH,
    ASTRO_TECH,
    AWS_TECH,
    AZURE_TECH,
    BASH_TECH,
    BETTER_AUTH_TECH,
    BUN_TECH,
    CELERY_TECH,
    CHROME_EXTENSION_TECH,
    CLERK_TECH,
    CLOUDFLARE_TECH,
    CLOUDFLARE_AGENTS_TECH,
    CLOUDFLARE_AI_TECH,
    CLOUDFLARE_DURABLE_OBJECTS_TECH,
    CSHARP_TECH,
    DART_TECH,
    DENO_TECH,
    DEVISE_TECH,
    DJANGO_TECH,
    DOTNET_TECH,
    DRIZZLE_TECH,
    ELECTRON_TECH,
    ELEVENLABS_TECH,
    ELYSIA_TECH,
    EXPO_TECH,
    EXPRESS_TECH,
    FASTAPI_TECH,
    FASTMCP_TECH,
    FLASK_TECH,
    FLUTTER_TECH,
    GO_TECH,
    GSAP_TECH,
    HONO_TECH,
    INSTANTDB_TECH,
    JAVA_TECH,
    KOTLIN_MULTIPLATFORM_TECH,
    LARAVEL_TECH,
    MATTPOCOCK_SKILLS_TECH,
    NEON_TECH,
    NESTJS_TECH,
    NEXTJS_TECH,
    NODE_TECH,
    NUMPY_TECH,
    NUXT_TECH,
    OXLINT_TECH,
    PANDAS_TECH,
    PHP_TECH,
    PINIA_TECH,
    PLAYWRIGHT_TECH,
    POSTGRES_RUBY_TECH,
    PRISMA_TECH,
    PYDANTIC_TECH,
    PYTEST_TECH,
    PYTHON_TECH,
    RAILS_TECH,
    REACT_TECH,
    REACT_HOOK_FORM_TECH,
    REACT_NATIVE_TECH,
    REACT_ROUTER_TECH,
    REDIS_RUBY_TECH,
    REMOTION_TECH,
    REQUESTS_TECH,
    RSPEC_TECH,
    RUBOCOP_TECH,
    RUBY_TECH,
    RUST_TECH,
    SCIKIT_LEARN_TECH,
    SHADCN_TECH,
    SIDEKIQ_TECH,
    SORBET_TECH,
    SPRINGBOOT_TECH,
    SQLALCHEMY_TECH,
    STRIPE_TECH,
    SUPABASE_TECH,
    SVELTE_TECH,
    SWIFTUI_TECH,
    TAILWIND_TECH,
    TANSTACK_START_TECH,
    TAURI_TECH,
    TERRAFORM_TECH,
    THREEJS_TECH,
    TURBOREPO_TECH,
    TYPESCRIPT_TECH,
    VERCEL_AI_TECH,
    VERCEL_DEPLOY_TECH,
    VITE_TECH,
    VITEST_TECH,
    VUE_TECH,
    WORDPRESS_TECH,
    ZOD_TECH,
];
