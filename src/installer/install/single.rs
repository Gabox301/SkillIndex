use std::env;
use std::path::PathBuf;

use crate::registry::{
    load_registry, load_registry_from_dir, parse_skill_path, security_check_for_entry,
    verify_registry_entry,
};

use super::super::download::{materialize_skill_into, update_skills_lock};
use super::super::helpers::rel_path_from_to;
use super::super::targets::resolve_install_targets;
use super::super::types::{InstallOptions, InstallResult};

pub async fn install_skill_with_client(
    skill_path: &str,
    agents: &[String],
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> InstallResult {
    let project_dir: PathBuf = opts
        .project_dir
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let command: String = format!("skillindex install {skill_path}");

    let fail = |msg: String| InstallResult {
        success: false,
        output: msg.clone(),
        stderr: msg,
        exit_code: Some(1),
        command: command.clone(),
        security_check: None,
    };

    let parsed: crate::registry::ParsedSkillPath = parse_skill_path(skill_path);
    if parsed.skill_name.is_empty() {
        return fail(format!("ruta de skill no válida: {skill_path}"));
    }
    let skill_name: String = parsed.skill_name;

    // load registry
    let registry: Option<crate::registry::Registry> = if let Some(dir) = &opts.registry_dir {
        load_registry_from_dir(dir)
    } else {
        load_registry()
    };
    let Some(registry) = registry else {
        return fail(
            "índice de skills-registry no encontrado. Ejecuta 'pnpm sync:skills' en el paquete skillindex."
                .to_string(),
        );
    };
    let Some(entry) = registry.skills.get(&skill_name) else {
        return fail(format!(
            "skill '{skill_name}' no encontrada en el registro (no auditada)."
        ));
    };
    let security_check: crate::registry::InstallSecurityCheck =
        security_check_for_entry(&skill_name, entry);

    // Copy the skill directly into each destination. Each mapped agent gets its
    // own real copy; `.agents` is used only for the universal destination. A
    // target is skipped when it already exists and matches the registry hash, so
    // only missing or drifted targets are (re)installed.
    let targets: Vec<crate::installer::InstallTarget> =
        resolve_install_targets(&project_dir, agents);
    let mut install_errors: Vec<String> = Vec::new();
    for target in &targets {
        let dest_dir: PathBuf = target.skills_dir.join(&skill_name);
        let verdict: crate::registry::VerifyResult =
            verify_registry_entry(&skill_name, entry, &target.skills_dir);
        if verdict.ok {
            continue;
        }
        if let Err(e) = materialize_skill_into(&skill_name, entry, &dest_dir, opts, client).await {
            install_errors.push(format!("{}: {e}", target.folder));
        }
    }

    if !install_errors.is_empty() {
        let msg: String = format!("falló la instalación: {}", install_errors.join("; "));
        return InstallResult {
            success: false,
            output: msg.clone(),
            stderr: msg,
            exit_code: Some(1),
            command,
            security_check: None,
        };
    }

    if let Err(e) = update_skills_lock(&project_dir, &skill_name, entry) {
        return fail(format!("falló la actualización del lockfile: {e}"));
    }

    let destinations: Vec<String> = targets
        .iter()
        .map(|t: &crate::installer::InstallTarget| {
            rel_path_from_to(&project_dir, &t.skills_dir.join(&skill_name))
        })
        .collect();

    InstallResult {
        success: true,
        output: format!("instalada {} en {}", skill_name, destinations.join(", ")),
        stderr: String::new(),
        exit_code: Some(0),
        command,
        security_check: Some(security_check),
    }
}

pub async fn install_skill(
    skill_path: &str,
    agents: &[String],
    opts: InstallOptions,
) -> InstallResult {
    let client: reqwest::Client = reqwest::Client::builder()
        .user_agent("skillindex")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    install_skill_with_client(skill_path, agents, &opts, &client).await
}
