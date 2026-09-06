use std::process::Command;

fn main() -> anyhow::Result<()> {
    // Delegar al script TS original hasta completar la migración 100% Rust
    // Mantiene compatibilidad con `cargo run --bin sync-skills -- --only book-to-skill`
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = manifest_dir.join("scripts").join("sync-skills.ts");
    let args: Vec<String> = std::env::args().skip(1).collect();

    if script.exists() {
        // Intentar con bun, fallback a node
        let mut cmd = Command::new("bun");
        cmd.arg("run").arg(&script).args(&args);
        let status = cmd.status();
        if let Ok(s) = status
            && (s.success() || s.code().is_some())
        {
            std::process::exit(s.code().unwrap_or(1));
        }
        // Fallback a npx/bunx implícito si bun no está
        let mut cmd2 = Command::new("npx");
        cmd2.arg("bun").arg("run").arg(&script).args(&args);
        let status2 = cmd2.status()?;
        std::process::exit(status2.code().unwrap_or(1));
    }

    eprintln!(
        "sync-skills: script TS no encontrado en {}",
        script.display()
    );
    eprintln!("Tip: este binario será reescrito 100% Rust en la próxima iteración.");
    std::process::exit(1);
}
