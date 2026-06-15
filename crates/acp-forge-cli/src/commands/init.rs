//! `acp-forge init` — set up a forge session directory.

use std::fs;
use std::path::Path;

pub async fn run(base: &str) -> anyhow::Result<()> {
    let forge_dir = Path::new(".forge");
    if forge_dir.exists() {
        anyhow::bail!(".forge directory already exists");
    }

    fs::create_dir_all(forge_dir.join("sessions"))?;
    fs::create_dir_all(forge_dir.join("skills"))?;
    fs::create_dir_all(forge_dir.join("worktrees"))?;

    let config = format!(
        r#"# ACP Forge configuration
[session]
base_ref = "{base}"
max_agents = 8

[worktree]
root = ".forge/worktrees"

[cache]
ttl_seconds = 300
"#,
    );

    fs::write(forge_dir.join("config.toml"), config)?;

    tracing::info!(base, "forge initialized");
    println!("✓ Initialized .forge/ (base: {base})");
    Ok(())
}
