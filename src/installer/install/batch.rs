use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::registry::parse_skill_path;

use super::super::types::{InstallAllResult, InstallError, InstallOptions, SkillEntry};
use super::single::install_skill_with_client;

pub async fn install_all_with_client(
    skills: Vec<SkillEntry>,
    agents: &[String],
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> InstallAllResult {
    // sort by repo (parse_skill_path repo)
    let mut sorted: Vec<SkillEntry> = skills;
    sorted.sort_by(|a: &SkillEntry, b: &SkillEntry| {
        let ra: String = parse_skill_path(&a.skill).repo;
        let rb: String = parse_skill_path(&b.skill).repo;
        ra.cmp(&rb)
    });

    let concurrency: usize = 6usize;
    let semaphore: Arc<Semaphore> = Arc::new(Semaphore::new(concurrency));

    let mut handles: Vec<tokio::task::JoinHandle<(String, crate::installer::InstallResult)>> =
        Vec::new();
    for entry in sorted {
        let permit: tokio::sync::OwnedSemaphorePermit =
            semaphore.clone().acquire_owned().await.unwrap();
        let agents: Vec<String> = agents.to_vec();
        let opts: InstallOptions = opts.clone();
        let client: reqwest::Client = client.clone();
        let skill_clone: String = entry.skill.clone();
        handles.push(tokio::spawn(async move {
            let result: crate::installer::InstallResult =
                install_skill_with_client(&skill_clone, &agents, &opts, &client).await;
            drop(permit);
            (skill_clone, result)
        }));
    }

    let mut installed: usize = 0usize;
    let mut failed: usize = 0usize;
    let mut security_checks: Vec<crate::registry::InstallSecurityCheck> = Vec::new();
    let mut errors: Vec<InstallError> = Vec::new();

    for h in handles {
        let (skill_name, result) = h.await.unwrap();
        if result.success {
            installed += 1;
            if let Some(sc) = result.security_check {
                security_checks.push(sc);
            }
        } else {
            failed += 1;
            errors.push(InstallError {
                name: skill_name,
                output: result.output,
                stderr: result.stderr,
                exit_code: result.exit_code,
                command: result.command,
            });
        }
    }

    InstallAllResult {
        installed,
        failed,
        security_checks,
        errors,
    }
}

pub async fn install_all(
    skills: Vec<SkillEntry>,
    agents: Vec<String>,
    opts: InstallOptions,
) -> InstallAllResult {
    let client: reqwest::Client = reqwest::Client::builder()
        .user_agent("skillindex")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    install_all_with_client(skills, &agents, &opts, &client).await
}
