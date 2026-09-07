use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // fendo checks from repo root (where .npmrc, package.json, lockfiles live)
    // When run via cargo, CARGO_MANIFEST_DIR is the crate root (project root after migration)
    let root: PathBuf = manifest_dir.clone();

    let mut failed: bool = false;

    let ok = |msg: &str| println!("✔ {msg}");
    let mut fail = |msg: &str| {
        eprintln!("✘ {msg}");
        failed = true;
    };

    println!("fendo — GaboTech supply-chain check\n");

    // Check package.json files
    for rel in ["package.json", "packages/skillindex/package.json"] {
        let path: PathBuf = root.join(rel);
        if !path.exists() {
            continue;
        }
        let content: String = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let pkg: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let mut all_deps: Vec<(String, String)> = Vec::new();
        for key in [
            "dependencies",
            "devDependencies",
            "peerDependencies",
            "optionalDependencies",
        ] {
            if let Some(obj) = pkg.get(key).and_then(|v| v.as_object()) {
                for (name, ver) in obj {
                    if let Some(s) = ver.as_str() {
                        all_deps.push((name.clone(), s.to_string()));
                    }
                }
            }
        }
        for (name, ver) in &all_deps {
            let v: &str = ver.trim();
            if v.starts_with('^') || v.starts_with('~') {
                fail(&format!(
                    "{}: {name}@{v} uses ^/~ — pin exact version",
                    path.display()
                ));
            }
            let lower: String = v.to_lowercase();
            let is_git_or_tarball: bool = lower.starts_with("git+")
                || lower.starts_with("github:")
                || (lower.contains("://")
                    && (lower.ends_with(".tgz") || lower.ends_with(".tar.gz")))
                || v.contains("://") && !v.starts_with("https://raw.githubusercontent.com/");
            // Allow raw.githubusercontent.com
            if v.starts_with("https://raw.githubusercontent.com/") {
                continue;
            }
            if is_git_or_tarball {
                // Check for URL-like
                if v.contains("://") {
                    fail(&format!(
                        "{}: {name}@{v} looks like git/tarball URL — requires explicit approval",
                        path.display()
                    ));
                }
            }
        }
        if !all_deps.is_empty() {
            ok(&format!("{}: versions pinned", path.display()));
        }
    }

    // Check .npmrc
    let npmrc_path: PathBuf = root.join(".npmrc");
    if !npmrc_path.exists() {
        fail(".npmrc missing — hardening not applied");
    } else {
        let txt: String = fs::read_to_string(&npmrc_path).unwrap_or_default();
        let checks: [(&str, &str); 3] = [
            ("save-exact=true", "save-exact=true"),
            ("ignore-scripts=true", "ignore-scripts=true"),
            ("engine-strict=true", "engine-strict=true"),
        ];
        for (needle, label) in checks {
            if !txt.contains(needle) {
                fail(&format!(".npmrc missing {label}"));
            } else {
                ok(&format!(".npmrc has {label}"));
            }
        }
        if txt.contains("minimum-release-age") {
            ok(".npmrc has minimum-release-age");
        }
    }

    // Check lockfile
    let candidates: [&str; 5] = [
        "bun.lock",
        "bun.lockb",
        "pnpm-lock.yaml",
        "package-lock.json",
        "yarn.lock",
    ];
    let mut found: Vec<String> = Vec::new();
    for f in candidates {
        if root.join(f).exists() {
            found.push(f.to_string());
        }
    }
    let pkg_candidates: [&str; 3] = ["pnpm-lock.yaml", "bun.lock", "bun.lockb"];
    let mut pkg_found: Vec<String> = Vec::new();
    for f in pkg_candidates {
        if root.join("packages/skillindex").join(f).exists() {
            pkg_found.push(f.to_string());
        }
        // Also check at root's packages if it still exists, otherwise at root
        if root.join(f).exists() && !found.contains(&f.to_string()) {
            // already counted
        }
    }
    if found.is_empty() && pkg_found.is_empty() {
        fail("No lockfile found (bun.lock / pnpm-lock.yaml) — commit it");
    } else {
        let all_found: Vec<String> = [found.clone(), pkg_found.clone()].concat();
        ok(&format!("Lockfile present: {}", all_found.join(", ")));
    }

    // Check .gitignore doesn't ignore lockfile
    let gi_path: PathBuf = root.join(".gitignore");
    if gi_path.exists() {
        let txt: String = fs::read_to_string(&gi_path).unwrap_or_default();
        let lines: Vec<String> = txt
            .split(|c: char| ['\n', '\r'].contains(&c))
            .map(|l: &str| l.trim().to_string())
            .collect();
        for f in &found {
            if lines.contains(f)
                || lines.contains(&format!("/{}", f))
                || lines.contains(&format!("**/{}", f))
            {
                fail(&format!(".gitignore ignores {f} — must be committed"));
            }
        }
    }

    if failed {
        eprintln!("\n✘ fendo: hardening failed — fix issues above");
        std::process::exit(1);
    } else {
        println!("\n✔ fendo: all hardening checks passed");
    }
}
